use crate::{
    authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine},
    capability,
    errors::ForgeError,
    evidence::{capture_file, JsonlEvidenceWriter},
    ids, planner, runtime, sandbox, settlement,
    trace::JsonlTraceRecorder,
    types::*,
    RECORD_VERSION,
};
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use std::{
    collections::BTreeMap,
    env, fs,
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
    pub executable: Option<PathBuf>,
}
#[derive(Clone, Debug)]
pub struct RunOutcome {
    pub run_id: String,
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
    crate::domain::interpret(&options.intent)?;
    validate_attribution(&options.entity, "entity")?;
    validate_attribution(&options.process, "process")?;
    fs::create_dir_all(options.root.join("runs"))
        .map_err(|e| ForgeError::io("preflight root", e))?;
    fs::create_dir_all(options.root.join("cases"))
        .map_err(|e| ForgeError::io("preflight cases", e))?;
    let run_id = ids::run_id()?;
    let case_id = ids::case_id()?;
    let run_dir = options.root.join("runs").join(&run_id);
    fs::create_dir(&run_dir).map_err(|e| ForgeError::io("create run directory", e))?;
    let workspace = run_dir.join("workspace");
    let artifacts = run_dir.join("artifacts");
    fs::create_dir(&artifacts).map_err(|e| ForgeError::io("create artifacts", e))?;
    let now = Utc::now().to_rfc3339();
    let intent = Intent {
        intent_id: ids::random_id("int")?,
        summary: options.intent,
        actor_id: options.entity.clone(),
        process_id: options.process.clone(),
        created_at: now.clone(),
    };
    let executable = options.executable.unwrap_or(
        env::current_exe().map_err(|e| ForgeError::io("resolve current executable", e))?,
    );
    let executable = executable.to_string_lossy().into_owned();
    let plan = planner::plan(&intent, &case_id, &run_id, &executable)?;
    let item = &plan.items[0];
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
    replace_json(
        &options.root.join("cases").join(format!("{case_id}.json")),
        &case,
    )?;
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
    let engine = PolicyAuthorityEngine::new(bundle)?;
    let actor = Actor {
        actor_id: intent.actor_id.clone(),
        role: ActorRole::Operator,
    };
    let binding = IdentityBinding {
        principal: actor.actor_id.clone(),
        actor_type: ActorType::Human,
        binding_resolution: BindingResolution::LocalDefault,
        identity_binding_source: "local-slice-default".into(),
        sponsor: None,
    };
    let mut decisions = Vec::new();
    for (index, operation) in item.operations.iter().enumerate() {
        let decision = engine.evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: binding.clone(),
            run_id: &run_id,
            plan_item_id: &item.plan_item_id,
            sequence: index + 1,
            operation,
            workspace_root: &workspace,
        })?;
        let event = trace.append(
            TraceKind::AuthorityEvaluated,
            Some(item.plan_item_id.clone()),
            json!({"decision_id":decision.decision_id,"verdict":decision.verdict}),
        )?;
        evidence.append(
            EvidenceKind::AuthorityDecision,
            decision.decision_id.clone(),
            None,
            event,
            BTreeMap::new(),
        )?;
        decisions.push(decision)
    }
    write_json(&run_dir.join("authority.json"), &decisions)?;
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
        fs::create_dir(&workspace).map_err(|e| ForgeError::io("create workspace", e))?;
        trace.append(TraceKind::WorkspaceCreated, None, json!({}))?;
        for operation in &item.operations {
            if matches!(operation, Operation::WriteFile { .. }) {
                sandbox::materialize(&workspace, operation)?;
            }
        }
        if let Some(operation) = item
            .operations
            .iter()
            .find(|o| matches!(o, Operation::ExecuteCommand { .. }))
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
            let result = runtime::execute(
                &ExecutionRequest {
                    plan_item_id: item.plan_item_id.clone(),
                    operation: operation.clone(),
                    timeout_secs: options.timeout_secs,
                    env: envs,
                },
                &workspace,
                &artifacts,
            )?;
            let finished = trace.append(
                TraceKind::CommandFinished,
                Some(item.plan_item_id.clone()),
                json!({"status":result.status,"started_event":start}),
            )?;
            evidence.append(
                EvidenceKind::ExecutionResult,
                "execution_result".into(),
                None,
                finished.clone(),
                BTreeMap::new(),
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
            let model = workspace.join("model.sea");
            if model.is_file() {
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
                let d = descriptor.expect("descriptor requested");
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
    let settled = settlement::settle(&claim, &workspace, &run_dir);
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
    capability::append(&options.root.join("capabilities.jsonl"), &envelope)?;
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
    trace.append(
        TraceKind::CaseClosed,
        None,
        json!({"case_id":case_id,"state":case.state}),
    )?;
    write_json(
        &options.root.join("cases").join(format!("{case_id}.json")),
        &case,
    )?;
    Ok(RunOutcome {
        run_id,
        case_id,
        run_dir,
        decisions,
        execution,
        settlement: settled,
    })
}
