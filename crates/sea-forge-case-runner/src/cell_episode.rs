//! Governed cell episodes: one command in an OpenShell sandbox, its changeset
//! review, and a separately approved promotion.
//!
//! The ordering is the security contract, and it does not change:
//!
//! 1. The projection is read and its identity (Cell, model, policy digest) is
//!    taken from the locked projection. A malformed or tampered projection, or
//!    a binding without a sandbox image, fails before any authority call.
//! 2. If the policy configures an `opa` engine and the caller supplies an
//!    input, the OPA candidate is evaluated and its trace is committed before
//!    the decision. Without an input the engine decides on its own.
//! 3. The sandbox command is authorized as `execute_sandbox_command` and the
//!    decision is committed. A denial stops here: the repository is not read,
//!    nothing is exported, and no openshell process runs.
//! 4. Only after an Allow is the base captured, exported to a private
//!    workspace, and executed. The sandbox is destroyed before its result is
//!    trusted, and the returned tree is evidence, not a decision.
//! 5. The returned tree becomes a changeset that is validated and settled.
//! 6. If settlement accepted the work and validation accepted a non-empty
//!    changeset, the episode requests promotion. `promote_changeset` must
//!    escalate; the request is returned and the authoritative ref is untouched.
//! 7. A human approver resolves the escalation. `promote_cell_changeset` then
//!    grants that exact action once, checks it against the changeset, and moves
//!    the authoritative ref by compare-and-swap.
//!
//! External effects of the command are outside this transaction. The ledger
//! records what was decided and observed; it does not undo what a command did
//! inside the sandbox.

use sea_forge_authority::{
    AuthorityEvaluation, AuthorityPolicyBundle, DomainForgeCandidate, OpaBinding, OpaCandidate,
    PolicyAuthorityEngine,
};
use sea_forge_changeset::{
    capture_base, collect, export_base, promote, validate, BaseIdentity, ChangeRules, Changeset,
    ChangesetError, Promotion, PromotionAuthorization, ProtectedDecision, Validation,
};
use sea_forge_core::{errors::ForgeError, types::*};
use sea_forge_evidence::{capture_file, JsonlEvidenceWriter};
use sea_forge_ledger::{CommittedRecordRef, LedgerStream};
use sea_forge_sandbox::{projection_identity, OpenShellBinding};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// An OPA candidate input for the sandbox-command decision. Used only when the
/// policy configures an `opa` engine.
pub struct CellOpaInput {
    pub binding: OpaBinding,
    /// The `domainforge.rego.input/v1` document, `{"facts": {...}}`.
    pub input: Value,
}

/// Everything one cell episode needs. Borrowed inputs are operator-owned or
/// run-owned; nothing here is read from the host environment.
pub struct CellEpisodeRequest<'a> {
    /// Authority policy bundle, outside the workspace.
    pub policy_path: &'a Path,
    /// Actor id.
    pub entity: &'a str,
    pub case_id: &'a str,
    pub run_id: &'a str,
    pub plan_item_id: &'a str,
    /// Everything this run writes lives under here.
    pub run_dir: &'a Path,
    /// Authoritative Git repository.
    pub repository: &'a Path,
    /// Full ref, e.g. `refs/heads/main`.
    pub reference: &'a str,
    /// Must carry an explicit sandbox image.
    pub binding: OpenShellBinding,
    /// Command to run in the sandbox workspace.
    pub argv: Vec<String>,
    /// Passed to the sandbox verbatim; never the host environment.
    pub env: BTreeMap<String, String>,
    pub criteria: SettlementCriteria,
    /// OPA input for the sandbox command, if the policy configures `opa`.
    pub opa: Option<CellOpaInput>,
    /// Timestamp for the proposal commit (RFC 3339).
    pub committed_at: &'a str,
}

/// What one episode produced. `promotion_request` is set only when the episode
/// asked for a promotion and it is waiting for a separate approval.
#[derive(Debug)]
pub struct CellEpisode {
    pub decision: AuthorityDecision,
    pub execution: Option<ExecutionResult>,
    pub changeset: Option<Changeset>,
    pub validation: Option<Validation>,
    pub settlement: SettlementEvent,
    pub promotion_request: Option<PromotionRequest>,
}

/// A promotion the episode asked for. It is not a grant: it carries the
/// escalated decision and the changeset it covers. Only
/// `promote_cell_changeset`, with a matching human approval, can act on it.
#[derive(Debug, Clone)]
pub struct PromotionRequest {
    pub action: AuthorityAction,
    pub decision: AuthorityDecision,
    pub committed_decision: CommittedRecordRef,
    pub changeset: Changeset,
}

