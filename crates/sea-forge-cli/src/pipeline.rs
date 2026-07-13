use chrono::Utc;
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_capability as capability;
use sea_forge_core::ids;
use sea_forge_core::{errors::ForgeError, types::*, RECORD_VERSION};
use sea_forge_evidence::{capture_file, JsonlEvidenceWriter};
use sea_forge_ledger::LedgerStream;
use sea_forge_planner as planner;
use sea_forge_runtime as runtime;
use sea_forge_sandbox as sandbox;
use sea_forge_settlement as settlement;
use sea_forge_trace::JsonlTraceRecorder;
use serde::Serialize;
use serde_json::json;
use std::{
    collections::BTreeMap,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct RunOptions {
    pub intent: String,
    pub policy: PathBuf,
    pub root: PathBuf,
    pub timeout_secs: u64,
    pub entity: String,
    pub process: String,
}
#[derive(Clone, Debug)]
pub struct RunOutcome {
    pub run_id: String,
    #[allow(dead_code)]
    pub case_id: String,
    pub run_dir: PathBuf,
    pub decisions: Vec<AuthorityDecision>,
    pub execution: Option<ExecutionResult>,
    pub settlement: SettlementEvent,
}
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ForgeError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).map_err(|e| ForgeError::io(format!("write {}", path.display()), e))
}
fn write_json_new<T: Serialize>(path: &Path, value: &T) -> Result<(), ForgeError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| ForgeError::io(format!("create {}", path.display()), error))?;
    file.write_all(&bytes)
        .and_then(|_| file.flush())
        .map_err(|error| ForgeError::io(format!("write {}", path.display()), error))
}
fn replace_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ForgeError> {
    let temporary = path.with_extension("json.tmp");
    write_json(&temporary, value)?;
    fs::rename(&temporary, path)
        .map_err(|error| ForgeError::io(format!("replace {}", path.display()), error))
}
fn validate_attribution(value: &str, name: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    {
        Err(ForgeError::Input(format!("invalid {name}")))
    } else {
        Ok(())
    }
}
pub fn run_intent(options: RunOptions) -> Result<RunOutcome, ForgeError> {
    let bundle = AuthorityPolicyBundle::load(&options.policy)?;
    sea_forge_domain::interpret(&options.intent)?;
    validate_attribution(&options.entity, "entity")?;
    validate_attribution(&options.process, "process")?;
    fs::create_dir_all(&options.root).map_err(|e| ForgeError::io("preflight root", e))?;
    let root = options
        .root
        .canonicalize()
        .map_err(|e| ForgeError::io("canonicalize root", e))?;
    let runs_root = ensure_state_directory(&root, "runs")?;
    let cases_root = ensure_state_directory(&root, "cases")?;
    let run_id = ids::run_id()?;
    let case_id = ids::case_id()?;
    let run_dir = runs_root.join(&run_id);
    fs::create_dir(&run_dir).map_err(|e| ForgeError::io("create run directory", e))?;
    let actor_for_error = options.entity.clone();
    let result = (|| -> Result<RunOutcome, ForgeError> {
        let workspace = run_dir.join("workspace");
        let artifacts = run_dir.join("artifacts");
        fs::create_dir(&workspace).map_err(|e| ForgeError::io("create workspace structure", e))?;
        fs::create_dir(&artifacts).map_err(|e| ForgeError::io("create artifacts", e))?;
        let now = Utc::now().to_rfc3339();
        let intent = Intent {
            intent_id: ids::random_id("int")?,
            summary: options.intent,
            actor_id: options.entity.clone(),
            process_id: options.process.clone(),
            created_at: now.clone(),
        };
        let executable =
            env::current_exe().map_err(|e| ForgeError::io("resolve current executable", e))?;
        let executable = executable.to_string_lossy().into_owned();
        let plan = planner::plan(&intent, &case_id, &run_id, &executable)?;
        let item = plan
            .items
            .first()
            .ok_or_else(|| ForgeError::Internal("planner returned an empty plan".into()))?;
        let mut case = Case {
            version: RECORD_VERSION.into(),
            case_id: case_id.clone(),
            intent: intent.clone(),
            state: CaseState::Active,
            plan_ref: plan.plan_id.clone(),
            run_ids: vec![run_id.clone()],
            close_reason: None,
            created_at: now,
            closed_at: None,
        };
        let case_path = cases_root.join(format!("{case_id}.json"));
        write_json_new(&case_path, &case)?;
        let mut trace =
            JsonlTraceRecorder::create(&run_dir.join("trace.jsonl"), &run_id, &intent.actor_id)?;
        trace.append(TraceKind::CaseCreated, None, json!({"case_id":case_id}))?;
        trace.append(TraceKind::RunStarted, None, json!({}))?;
        write_json(&run_dir.join("plan.json"), &plan)?;
        trace.append(
            TraceKind::PlanCreated,
            Some(item.plan_item_id.clone()),
            json!({"plan_id":plan.plan_id}),
        )?;
        let mut evidence = JsonlEvidenceWriter::create(&run_dir.join("evidence.jsonl"), &run_id)?;
        let identity_source = bundle.identity.source.clone();
        let engine = PolicyAuthorityEngine::new(bundle)?;
        let actor = Actor {
            actor_id: intent.actor_id.clone(),
            role: ActorRole::Operator,
        };
        let binding = IdentityBinding {
            principal: actor.actor_id.clone(),
            actor_type: ActorType::Human,
            binding_resolution: BindingResolution::LocalDefault,
            identity_binding_source: identity_source,
            sponsor: None,
        };
        let authority_stream =
            LedgerStream::open(&root, format!("case-{case_id}"), &intent.actor_id)?;
        let mut decisions = Vec::new();
        for (index, operation) in item.operations.iter().enumerate() {
            let action = AuthorityAction::from(operation);
            let mut decision = engine.evaluate(AuthorityEvaluation {
                actor: &actor,
                binding: binding.clone(),
                run_id: &run_id,
                plan_item_id: &item.plan_item_id,
                sequence: index + 1,
                action: &action,
                workspace_root: &workspace,
            })?;
            let event = trace.append(
                TraceKind::AuthorityEvaluated,
                Some(item.plan_item_id.clone()),
                json!({"decision_id":decision.decision_id,"verdict":decision.verdict}),
            )?;
            let evidence_record = evidence.append(
                EvidenceKind::AuthorityDecision,
                decision.decision_id.clone(),
                None,
                event,
                BTreeMap::new(),
            )?;
            authority_stream.commit_typed(
                "authority_evidence",
                vec![run_id.clone(), decision.decision_id.clone()],
                &evidence_record,
                vec![],
            )?;
            decision
                .audit_record
                .evidence_refs
                .push(evidence_record.evidence_id);
            decisions.push(decision)
        }
        let committed_decisions = decisions
            .iter()
            .map(|decision| {
                authority_stream.commit_typed(
                    "authority_decision",
                    vec![run_id.clone(), item.plan_item_id.clone()],
                    decision,
                    vec![],
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let authority_bytes = serde_json::to_vec_pretty(&decisions)?;
        authority_stream.materialize_view(
            committed_decisions
                .last()
                .ok_or_else(|| ForgeError::Internal("no authority decisions committed".into()))?,
            &run_dir.join("authority.json"),
            &authority_bytes,
        )?;
        let all_allow = decisions.iter().all(|d| d.verdict == Verdict::Allow);
        let mut execution = None;
        let mut artifact_refs = Vec::new();
        if !all_allow {
            trace.append(
                TraceKind::RunHalted,
                Some(item.plan_item_id.clone()),
                json!({}),
            )?;
        } else {
            trace.append(TraceKind::WorkspaceCreated, None, json!({}))?;
            for (index, operation) in item.operations.iter().enumerate() {
                if matches!(operation, Operation::WriteFile { .. }) {
                    let grant = engine.grant(
                        &decisions[index],
                        &committed_decisions[index],
                        &AuthorityAction::from(operation),
                        &workspace,
                    )?;
                    sandbox::materialize(
                        grant,
                        &workspace,
                        &run_id,
                        &item.plan_item_id,
                        operation,
                    )?;
                }
            }
            if let Some((index, operation)) = item
                .operations
                .iter()
                .enumerate()
                .find(|(_, operation)| matches!(operation, Operation::ExecuteCommand { .. }))
            {
                let start = trace.append(
                    TraceKind::CommandStarted,
                    Some(item.plan_item_id.clone()),
                    json!({}),
                )?;
                let mut envs = BTreeMap::new();
                if let Ok(path) = env::var("PATH") {
                    envs.insert("PATH".into(), path);
                }
                if let Ok(home) = env::var("HOME") {
                    envs.insert("HOME".into(), home);
                }
                let grant = engine.grant(
                    &decisions[index],
                    &committed_decisions[index],
                    &AuthorityAction::from(operation),
                    &workspace,
                )?;
                let result = runtime::execute(
                    grant,
                    &ExecutionRequest {
                        plan_item_id: item.plan_item_id.clone(),
                        operation: operation.clone(),
                        timeout_secs: options.timeout_secs,
                        env: envs,
                    },
                    &run_id,
                    &workspace,
                    &artifacts,
                )?;
                let finished = trace.append(
                    TraceKind::CommandFinished,
                    Some(item.plan_item_id.clone()),
                    json!({"execution":result,"started_event":start}),
                )?;
                let mut execution_metadata = BTreeMap::new();
                execution_metadata
                    .insert("execution_result".into(), serde_json::to_value(&result)?);
                evidence.append(
                    EvidenceKind::ExecutionResult,
                    finished.clone(),
                    None,
                    finished.clone(),
                    execution_metadata,
                )?;
                for name in ["stdout.txt", "stderr.txt"] {
                    let event = trace.append(
                        TraceKind::ArtifactCaptured,
                        Some(item.plan_item_id.clone()),
                        json!({"artifact":name}),
                    )?;
                    capture_file(
                        &artifacts.join(name),
                        &artifacts,
                        name,
                        &mut evidence,
                        event,
                        None,
                    )?;
                }
                if let Ok(model) = sandbox::safe_existing(&workspace, "model.sea") {
                    let event = trace.append(
                        TraceKind::ArtifactCaptured,
                        Some(item.plan_item_id.clone()),
                        json!({"artifact":"model.sea"}),
                    )?;
                    let (record, descriptor) = capture_file(
                        &model,
                        &artifacts,
                        "model.sea",
                        &mut evidence,
                        event,
                        Some((&intent, &item.plan_item_id)),
                    )?;
                    let d = descriptor.ok_or_else(|| {
                        ForgeError::Internal("work-product descriptor was not produced".into())
                    })?;
                    artifact_refs.push(ArtifactRef {
                        evidence_id: record.evidence_id,
                        artifact_id: d.artifact_id,
                        pre_mint_identity: d.pre_mint_identity,
                    });
                }
                execution = Some(result);
            }
        }
        let claim = SettlementClaim {
            run_id: run_id.clone(),
            plan_item_id: item.plan_item_id.clone(),
            criteria: item.settlement_criteria.clone(),
            execution: execution.clone(),
            authority_verdicts: decisions.iter().map(|d| d.verdict.clone()).collect(),
        };
        let settled = settlement::settle(&claim, &workspace, &run_dir)?;
        write_json(&run_dir.join("settlement.json"), &settled)?;
        trace.append(
            TraceKind::SettlementRecorded,
            Some(item.plan_item_id.clone()),
            json!({"settlement_id":settled.settlement_id,"status":settled.status}),
        )?;
        let envelope = SemanticEnvelope {
            version: RECORD_VERSION.into(),
            run_id: run_id.clone(),
            case_ref: case_id.clone(),
            intent: intent.clone(),
            plan_ref: plan.plan_id.clone(),
            authority_decisions: decisions.iter().map(|d| d.decision_id.clone()).collect(),
            evidence_refs: evidence.refs(),
            settlement_ref: settled.settlement_id.clone(),
            capability_delta: CapabilityDelta {
                attempted_capability: item.name.clone(),
                result: settled.status.clone(),
            },
            attribution: Attribution {
                entity_id: intent.actor_id.clone(),
                process_id: intent.process_id.clone(),
                session_id: case_id.clone(),
            },
            artifact_refs,
            extension_refs: vec![],
            projection_refs: vec![],
        };
        write_json(&run_dir.join("semantic-envelope.json"), &envelope)?;
        trace.append(
            TraceKind::RunFinished,
            None,
            json!({"status":settled.status}),
        )?;
        case.closed_at = Some(Utc::now().to_rfc3339());
        match settled.status {
            SettlementStatus::Accepted => case.state = CaseState::Completed,
            SettlementStatus::Rejected => {
                case.state = CaseState::Terminated;
                case.close_reason = Some("rejected".into())
            }
            SettlementStatus::Escalated => {
                case.state = CaseState::Terminated;
                case.close_reason = Some("escalated".into())
            }
        }
        replace_json(&case_path, &case)?;
        trace.append(
            TraceKind::CaseClosed,
            None,
            json!({"case_id":case_id,"state":case.state}),
        )?;
        let capability_path = sandbox::safe_join(&root, "capabilities.jsonl")?;
        capability::append(&capability_path, &envelope)?;
        Ok(RunOutcome {
            run_id: run_id.clone(),
            case_id: case_id.clone(),
            run_dir: run_dir.clone(),
            decisions,
            execution,
            settlement: settled,
        })
    })();
    result.map_err(|error| {
        let error = ForgeError::run(run_id.clone(), error);
        sea_forge_trace::append_internal_error(
            &run_dir.join("trace.jsonl"),
            &run_id,
            &actor_for_error,
            error.class(),
        );
        error
    })
}

fn ensure_state_directory(root: &Path, name: &str) -> Result<PathBuf, ForgeError> {
    let path = sandbox::safe_join(root, name)?;
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_dir() => Ok(path),
        Ok(_) => Err(ForgeError::UnsafePath(format!(
            "state path is not a directory: {name}"
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&path)
                .map_err(|error| ForgeError::io(format!("create state directory {name}"), error))?;
            Ok(path)
        }
        Err(error) => Err(ForgeError::io(
            format!("inspect state directory {name}"),
            error,
        )),
    }
}
