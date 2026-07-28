#![forbid(unsafe_code)]

use chrono::Utc;
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::{errors::ForgeError, ids, types::*, RECORD_VERSION};
use sea_forge_ledger::LedgerStream;
use sea_forge_planner::case_engine::{next_case_actions, replay_case, CaseAction};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Synchronous case lifecycle operations shared by the CLI and server.
pub struct CaseRunner;

impl CaseRunner {
    pub fn initialize_case(case_dir: &Path) -> Result<(PathBuf, PathBuf), ForgeError> {
        let runs_dir = case_dir.join("runs");
        fs::create_dir_all(&runs_dir).map_err(|error| ForgeError::io("create case runs", error))?;
        Ok((runs_dir, case_dir.join("case-events.jsonl")))
    }

    pub fn next_ready_actions(items: &[PlanItem], events: &[TraceEvent]) -> Vec<CaseAction> {
        next_case_actions(items, events)
    }

    pub fn append_event(
        path: &Path,
        stream: &LedgerStream,
        events: &mut Vec<TraceEvent>,
        kind: TraceKind,
        item_id: Option<&str>,
        mut payload: serde_json::Value,
    ) -> Result<(), ForgeError> {
        // Additive persisted ordinals (spec §17.2 T13.2): each dispatch/
        // activation and settlement gets a 1-based monotonic ordinal derived
        // from prior persisted events. Additive only — old readers ignore it.
        // ponytail: O(n) scan per append; switch to a runner-held counter if
        // a case ever accumulates thousands of dispatch/settlement events.
        match kind {
            TraceKind::ItemActivated => {
                let next = Self::next_ordinal(events, TraceKind::ItemActivated, "dispatch_ordinal");
                payload["dispatch_ordinal"] = serde_json::json!(next);
            }
            TraceKind::SettlementRecorded => {
                let next =
                    Self::next_ordinal(events, TraceKind::SettlementRecorded, "settlement_ordinal");
                payload["settlement_ordinal"] = serde_json::json!(next);
            }
            _ => {}
        }
        Self::commit_event(
            path,
            stream,
            events,
            TraceEvent {
                version: RECORD_VERSION.into(),
                event_id: ids::seq_id("cev", 6, events.len() + 1),
                run_id: "case".into(),
                plan_item_id: item_id.map(str::to_owned),
                kind,
                actor_id: "case_engine".into(),
                timestamp: Utc::now().to_rfc3339(),
                payload,
                cell_id: None,
            },
        )
    }

    fn next_ordinal(events: &[TraceEvent], kind: TraceKind, key: &str) -> u64 {
        events
            .iter()
            .filter(|event| event.kind == kind)
            .filter_map(|event| event.payload.get(key).and_then(|value| value.as_u64()))
            .max()
            .unwrap_or(0)
            + 1
    }

    pub fn commit_event(
        path: &Path,
        stream: &LedgerStream,
        events: &mut Vec<TraceEvent>,
        event: TraceEvent,
    ) -> Result<(), ForgeError> {
        stream.commit_typed("case_event", vec![], &event, vec![])?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| ForgeError::io("open case events", error))?;
        serde_json::to_writer(&mut file, &event)?;
        file.write_all(b"\n")
            .and_then(|_| file.flush())
            .map_err(|error| ForgeError::io("append case event", error))?;
        events.push(event);
        Ok(())
    }

    pub fn run_sandboxed_episode<T>(
        execute: impl FnOnce() -> Result<T, ForgeError>,
    ) -> Result<T, ForgeError> {
        execute()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply_episode_completion(
        case: &mut Case,
        run_id: &str,
        item_id: &str,
        instance: u32,
        settlement: &SettlementEvent,
        case_events: &Path,
        stream: &LedgerStream,
        events: &mut Vec<TraceEvent>,
    ) -> Result<bool, ForgeError> {
        if !case.run_ids.iter().any(|existing| existing == run_id) {
            case.run_ids.push(run_id.into());
        }
        let accepted = settlement.status == SettlementStatus::Accepted;
        Self::append_event(
            case_events,
            stream,
            events,
            if accepted {
                TraceKind::ItemCompleted
            } else {
                TraceKind::ItemFailed
            },
            Some(item_id),
            json!({"instance": instance, "settlement": settlement.status}),
        )?;
        if accepted {
            Self::append_event(
                case_events,
                stream,
                events,
                TraceKind::MilestoneAchieved,
                Some(item_id),
                json!({"instance": instance}),
            )?;
        }
        Ok(accepted)
    }
}