/// The approval a promotion needs, with both records already committed.
pub struct PromotionApproval<'a> {
    pub approval: &'a ApprovalRequest,
    pub committed_approval: &'a CommittedRecordRef,
    pub criteria: &'a SettlementCriteriaRecord,
    pub committed_criteria: &'a CommittedRecordRef,
}

/// Run one governed cell episode. See the module documentation for the order.
pub fn run_cell_episode(
    request: CellEpisodeRequest<'_>,
    stream: &LedgerStream,
) -> Result<CellEpisode, ForgeError> {
    let CellEpisodeRequest {
        policy_path,
        entity,
        case_id,
        run_id,
        plan_item_id,
        run_dir,
        repository,
        reference,
        binding,
        argv,
        env,
        criteria,
        opa,
        committed_at,
    } = request;

    // 1. Fail closed on a malformed projection or a missing image, before any authority call.
    let projection = read_projection(&binding.projection_dir)?;
    let identity =
        projection_identity(&binding.projection_dir).map_err(|error| ForgeError::Config {
            class: error.class,
            path: binding.projection_dir.clone(),
            message: error.message,
        })?;
    let image = binding.image.clone().ok_or_else(|| ForgeError::Config {
        class: "openshell_image_unspecified",
        path: binding.projection_dir.clone(),
        message: "the OpenShell binding names no sandbox image".into(),
    })?;
    let layout = Layout::new(run_dir);

    // 2. Optional OPA candidate, committed before the decision it informs.
    let authority = Authority::open(policy_path, entity)?;
    let opa_candidate = match opa {
        Some(opa) if authority.configures_opa() => {
            let candidate = OpaCandidate::evaluate(&authority.bundle, &opa.binding, &opa.input)?;
            stream.commit_typed(
                "opa_evaluation",
                vec![run_id.into(), plan_item_id.into()],
                candidate.trace(),
                vec![],
            )?;
            Some(candidate)
        }
        _ => None,
    };

    // 3. Authorize the sandbox command, and commit the decision before anything else.
    let env_keys: BTreeSet<String> = env.keys().cloned().collect();
    let operation = Operation::ExecuteSandboxCommand {
        argv,
        cwd: ".".into(),
        cell_id: identity.cell_id,
        model_hash: identity.model_hash,
        policy_digest: identity.policy_digest,
        image,
    };
    let action = AuthorityAction::from(&operation);
    let decision = authority.evaluate(
        Scope {
            case_id,
            run_id,
            plan_item_id,
            sequence: 1,
            workspace: &layout.workspace,
            artifacts: &layout.artifacts,
            timeout_secs: Some(projection.timeout_secs),
            env_keys,
        },
        &action,
        opa_candidate.as_ref(),
    )?;
    let decision_ref = stream.commit_typed(
        "authority_decision",
        vec![run_id.into(), plan_item_id.into()],
        &decision,
        vec![],
    )?;

    // Not allowed: no repository access, no export, no openshell process.
    if decision.verdict != Verdict::Allow {
        let settlement = settle_episode(
            run_id,
            plan_item_id,
            criteria,
            None,
            vec![decision.verdict.clone()],
            &layout.workspace,
            run_dir,
            stream,
        )?;
        return Ok(CellEpisode {
            decision,
            execution: None,
            changeset: None,
            validation: None,
            settlement,
            promotion_request: None,
        });
    }

    // 4. Only now touch the repository and the sandbox.
    let base = capture_base(repository, reference).map_err(|e| config_error(e, repository))?;
    export_base(repository, &base, &layout.staging, &layout.workspace)
        .map_err(|e| config_error(e, repository))?;
    fs::create_dir_all(&layout.artifacts).map_err(|e| ForgeError::io("create artifacts", e))?;
    let grant = authority
        .engine
        .grant(&decision, &decision_ref, &action, None)?;
    let exec_request = ExecutionRequest {
        plan_item_id: plan_item_id.into(),
        operation,
        timeout_secs: projection.timeout_secs,
        env,
        compensating_controls: decision.compensating_controls.clone(),
    };
    let execution = sea_forge_runtime::execute_openshell(
        grant,
        &exec_request,
        run_id,
        &layout.workspace,
        &layout.artifacts,
        binding,
        &layout.result,
    )?;

    // 5. Record what the sandbox reported, then the evidence files.
    let execution_ref = record_execution(
        stream,
        &layout,
        run_id,
        plan_item_id,
        &decision.decision_id,
        &execution,
    )?;
    capture_evidence(&layout, run_id, execution_ref.entry_ulid())?;

    // 6. Changeset review. A malformed workspace is recorded, not fatal.
    let review = review_workspace(
        repository,
        &base,
        &layout,
        committed_at,
        &projection.rules,
        stream,
    )?;

    // 7. Settle on the execution verdict plus the validation verdict.
    let settlement = settle_episode(
        run_id,
        plan_item_id,
        criteria,
        Some(execution.clone()),
        vec![
            decision.verdict.clone(),
            validation_verdict(&review.validation),
        ],
        &layout.workspace,
        run_dir,
        stream,
    )?;

    // 8. Request promotion only with settlement acceptance and a valid, non-empty changeset.
    let promotion_request = request_promotion(
        &authority,
        PromotionContext {
            case_id,
            run_id,
            plan_item_id,
            reference,
            repository,
            layout: &layout,
            base: &base,
            stream,
        },
        &settlement,
        review.changeset.as_ref(),
        &review.validation,
        opa_candidate.as_ref(),
    )?;

    Ok(CellEpisode {
        decision,
        execution: Some(execution),
        changeset: review.changeset,
        validation: review.validation,
        settlement,
        promotion_request,
    })
}

