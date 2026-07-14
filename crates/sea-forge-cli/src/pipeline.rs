use chrono::Utc;
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_capability as capability;
use sea_forge_core::ids;
use sea_forge_core::{errors::ForgeError, types::*, RECORD_VERSION};
use sea_forge_evidence::{capture_file, JsonlEvidenceWriter};
use sea_forge_ledger::{LedgerManager, LedgerStream};
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

pub(crate) fn rebuild_authority_mirrors(
    root: &Path,
    materializer: &LedgerStream,
    committed: &sea_forge_ledger::CommittedRecordRef,
) -> Result<(), ForgeError> {
    let manager = LedgerManager::new(root)?;
    let mut decisions = Vec::new();
    for stream_id in manager.list_stream_ids()? {
        let stream = manager.open_stream(&stream_id, "authority-rebuild")?;
        stream.verify()?;
        for entry in stream.read_entries()? {
            if entry.record_kind == "authority_decision" {
                decisions.push(entry);
            }
        }
    }
    decisions.sort_by(|left, right| {
        (&left.committed_at, &left.ledger_id, left.append_ordinal).cmp(&(
            &right.committed_at,
            &right.ledger_id,
            right.append_ordinal,
        ))
    });
    let mut decision_bytes = Vec::new();
    let mut audit_bytes = Vec::new();
    let mut opaque_constraints = Vec::new();
    let source_entry_ulids = decisions
        .iter()
        .map(|entry| entry.entry_ulid.clone())
        .collect::<Vec<_>>();
    for entry in decisions {
        serde_json::to_writer(&mut decision_bytes, &entry.payload)?;
        decision_bytes.push(b'\n');
        if let Some(audit) = entry.payload.get("audit_record") {
            serde_json::to_writer(&mut audit_bytes, audit)?;
            audit_bytes.push(b'\n');
        }
        if let Some(constraint_id) = entry
            .payload
            .get("opaque_constraint_id")
            .and_then(serde_json::Value::as_str)
        {
            opaque_constraints.push(json!({
                "constraint_id": constraint_id,
                "created_by_decision_id": entry.payload["decision_id"],
                "target": {
                    "resource_type": entry.payload["action_request"]["action"]["resource_type"],
                    "resource_id": entry.payload["action_request"]["action"]["resource_id"]
                },
                "reason": "all configured evaluators escalated",
                "created_at": entry.payload["decided_at"],
                "expires_at": serde_json::Value::Null
            }));
        }
    }
    materializer.materialize_aggregate_view(
        committed,
        source_entry_ulids.clone(),
        &root.join("authority/decisions.jsonl"),
        &decision_bytes,
    )?;
    materializer.materialize_aggregate_view(
        committed,
        source_entry_ulids.clone(),
        &root.join("authority/opaque-constraints.json"),
        &serde_json::to_vec_pretty(&opaque_constraints)?,
    )?;
    materializer.materialize_aggregate_view(
        committed,
        source_entry_ulids,
        &root.join("authority/audit.jsonl"),
        &audit_bytes,
    )?;
    Ok(())
}
pub(crate) fn load_opaque_constraints(
    root: &Path,
) -> Result<Vec<sea_forge_authority::OpaqueConstraint>, ForgeError> {
    let manager = LedgerManager::new(root)?;
    let mut constraints = BTreeMap::new();
    for stream_id in manager.list_stream_ids()? {
        let stream = manager.open_stream(&stream_id, "opaque-constraint-load")?;
        stream.verify()?;
        for entry in stream.read_entries()? {
            let Some(constraint_id) = entry
                .payload
                .get("opaque_constraint_id")
                .and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            let action = &entry.payload["action_request"]["action"];
            let resource_type = action["resource_type"].as_str().unwrap_or_default();
            let resource_id = action["resource_id"].as_str().unwrap_or_default();
            if resource_type.is_empty() || resource_id.is_empty() {
                continue;
            }
            constraints.insert(
                format!("{resource_type}:{resource_id}"),
                sea_forge_authority::OpaqueConstraint {
                    constraint_id: constraint_id.into(),
                    resource_type: resource_type.into(),
                    resource_id: resource_id.into(),
                    created_by_decision_id: entry.payload["decision_id"]
                        .as_str()
                        .unwrap_or_default()
                        .into(),
                    reason: "all configured evaluators escalated".into(),
                    created_at: entry.payload["decided_at"]
                        .as_str()
                        .unwrap_or_default()
                        .into(),
                    expires_at: None,
                },
            );
        }
    }
    Ok(constraints.into_values().collect())
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
    let policy_snapshot = bundle.clone();
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
        let actor = Actor {
            actor_id: intent.actor_id.clone(),
            role: ActorRole::Operator,
        };
        let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
        let executable =
            env::current_exe().map_err(|e| ForgeError::io("resolve current executable", e))?;
        let executable = executable.to_string_lossy().into_owned();
        let plan = planner::plan(&intent, &case_id, &run_id, &executable, &root)?;
        let authority_stream =
            LedgerStream::open(&root, format!("case-{case_id}"), &intent.actor_id)?;
        let mut criteria_map: BTreeMap<String, SettlementCriteriaRecord> = BTreeMap::new();
        let declared_at = Utc::now().to_rfc3339();
        let mut plan = plan;
        for item in &mut plan.items {
            let record =
                planner::derive_from_intent(&intent, item, &intent.actor_id, &declared_at)?;
            let committed = authority_stream.commit_typed(
                "settlement_criteria",
                vec![run_id.clone(), record.criteria_id.clone()],
                &record,
                vec![],
            )?;
            let _ = committed;
            item.settlement_criteria_ref = Some(record.criteria_id.clone());
            criteria_map.insert(record.criteria_id.clone(), record);
        }
        planner::verify_plan_criteria(&plan, &criteria_map)?;
        write_json(&run_dir.join("plan.json"), &plan)?;
        let item = plan
            .items
            .first()
            .ok_or_else(|| ForgeError::Internal("planner returned an empty plan".into()))?;
        authority_stream.commit_typed("intent", vec![run_id.clone()], &intent, vec![])?;
        authority_stream.commit_typed("case_plan", vec![run_id.clone()], &plan, vec![])?;
        authority_stream.commit_typed(
            "identity_binding",
            vec![run_id.clone()],
            &binding,
            vec![],
        )?;
        let committed_policy = authority_stream.commit_typed(
            "authority_policy",
            vec![run_id.clone()],
            &policy_snapshot,
            vec![],
        )?;
        authority_stream.materialize_view(
            &committed_policy,
            &root.join("authority/active-policy.json"),
            &serde_json::to_vec_pretty(&policy_snapshot)?,
        )?;
        let policy_hash = policy_snapshot
            .policy_bundle_hash
            .as_deref()
            .unwrap_or(committed_policy.payload_hash());
        authority_stream.materialize_view(
            &committed_policy,
            &root.join("authority/policy-bundles").join(format!(
                "{}.json",
                policy_hash.strip_prefix("sha256:").unwrap_or(policy_hash)
            )),
            &serde_json::to_vec_pretty(&policy_snapshot)?,
        )?;
        let mut case = Case {
            version: RECORD_VERSION.into(),
            case_id: case_id.clone(),
            intent: intent.clone(),
            state: CaseState::Active,
            plan_ref: plan.plan_id.clone(),
            run_ids: vec![run_id.clone()],
            stages: vec![],
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
        let domainforge_model = policy_snapshot.load_domainforge_model()?;
        let engine = PolicyAuthorityEngine::new(bundle)?;
        engine.load_opaque_constraints(load_opaque_constraints(&root)?)?;
        let mut decisions = Vec::new();
        for (index, operation) in item.operations.iter().enumerate() {
            let action = AuthorityAction::from(operation);
            let env_keys = if matches!(operation, Operation::ExecuteCommand { .. }) {
                ["PATH", "HOME"]
                    .into_iter()
                    .filter(|key| env::var(key).is_ok())
                    .map(str::to_owned)
                    .collect()
            } else {
                Default::default()
            };
            let evidence_id = format!("evi_{:04}", index + 1);
            let domainforge_candidate = domainforge_model
                .as_ref()
                .map(|model| {
                    sea_forge_authority::DomainForgeCandidate::evaluate(
                        model,
                        &action,
                        vec![format!(
                            "domain-model:{}",
                            model.model_ref.semantic_model_sha256
                        )],
                    )
                })
                .transpose()?;
            let decision = engine.evaluate(AuthorityEvaluation {
                actor: &actor,
                binding: binding.clone(),
                run_id: &run_id,
                case_id: &case_id,
                plan_item_id: &item.plan_item_id,
                sequence: index + 1,
                action: &action,
                workspace_root: &workspace,
                evidence_refs: vec![evidence_id.clone()],
                artifacts_root: matches!(operation, Operation::ExecuteCommand { .. })
                    .then_some(artifacts.as_path()),
                timeout_secs: matches!(operation, Operation::ExecuteCommand { .. })
                    .then_some(options.timeout_secs),
                env_keys,
                domainforge_candidate: domainforge_candidate.as_ref(),
            })?;
            let event = trace.append(
                TraceKind::AuthorityEvaluated,
                Some(item.plan_item_id.clone()),
                json!({"decision_id":decision.decision_id,"verdict":decision.verdict}),
            )?;
            let evidence_record = evidence.prepare(
                EvidenceKind::AuthorityDecision,
                decision.decision_id.clone(),
                None,
                event,
                BTreeMap::new(),
            );
            if evidence_record.evidence_id != evidence_id {
                return Err(ForgeError::Internal(
                    "authority evidence sequence diverged from decision".into(),
                ));
            }
            authority_stream.commit_typed(
                "authority_evidence",
                vec![run_id.clone(), decision.decision_id.clone()],
                &evidence_record,
                vec![],
            )?;
            evidence.append_prepared(evidence_record)?;
            decisions.push(decision)
        }
        let committed_decisions = decisions
            .iter()
            .map(|decision| {
                let request = authority_stream.commit_typed(
                    "authority_request",
                    vec![run_id.clone(), item.plan_item_id.clone()],
                    &decision.action_request,
                    vec![],
                )?;
                authority_stream.commit_typed(
                    "authority_decision",
                    vec![run_id.clone(), item.plan_item_id.clone()],
                    decision,
                    vec![request.entry_ulid().into()],
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let pre_action_assurance = if policy_snapshot.integrity_ledger.required_for_side_effects {
            let key_dir = policy_snapshot
                .integrity_ledger
                .signing_key_dir
                .as_deref()
                .ok_or_else(|| {
                    ForgeError::Internal(
                        "ledger_integrity_error: signing_key_dir is required".into(),
                    )
                })?;
            Some(
                LedgerManager::new(&root)?.create_pre_action_assurance(
                    key_dir,
                    &policy_snapshot.integrity_ledger.signing_key_id,
                    &intent.actor_id,
                    &policy_snapshot
                        .integrity_ledger
                        .witnesses
                        .iter()
                        .map(|witness| {
                            (
                                witness.witness_id.clone(),
                                witness.key_dir.clone(),
                                witness.key_id.clone(),
                            )
                        })
                        .collect::<Vec<_>>(),
                    policy_snapshot.integrity_ledger.min_witnesses,
                    &committed_decisions.iter().collect::<Vec<_>>(),
                )?,
            )
        } else {
            None
        };
        let authority_bytes = serde_json::to_vec_pretty(&decisions)?;
        authority_stream.materialize_view(
            committed_decisions
                .last()
                .ok_or_else(|| ForgeError::Internal("no authority decisions committed".into()))?,
            &run_dir.join("authority.json"),
            &authority_bytes,
        )?;
        rebuild_authority_mirrors(
            &root,
            &authority_stream,
            committed_decisions
                .last()
                .ok_or_else(|| ForgeError::Internal("no authority decisions committed".into()))?,
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
                        pre_action_assurance.as_ref(),
                    )?;
                    sandbox::materialize(
                        grant,
                        &workspace,
                        &run_id,
                        &item.plan_item_id,
                        operation,
                        &decisions[index].compensating_controls,
                    )?;
                }
            }
            if let Some((index, operation)) = item
                .operations
                .iter()
                .enumerate()
                .find(|(_, operation)| matches!(operation, Operation::ExecuteCommand { .. }))
            {
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
                    pre_action_assurance.as_ref(),
                )?;
                let start = trace.append(
                    TraceKind::CommandStarted,
                    Some(item.plan_item_id.clone()),
                    json!({}),
                )?;
                let result = runtime::execute(
                    grant,
                    &ExecutionRequest {
                        plan_item_id: item.plan_item_id.clone(),
                        operation: operation.clone(),
                        timeout_secs: options.timeout_secs,
                        env: envs,
                        compensating_controls: decisions[index].compensating_controls.clone(),
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
            criteria_ref: item.settlement_criteria_ref.clone(),
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
            template_ref: plan.template_ref.clone(),
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
        let committed_envelope = authority_stream.commit_typed(
            "capability_envelope",
            vec![run_id.clone(), settled.settlement_id.clone()],
            &envelope,
            vec![],
        )?;
        authority_stream.materialize_view(
            &committed_envelope,
            &run_dir.join("semantic-envelope.json"),
            &serde_json::to_vec_pretty(&envelope)?,
        )?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_mirrors_rebuild_byte_identically() {
        let root =
            std::env::temp_dir().join(format!("sea-forge-authority-mirror-{}", std::process::id()));
        let stream = LedgerStream::open(&root, "case_test", "test").unwrap();
        let committed = stream
            .commit_typed(
                "authority_decision",
                vec![],
                &json!({
                    "decision_id": "auth_01",
                    "decided_at": "2026-07-13T00:00:00Z",
                    "opaque_constraint_id": null,
                    "audit_record": {"outcome": "allow"}
                }),
                vec![],
            )
            .unwrap();
        rebuild_authority_mirrors(&root, &stream, &committed).unwrap();
        let path = root.join("authority/decisions.jsonl");
        let first = fs::read(&path).unwrap();
        fs::remove_file(&path).unwrap();
        rebuild_authority_mirrors(&root, &stream, &committed).unwrap();
        assert_eq!(fs::read(&path).unwrap(), first);
        fs::remove_dir_all(root).unwrap();
    }
}