/// Outcome of driving a stage `CasePlan` to completion or a blocking point
/// (spec-audit-remediation Task 10A).
pub struct StageCaseOutcome {
    pub case: Case,
    pub stages: Vec<SpecPipelineStage>,
    pub events: Vec<TraceEvent>,
    /// `"completed"`, `"terminated"`, or `"active"` (blocked/no ready action).
    pub state: &'static str,
}

/// Run a single stage episode: validate Task 9's prerequisite chain, deny
/// generated-zone edits before authority (§10.7, `generated_zone_direct_edit`),
/// then evaluate authority, execute under the granted sandbox, and settle —
/// mirroring the ordinary sandboxed-task episode pattern used elsewhere in
/// the workspace (no second authority/execution path for spec-pipeline
/// stages). Updates `stages[stage_index].status` in place and quarantines it
/// (via `sea_forge_spec_pipeline::quarantine_stage`) on any denial.
#[allow(clippy::too_many_arguments)]
fn run_stage_episode(
    stages: &mut [SpecPipelineStage],
    stage_index: usize,
    item: &PlanItem,
    policy_path: &Path,
    entity: &str,
    case_id: &str,
    run_id: &str,
    root: &Path,
    stream: &LedgerStream,
) -> Result<SettlementEvent, ForgeError> {
    if let Err(error) = sea_forge_spec_pipeline::validate_stage_prerequisites(stages, stage_index) {
        let reason = error.to_string();
        sea_forge_spec_pipeline::quarantine_stage(&mut stages[stage_index], &reason)?;
        return Ok(SettlementEvent {
            version: RECORD_VERSION.into(),
            settlement_id: ids::random_id("set")?,
            run_id: run_id.into(),
            status: SettlementStatus::Rejected,
            basis: vec!["prerequisite_invalid".into()],
            review_required: false,
            settled_at: Utc::now().to_rfc3339(),
            criteria_ref: item.settlement_criteria_ref.clone(),
        });
    }

    // Generated-zone outputs (src/gen, AST, IR, manifests, generated semantic
    // fixtures) MUST be read-only to operators (§10.7). Only the pipeline's
    // own generator-kind stages may legitimately produce them; any other
    // stage kind declaring a generated-zone output is a denied
    // `run_spec_pipeline` operation, basis `generated_zone_direct_edit`,
    // rejected before authority is ever consulted.
    const GENERATOR_KINDS: &[StageKind] = &[
        StageKind::Ast,
        StageKind::Ir,
        StageKind::Manifest,
        StageKind::GeneratedContract,
        StageKind::SemanticFixture,
    ];
    if !GENERATOR_KINDS.contains(&stages[stage_index].kind) {
        let direct_edit = stages[stage_index]
            .outputs
            .iter()
            .any(|output| sea_forge_spec_pipeline::is_generated_zone(&output.path));
        if direct_edit {
            sea_forge_spec_pipeline::quarantine_stage(
                &mut stages[stage_index],
                "generated_zone_direct_edit",
            )?;
            return Ok(SettlementEvent {
                version: RECORD_VERSION.into(),
                settlement_id: ids::random_id("set")?,
                run_id: run_id.into(),
                status: SettlementStatus::Rejected,
                basis: vec!["generated_zone_direct_edit".into()],
                review_required: false,
                settled_at: Utc::now().to_rfc3339(),
                criteria_ref: item.settlement_criteria_ref.clone(),
            });
        }
    }

    let operation = item
        .operations
        .first()
        .ok_or_else(|| ForgeError::Input("stage episode missing operation".into()))?
        .clone();
    let workspace = root
        .join("cases")
        .join(case_id)
        .join("runs")
        .join(run_id)
        .join("workspace");
    let run_dir = workspace.parent().unwrap().to_path_buf();
    let artifacts = run_dir.join("artifacts");
    // AUTH-01: created below, inside the allow branch. A denied stage must
    // leave nothing behind on the filesystem.

    let bundle = AuthorityPolicyBundle::load(policy_path)?;
    let engine = PolicyAuthorityEngine::new(bundle.clone())?;
    let actor = Actor {
        actor_id: entity.into(),
        role: ActorRole::Operator,
    };
    let action = AuthorityAction::from(&operation);
    let decision = engine.evaluate(AuthorityEvaluation {
        actor: &actor,
        binding: bundle.resolve_identity(entity, ActorRole::Operator),
        run_id,
        case_id,
        plan_item_id: &item.plan_item_id,
        sequence: 1,
        action: &action,
        workspace_root: &workspace,
        evidence_refs: vec![],
        artifacts_root: Some(&artifacts),
        timeout_secs: Some(600),
        env_keys: ["PATH", "HOME"].into_iter().map(str::to_owned).collect(),
        domainforge_candidate: None,
        environment: None,
    })?;
    let decision_ref = stream.commit_typed(
        "authority_decision",
        vec![run_id.into(), item.plan_item_id.clone()],
        &decision,
        vec![],
    )?;
    let execution = if decision.verdict == Verdict::Allow {
        // The Allow is committed (`decision_ref`) before anything is written.
        fs::create_dir_all(&workspace).map_err(|e| ForgeError::io("create workspace", e))?;
        fs::create_dir_all(&artifacts).map_err(|e| ForgeError::io("create artifacts", e))?;
        let grant = engine.grant(&decision, &decision_ref, &action, None)?;
        Some(sea_forge_runtime::execute(
            grant,
            &ExecutionRequest {
                plan_item_id: item.plan_item_id.clone(),
                operation,
                timeout_secs: 600,
                env: BTreeMap::from([
                    (
                        String::from("PATH"),
                        std::env::var("PATH").unwrap_or_default(),
                    ),
                    (
                        String::from("HOME"),
                        std::env::var("HOME").unwrap_or_default(),
                    ),
                ]),
                compensating_controls: decision.compensating_controls.clone(),
            },
            run_id,
            &workspace,
            &artifacts,
        )?)
    } else {
        // Not a spawn that failed — no spawn was ever attempted.
        None
    };

    // Settlement evaluates the stage's declared criteria against the evidence
    // the stage produced, rather than trusting its exit code. It is also the
    // only path that represents `Escalate`, which the previous `accepted`
    // boolean collapsed into a plain rejection — discarding the review the
    // verdict asked for.
    let settlement = sea_forge_settlement::settle(
        &sea_forge_core::types::SettlementClaim {
            run_id: run_id.into(),
            plan_item_id: item.plan_item_id.clone(),
            criteria_ref: item.settlement_criteria_ref.clone(),
            criteria: item.settlement_criteria.clone(),
            execution,
            authority_verdicts: vec![decision.verdict.clone()],
            evaluator_scores: BTreeMap::new(),
            batch: None,
        },
        &workspace,
        &run_dir,
    )?;

    stages[stage_index].status = match settlement.status {
        SettlementStatus::Accepted => StageStatus::Accepted,
        _ => StageStatus::Rejected,
    };
    if settlement.status != SettlementStatus::Accepted {
        // `basis` already carries why (authority_deny / spawn_failed /
        // exit_nonzero / required_artifact_missing / ...), so the quarantine
        // reason quotes it instead of re-deriving a coarser one.
        sea_forge_spec_pipeline::quarantine_stage(
            &mut stages[stage_index],
            &settlement.basis.join(","),
        )?;
    }
    Ok(settlement)
}