/// The proposal repository a run collects its changeset into. Pass the run's
/// staging directory to `promote_cell_changeset`.
pub fn staging_dir(run_dir: &Path) -> PathBuf {
    Layout::new(run_dir).staging
}

/// Perform an approved promotion. The approval is consumed only after the
/// request has been checked against the changeset, and the authoritative ref
/// moves only inside `promote`, by compare-and-swap.
///
/// Errors: `promotion_binding_mismatch` when the request does not describe its
/// own changeset or this repository (checked before the approval is consumed);
/// any authority error from `grant_after_approval`; and the changeset class
/// (for example `stale_base`) when the ref cannot be moved. Each promotion
/// attempt that reaches `promote` is recorded as `workspace_promotion`.
#[allow(clippy::too_many_arguments)]
pub fn promote_cell_changeset(
    policy_path: &Path,
    repository: &Path,
    staging: &Path,
    request: &PromotionRequest,
    approval: PromotionApproval<'_>,
    run_id: &str,
    plan_item_id: &str,
    stream: &LedgerStream,
) -> Result<Promotion, ForgeError> {
    let bundle = AuthorityPolicyBundle::load(policy_path)?;
    let engine = PolicyAuthorityEngine::new(bundle)?;
    let repository_path =
        fs::canonicalize(repository).map_err(|e| ForgeError::io("canonicalize repository", e))?;
    let changeset = &request.changeset;
    let mismatch = |message: &str| binding_mismatch(repository, message);

    // Pre-checks: the request must describe exactly its changeset and this repository.
    let AuthorityAction::PromoteChangeset {
        repository: bound_repository,
        reference,
        expected_base,
        proposed_commit,
        proposed_tree,
        changeset_digest,
        paths,
    } = &request.action
    else {
        return Err(mismatch("the request is not a promote_changeset action"));
    };
    let mut expected_paths: Vec<String> = changeset
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect();
    expected_paths.sort();
    let consistent = bound_repository == &repository_path.display().to_string()
        && reference == &changeset.base.reference
        && expected_base == &changeset.base.commit
        && proposed_commit == &changeset.proposed_commit
        && proposed_tree == &changeset.proposed_tree
        && changeset_digest == &changeset.digest
        && paths == &expected_paths
        && changeset.compute_digest() == changeset.digest;
    if !consistent {
        return Err(mismatch(
            "the promotion request does not match its changeset or repository",
        ));
    }

    // Only now is the approval consumed.
    let grant = engine.grant_after_approval(
        &request.decision,
        &request.committed_decision,
        &request.action,
        approval.approval,
        approval.committed_approval,
        approval.criteria,
        approval.committed_criteria,
        None,
        stream,
    )?;
    if request.decision.action_request.evidence["payload_hash"].as_str()
        != Some(grant.action_hash())
    {
        return Err(mismatch("the grant does not cover the requested action"));
    }

    // Everything the compare-and-swap needs comes from the approved action.
    let authorization = PromotionAuthorization {
        decision_id: request.decision.decision_id.clone(),
        reference: reference.clone(),
        expected_base: expected_base.clone(),
        changeset_digest: changeset_digest.clone(),
    };
    let refs = vec![
        request.decision.decision_id.clone(),
        approval.approval.approval_id.clone(),
    ];
    let subject = vec![run_id.into(), plan_item_id.into()];
    match promote(&repository_path, changeset, staging, &authorization) {
        Ok(promotion) => {
            stream.commit_typed("workspace_promotion", subject, &promotion, refs)?;
            Ok(promotion)
        }
        Err(error) => {
            let payload = json!({
                "error": {"class": error.class, "message": error.message.clone()},
                "changeset_digest": changeset.digest,
            });
            stream.commit_typed("workspace_promotion", subject, &payload, refs)?;
            Err(ForgeError::Config {
                class: error.class,
                path: repository.into(),
                message: error.message,
            })
        }
    }
}

/// A promotion-binding failure, reported against the repository.
fn binding_mismatch(repository: &Path, message: &str) -> ForgeError {
    ForgeError::Config {
        class: "promotion_binding_mismatch",
        path: repository.into(),
        message: message.into(),
    }
}

/// Directories and files one run owns, all under `run_dir`.
struct Layout {
    /// Private export handed to the sandbox.
    workspace: PathBuf,
    artifacts: PathBuf,
    /// What came back from the sandbox.
    result: PathBuf,
    /// Proposal repository.
    staging: PathBuf,
    changeset_json: PathBuf,
    evidence: PathBuf,
}

impl Layout {
    fn new(run_dir: &Path) -> Self {
        Self {
            workspace: run_dir.join("workspace"),
            artifacts: run_dir.join("artifacts"),
            result: run_dir.join("result"),
            staging: run_dir.join("staging.git"),
            changeset_json: run_dir.join("changeset.json"),
            evidence: run_dir.join("evidence.jsonl"),
        }
    }
}

/// The projection facts the episode needs, read before any authority call.
struct Projection {
    timeout_secs: u64,
    rules: ChangeRules,
}

fn read_projection(projection_dir: &Path) -> Result<Projection, ForgeError> {
    let bytes = fs::read(projection_dir.join("sandbox.json"))
        .map_err(|e| ForgeError::io("read openshell projection sandbox.json", e))?;
    let sandbox: Value = serde_json::from_slice(&bytes)?;
    let timeout_secs = sandbox
        .get("exec_timeout_seconds")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            ForgeError::Input("openshell projection lacks exec_timeout_seconds".into())
        })?;

    let mut protected_paths = Vec::new();
    for obligation in sandbox
        .get("obligations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if obligation.get("kind").and_then(Value::as_str) != Some("protected_paths") {
            continue;
        }
        let mutation = obligation
            .get("mutation")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ForgeError::Input("protected_paths obligation lacks a mutation".into())
            })?;
        let decision = match mutation {
            "allow" => continue,
            "escalate" => ProtectedDecision::Escalate,
            "deny" => ProtectedDecision::Deny,
            other => {
                return Err(ForgeError::Input(format!(
                    "unsupported protected_paths mutation `{other}`"
                )))
            }
        };
        let paths = obligation
            .get("paths")
            .and_then(Value::as_array)
            .ok_or_else(|| ForgeError::Input("protected_paths obligation lacks paths".into()))?;
        for path in paths {
            let path = path
                .as_str()
                .ok_or_else(|| ForgeError::Input("protected path is not a string".into()))?;
            protected_paths.push((path.to_string(), decision));
        }
    }
    Ok(Projection {
        timeout_secs,
        rules: ChangeRules { protected_paths },
    })
}

/// The loaded policy and engine, bound to one operator actor.
struct Authority<'a> {
    entity: &'a str,
    bundle: AuthorityPolicyBundle,
    engine: PolicyAuthorityEngine,
    actor: Actor,
}

/// Inputs one authority evaluation needs beyond the action itself.
struct Scope<'a> {
    case_id: &'a str,
    run_id: &'a str,
    plan_item_id: &'a str,
    sequence: usize,
    workspace: &'a Path,
    artifacts: &'a Path,
    timeout_secs: Option<u64>,
    env_keys: BTreeSet<String>,
}

impl<'a> Authority<'a> {
    fn open(policy_path: &Path, entity: &'a str) -> Result<Self, ForgeError> {
        let bundle = AuthorityPolicyBundle::load(policy_path)?;
        let engine = PolicyAuthorityEngine::new(bundle.clone())?;
        Ok(Self {
            entity,
            bundle,
            engine,
            actor: Actor {
                actor_id: entity.into(),
                role: ActorRole::Operator,
            },
        })
    }