/// Drive a stage `CasePlan` (from `sea_forge_planner::stage_case_plan`) to
/// completion as an ordinary governed case: every stage activation, quarantine,
/// and settlement is an existing case/authority/ledger record, not a separate
/// spec-pipeline scheduler (Task 10A). `stages` must be in the same order and
/// carry the same `stage_id`s as `plan.items[*].plan_item_id`.
pub fn run_stage_case(
    root: &Path,
    policy_path: &Path,
    entity: &str,
    plan: &CasePlan,
    mut stages: Vec<SpecPipelineStage>,
) -> Result<StageCaseOutcome, ForgeError> {
    let case_id = plan.case_id.clone();
    let case_dir = root.join("cases").join(&case_id);
    let (_runs_dir, case_events) = CaseRunner::initialize_case(&case_dir)?;
    let stream = LedgerStream::open(root, format!("case-{case_id}"), entity)?;
    stream.commit_typed("case_plan", vec![case_id.clone()], plan, vec![])?;

    let mut case = Case {
        version: RECORD_VERSION.into(),
        case_id: case_id.clone(),
        intent: Intent {
            intent_id: plan.intent_id.clone(),
            summary: format!("Execute spec pipeline for case {case_id}"),
            actor_id: entity.into(),
            process_id: "spec_pipeline".into(),
            created_at: Utc::now().to_rfc3339(),
        },
        state: CaseState::Active,
        plan_ref: plan.plan_id.clone(),
        run_ids: vec![],
        stages: plan
            .items
            .iter()
            .map(|item| item.plan_item_id.clone())
            .collect(),
        close_reason: None,
        created_at: Utc::now().to_rfc3339(),
        closed_at: None,
    };
    write_json(&case_dir.join("case.json"), &case)?;
    write_json(&case_dir.join("plan.json"), plan)?;

    let mut events = vec![];
    CaseRunner::append_event(
        &case_events,
        &stream,
        &mut events,
        TraceKind::CaseCreated,
        None,
        json!({"case_id": case_id}),
    )?;

    let state = loop {
        let actions = CaseRunner::next_ready_actions(&plan.items, &events);
        if actions.is_empty() {
            break "active";
        }
        let mut terminal = None;
        for action in actions {
            match action {
                CaseAction::Enable(item_id) => {
                    CaseRunner::append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::ItemEnabled,
                        Some(&item_id),
                        json!({}),
                    )?;
                }
                CaseAction::CompleteCase => {
                    case.state = CaseState::Completed;
                    case.closed_at = Some(Utc::now().to_rfc3339());
                    CaseRunner::append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::CaseClosed,
                        None,
                        json!({}),
                    )?;
                    terminal = Some("completed");
                }
                CaseAction::TerminateCase { blocking_item } => {
                    case.state = CaseState::Terminated;
                    case.close_reason = Some(format!("required_item_failed:{blocking_item}"));
                    case.closed_at = Some(Utc::now().to_rfc3339());
                    CaseRunner::append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::CaseTerminated,
                        None,
                        json!({"blocking_item": blocking_item}),
                    )?;
                    terminal = Some("terminated");
                }
                CaseAction::AchieveMilestone(item_id) => {
                    CaseRunner::append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::MilestoneAchieved,
                        Some(&item_id),
                        json!({}),
                    )?;
                }
                CaseAction::ParkHumanTask(item_id) => {
                    return Err(ForgeError::Input(format!(
                        "spec_pipeline_error: stage {item_id} cannot be a human task"
                    )));
                }
                CaseAction::Activate(item_id) => {
                    let item = plan
                        .items
                        .iter()
                        .find(|item| item.plan_item_id == item_id)
                        .ok_or_else(|| ForgeError::Internal("missing plan item".into()))?
                        .clone();
                    let stage_index = stages
                        .iter()
                        .position(|stage| stage.stage_id == item_id)
                        .ok_or_else(|| {
                            ForgeError::Internal(format!("missing stage for item {item_id}"))
                        })?;
                    let projection = replay_case(&plan.items, &events);
                    let instance = projection
                        .items
                        .iter()
                        .find(|state| state.item_id == item_id)
                        .map_or(1, |state| state.instances + 1);
                    let episode_run_id = ids::run_id()?;
                    CaseRunner::append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::ItemActivated,
                        Some(&item_id),
                        json!({"instance": instance, "run_id": episode_run_id}),
                    )?;
                    let settlement = run_stage_episode(
                        &mut stages,
                        stage_index,
                        &item,
                        policy_path,
                        entity,
                        &case_id,
                        &episode_run_id,
                        root,
                        &stream,
                    )?;
                    stream.commit_typed(
                        "settlement_event",
                        vec![
                            case_id.clone(),
                            episode_run_id.clone(),
                            settlement.settlement_id.clone(),
                        ],
                        &settlement,
                        vec![],
                    )?;
                    CaseRunner::append_event(
                        &case_events,
                        &stream,
                        &mut events,
                        TraceKind::SettlementRecorded,
                        Some(&item_id),
                        json!({
                            "instance": instance,
                            "run_id": episode_run_id,
                            "status": settlement.status,
                        }),
                    )?;
                    CaseRunner::apply_episode_completion(
                        &mut case,
                        &episode_run_id,
                        &item_id,
                        instance,
                        &settlement,
                        &case_events,
                        &stream,
                        &mut events,
                    )?;
                }
            }
        }
        write_json(&case_dir.join("case.json"), &case)?;
        if let Some(terminal) = terminal {
            break terminal;
        }
    };

    Ok(StageCaseOutcome {
        case,
        stages,
        events,
        state,
    })
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ForgeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| ForgeError::io("create JSON parent", error))?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .map_err(|error| ForgeError::io("write JSON", error))
}