    fn configures_opa(&self) -> bool {
        self.bundle
            .policy_engines
            .iter()
            .any(|engine| engine.engine == "opa")
    }

    fn evaluate(
        &self,
        scope: Scope<'_>,
        action: &AuthorityAction,
        opa_candidate: Option<&OpaCandidate>,
    ) -> Result<AuthorityDecision, ForgeError> {
        let candidate = self
            .bundle
            .load_domainforge_model()?
            .map(|model| {
                DomainForgeCandidate::evaluate(
                    &model,
                    action,
                    vec![format!(
                        "domain-model:{}",
                        model.model_ref.semantic_model_sha256
                    )],
                )
            })
            .transpose()?;
        self.engine.evaluate(AuthorityEvaluation {
            actor: &self.actor,
            binding: self
                .bundle
                .resolve_identity(self.entity, ActorRole::Operator),
            run_id: scope.run_id,
            case_id: scope.case_id,
            plan_item_id: scope.plan_item_id,
            sequence: scope.sequence,
            action,
            workspace_root: scope.workspace,
            evidence_refs: vec![],
            artifacts_root: Some(scope.artifacts),
            timeout_secs: scope.timeout_secs,
            env_keys: scope.env_keys,
            domainforge_candidate: candidate.as_ref(),
            opa_candidate,
            environment: None,
        })
    }
}

/// Map a changeset failure that must abort the episode onto a typed error.
fn config_error(error: ChangesetError, repository: &Path) -> ForgeError {
    ForgeError::Config {
        class: error.class,
        path: repository.into(),
        message: error.message,
    }
}

/// Commit the execution's sandbox evidence and result as one ledger record.
fn record_execution(
    stream: &LedgerStream,
    layout: &Layout,
    run_id: &str,
    plan_item_id: &str,
    decision_id: &str,
    execution: &ExecutionResult,
) -> Result<CommittedRecordRef, ForgeError> {
    let openshell = layout.artifacts.join("openshell");
    let sandbox_text = fs::read_to_string(openshell.join("execution.json"))
        .map_err(|e| ForgeError::io("read openshell execution.json", e))?;
    let sandbox: Value = serde_json::from_str(&sandbox_text)?;
    let denials: Vec<String> = match fs::read_to_string(openshell.join("denials.log")) {
        Ok(text) => text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(str::to_owned)
            .collect(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(ForgeError::io("read openshell denials.log", error)),
    };
    stream.commit_typed(
        "openshell_execution",
        vec![run_id.into(), plan_item_id.into()],
        &json!({"sandbox": sandbox, "result": execution, "denials": denials}),
        vec![decision_id.into()],
    )
}

/// Record the execution files that exist as evidence, one line each.
fn capture_evidence(
    layout: &Layout,
    run_id: &str,
    source_event_id: &str,
) -> Result<(), ForgeError> {
    let openshell = layout.artifacts.join("openshell");
    let mut writer = JsonlEvidenceWriter::create(&layout.evidence, run_id)?;
    for name in ["execution.json", "effective-policy.yaml", "denials.log"] {
        if openshell.join(name).is_file() {
            capture_file(
                &openshell.join(name),
                &openshell,
                name,
                &mut writer,
                source_event_id.to_string(),
                None,
            )?;
        }
    }
    Ok(())
}

/// The outcome of reviewing the returned workspace.
struct Review {
    changeset: Option<Changeset>,
    validation: Option<Validation>,
}

/// Collect and validate the returned tree, and record the result. A malformed
/// workspace is recorded as an error and yields no changeset.
fn review_workspace(
    repository: &Path,
    base: &BaseIdentity,
    layout: &Layout,
    committed_at: &str,
    rules: &ChangeRules,
    stream: &LedgerStream,
) -> Result<Review, ForgeError> {
    let (payload, review) = match collect(
        repository,
        base,
        &layout.result,
        &layout.staging,
        committed_at,
    ) {
        Ok((changeset, findings)) => {
            let validation = validate(&changeset, rules, &findings);
            let payload = json!({"changeset": changeset, "validation": validation});
            let review = Review {
                changeset: Some(changeset),
                validation: Some(validation),
            };
            (payload, review)
        }
        Err(error) if error.class == "malformed_workspace" => {
            let payload = json!({"error": {"class": error.class, "message": error.message}});
            (
                payload,
                Review {
                    changeset: None,
                    validation: None,
                },
            )
        }
        Err(error) => return Err(config_error(error, repository)),
    };
    crate::write_json(&layout.changeset_json, &payload)?;
    stream.commit_typed("workspace_changeset", vec![], &payload, vec![])?;
    Ok(review)
}

/// Map validation onto the settlement verdict. A missing validation (a
/// malformed workspace) is a denial.
fn validation_verdict(validation: &Option<Validation>) -> Verdict {
    match validation {
        Some(Validation::Accept) => Verdict::Allow,
        Some(Validation::Escalate(_)) => Verdict::Escalate,
        Some(Validation::Reject(_)) | None => Verdict::Deny,
    }
}

/// Settle the episode with the existing settlement path and record it.
#[allow(clippy::too_many_arguments)]
fn settle_episode(
    run_id: &str,
    plan_item_id: &str,
    criteria: SettlementCriteria,
    execution: Option<ExecutionResult>,
    authority_verdicts: Vec<Verdict>,
    workspace: &Path,
    run_dir: &Path,
    stream: &LedgerStream,
) -> Result<SettlementEvent, ForgeError> {
    let settlement = sea_forge_settlement::settle(
        &SettlementClaim {
            run_id: run_id.into(),
            plan_item_id: plan_item_id.into(),
            criteria_ref: None,
            criteria,
            execution,
            authority_verdicts,
            evaluator_scores: BTreeMap::new(),
            batch: None,
            write_only: false,
        },
        workspace,
        run_dir,
    )?;
    stream.commit_typed(
        "settlement_event",
        vec![
            run_id.into(),
            plan_item_id.into(),
            settlement.settlement_id.clone(),
        ],
        &settlement,
        vec![],
    )?;
    Ok(settlement)
}

/// Everything promotion needs that the episode already holds.
struct PromotionContext<'a> {
    case_id: &'a str,
    run_id: &'a str,
    plan_item_id: &'a str,
    reference: &'a str,
    repository: &'a Path,
    layout: &'a Layout,
    base: &'a BaseIdentity,
    stream: &'a LedgerStream,
}

/// Ask for a promotion when settlement accepted the work, validation accepted
/// a non-empty changeset, and `promote_changeset` escalates for approval. The
/// authoritative ref is never touched here.
fn request_promotion(
    authority: &Authority<'_>,
    context: PromotionContext<'_>,
    settlement: &SettlementEvent,
    changeset: Option<&Changeset>,
    validation: &Option<Validation>,
    // The model-level OPA evaluation already recorded for this episode. It can
    // only deny or escalate the request; the promotion rule still escalates.
    opa_candidate: Option<&OpaCandidate>,
) -> Result<Option<PromotionRequest>, ForgeError> {
    let accepted = settlement.status == SettlementStatus::Accepted
        && matches!(validation, Some(Validation::Accept));
    let Some(changeset) = changeset.filter(|changeset| accepted && !changeset.entries.is_empty())
    else {
        return Ok(None);
    };

    let repository = fs::canonicalize(context.repository)
        .map_err(|e| ForgeError::io("canonicalize repository", e))?;
    let mut paths: Vec<String> = changeset
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect();
    paths.sort();
    let action = AuthorityAction::PromoteChangeset {
        repository: repository.display().to_string(),
        reference: context.reference.into(),
        expected_base: context.base.commit.clone(),
        proposed_commit: changeset.proposed_commit.clone(),
        proposed_tree: changeset.proposed_tree.clone(),
        changeset_digest: changeset.digest.clone(),
        paths,
    };
    let decision = authority.evaluate(
        Scope {
            case_id: context.case_id,
            run_id: context.run_id,
            plan_item_id: context.plan_item_id,
            sequence: 2,
            workspace: &context.layout.workspace,
            artifacts: &context.layout.artifacts,
            timeout_secs: None,
            env_keys: BTreeSet::new(),
        },
        &action,
        opa_candidate,
    )?;
    let committed_decision = context.stream.commit_typed(
        "authority_decision",
        vec![context.run_id.into(), context.plan_item_id.into()],
        &decision,
        vec![],
    )?;

    if decision.verdict == Verdict::Allow {
        return Err(ForgeError::Internal(
            "promote_changeset must escalate".into(),
        ));
    }
    let escalated = decision.verdict == Verdict::Escalate && decision.matched_rule.is_some();
    if !escalated {
        return Ok(None);
    }
    Ok(Some(PromotionRequest {
        action,
        decision,
        committed_decision,
        changeset: changeset.clone(),
    }))
}
