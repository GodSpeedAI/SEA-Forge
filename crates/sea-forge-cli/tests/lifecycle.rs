use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("sea-forge-{name}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}
fn policy(root: &std::path::Path, rules: &str) -> PathBuf {
    let path = root.join("policy.yaml");
    fs::write(&path, format!("version: \"0.1\"\nrules:\n{rules}")).unwrap();
    path
}

fn integrity_policy(
    parent: &std::path::Path,
    min_witnesses: usize,
    include_witness: bool,
    include_domainforge: bool,
) -> PathBuf {
    const SURFACES: &[&str] = &[
        "authority_hooks",
        "identity_map",
        "domain_model",
        "file_access",
        "api_allowlist",
        "git_commit",
        "pr_merge",
        "prompt_risk",
        "memory_recall",
        "spec_pipeline",
        "artifact_transition",
        "attestation",
        "approval",
        "settlement_authority",
        "capability_promotion",
        "deployment",
        "secret_access",
        "policy_mutation",
        "evidence_mutation",
    ];
    const ROLES: &[&str] = &[
        "R-DS", "R-AG", "R-LC", "R-SO", "R-RM", "R-DEV", "R-AA", "operator",
    ];
    // (name, operation_kind, optional transition_kind)
    const SOD: &[(&str, &str, Option<&str>)] = &[
        ("production_proposer_approver", "pr_merge", None),
        (
            "semantic_debt_requester_acceptor",
            "settlement_authority_mutation",
            None,
        ),
        ("break_glass_requester_approver", "policy_mutation", None),
        ("key_generator_approver", "identity_minting", None),
        (
            "capitalization_requester_approver",
            "transition_artifact_stage",
            Some("capitalize"),
        ),
    ];
    let path = parent.join(format!("integrity-{min_witnesses}-{include_witness}.json"));
    let sources_dir = parent.join("sources");
    fs::create_dir_all(&sources_dir).unwrap();
    let sources = SURFACES
        .iter()
        .map(|surface| {
            let content = if *surface == "domain_model" && include_domainforge {
                "@namespace \"demo\"\n@version \"1.0.0\"\nEntity \"model\" in demo\n"
            } else {
                ""
            };
            fs::write(sources_dir.join(surface), content).unwrap();
            serde_json::json!({
                "surface": surface,
                "path": format!("sources/{surface}"),
                "sha256": sea_forge_evidence::sha256_bytes(content.as_bytes())
            })
        })
        .collect::<Vec<_>>();
    let witnesses = if include_witness {
        vec![serde_json::json!({
            "witness_id": "independent_witness",
            "key_dir": parent.join("witness-keys"),
            "key_id": "witness-key"
        })]
    } else {
        vec![]
    };
    let mut bundle: sea_forge_authority::AuthorityPolicyBundle = serde_json::from_value(
        serde_json::json!({
            "version": "0.2",
            "bundle_id": "bundle_integrity_test",
            "policy_bundle_version": "1",
            "loaded_at": "2026-07-13T00:00:00Z",
            "sources": sources,
            "roles": ROLES.iter().map(|role| ((*role).to_owned(), vec!["fixture"])).collect::<std::collections::BTreeMap<_, _>>(),
            "permissions": ROLES.iter().map(|role| serde_json::json!({"role": role, "operation_kind": "*"})).collect::<Vec<_>>(),
            "sod_rules": SOD.iter().map(|(name, operation, transition)| {
                let mut rule = serde_json::json!({
                    "name": name,
                    "requester_role": "R-DEV",
                    "approver_role": "R-SO",
                    "operation_kind": operation,
                    "allow_same_principal": false
                });
                if let Some(kind) = transition {
                    rule["transition_kind"] = serde_json::json!(kind);
                }
                rule
            }).collect::<Vec<_>>(),
            "identity_bindings": [{"principal": "operator_local", "actor_type": "human", "role": "operator"}],
            "policy_engines": if include_domainforge { vec![serde_json::json!({"engine": "domainforge", "fail_mode": "closed", "required": false})] } else { vec![] },
            "integrity_ledger": {
                "required_for_side_effects": true,
                "min_witnesses": min_witnesses,
                "signing_key_dir": parent.join("keys"),
                "signing_key_id": "test-key",
                "witnesses": witnesses
            },
            "rules": [
                {"name": "allow-model-write", "verdict": "allow", "actor_role": "operator", "operation_kind": "write_file", "path_prefix": ""},
                {"name": "allow-self-validate", "verdict": "allow", "actor_role": "operator", "operation_kind": "execute_command", "argv0": "sea-forge"},
                {"name": "allow-model-read", "verdict": "allow", "actor_role": "operator", "operation_kind": "validate_model"},
                {"name": "allow-inspect", "verdict": "allow", "actor_role": "operator", "operation_kind": "inspect_run"},
                {"name": "allow-recall", "verdict": "allow", "actor_role": "operator", "operation_kind": "recall_memory", "memory_scope": "any"}
            ]
        }),
    ).unwrap();
    bundle.refresh_policy_bundle_hash().unwrap();
    fs::write(&path, serde_json::to_vec_pretty(&bundle).unwrap()).unwrap();
    path
}

#[test]
fn required_integrity_checkpoint_precedes_command_start_and_witness_outage_halts() {
    for (min_witnesses, include_witness, expected_success) in
        [(0, false, true), (1, true, true), (1, false, false)]
    {
        let parent = temp_root(&format!("integrity-{min_witnesses}-{include_witness}"));
        let root = parent.join("state");
        let policy = integrity_policy(&parent, min_witnesses, include_witness, false);
        let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
            .args([
                "run",
                "--root",
                root.to_str().unwrap(),
                "--policy",
                policy.to_str().unwrap(),
                "--intent",
                "Generate and validate a simple DomainForge .sea model",
            ])
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            expected_success,
            "root: {}; status: {:?}; stderr: {}",
            root.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        let run_dir = fs::read_dir(root.join("runs"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let trace = fs::read_to_string(run_dir.join("trace.jsonl")).unwrap();
        assert_eq!(trace.contains("command_started"), expected_success);
        if expected_success {
            assert!(root.join("ledgers/global-checkpoints.jsonl").is_file());
            if include_witness {
                let run_id = run_dir.file_name().unwrap().to_str().unwrap();
                let inspect = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
                    .args([
                        "inspect",
                        run_id,
                        "--root",
                        root.to_str().unwrap(),
                        "--policy",
                        policy.to_str().unwrap(),
                    ])
                    .output()
                    .unwrap();
                assert!(
                    inspect.status.success(),
                    "inspect stderr: {}",
                    String::from_utf8_lossy(&inspect.stderr)
                );
                assert!(String::from_utf8(inspect.stdout)
                    .unwrap()
                    .contains("legacy_digest_only"));
                // Assurance is exposed through the governed memory-recall
                // evidence/log path (Task 8), not by mutating the
                // compatibility capability-envelope JSON.
                let recall = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
                    .args([
                        "recall",
                        "attempt",
                        "--root",
                        root.to_str().unwrap(),
                        "--policy",
                        policy.to_str().unwrap(),
                        "--kind",
                        "outcome",
                    ])
                    .output()
                    .unwrap();
                assert!(
                    recall.status.success(),
                    "recall stderr: {}",
                    String::from_utf8_lossy(&recall.stderr)
                );
                let recall_stderr = String::from_utf8(recall.stderr).unwrap();
                assert!(
                    recall_stderr.contains("externally_verified"),
                    "recall stderr: {recall_stderr}"
                );
            }
        } else {
            assert!(!run_dir.join("workspace/model.sea").exists());
        }
    }
}

#[test]
fn run_ingress_composes_domainforge_candidate() {
    let parent = temp_root("domainforge-ingress");
    let root = parent.join("state");
    let policy = integrity_policy(&parent, 0, false, true);
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run_dir = fs::read_dir(root.join("runs"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let decisions: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("authority.json")).unwrap()).unwrap();
    assert!(decisions.as_array().unwrap().iter().any(|decision| {
        decision["candidate_verdicts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|candidate| candidate["engine"] == "domainforge")
    }));
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn intent_to_settlement_produces_complete_accepted_run() {
    let parent = temp_root("accepted");
    let root = parent.join("state");
    let policy = parent.join("policy.yaml");
    fs::write(&policy, "version: \"0.1\"\nidentity:\n  source: test-binding\n  allow_unresolved: false\nrules:\n  - name: allow-model-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-self-validate\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let run_id = stdout
        .lines()
        .find_map(|l| l.strip_prefix("run_id="))
        .unwrap();
    let run = root.join("runs").join(run_id);
    for name in [
        "plan.json",
        "authority.json",
        "trace.jsonl",
        "evidence.jsonl",
        "settlement.json",
        "semantic-envelope.json",
    ] {
        assert!(run.join(name).is_file(), "missing {name}");
    }
    let settlement: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("settlement.json")).unwrap()).unwrap();
    assert_eq!(settlement["status"], "accepted");
    let authority: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("authority.json")).unwrap()).unwrap();
    assert!(authority[0]["action_request"]["context"]["workspace_root"]
        .as_str()
        .is_some_and(|path| std::path::Path::new(path).is_absolute()));
    assert_eq!(
        authority[0]["identity_binding"]["identity_binding_source"],
        "test-binding"
    );
    let trace_kinds: Vec<String> = fs::read_to_string(run.join("trace.jsonl"))
        .unwrap()
        .lines()
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line).unwrap()["kind"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(
        trace_kinds,
        [
            "case_created",
            "run_started",
            "plan_created",
            "authority_evaluated",
            "authority_evaluated",
            "workspace_created",
            "command_started",
            "command_finished",
            "artifact_captured",
            "artifact_captured",
            "artifact_captured",
            "settlement_recorded",
            "run_finished",
            "case_closed"
        ]
    );
    let envelope: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("semantic-envelope.json")).unwrap()).unwrap();
    assert!(envelope["artifact_refs"]
        .as_array()
        .is_some_and(|v| v.len() == 1));
    assert_eq!(envelope["settlement_ref"], settlement["settlement_id"]);
    let decision_ids: Vec<&str> = authority
        .as_array()
        .unwrap()
        .iter()
        .map(|decision| decision["decision_id"].as_str().unwrap())
        .collect();
    assert!(envelope["authority_decisions"]
        .as_array()
        .unwrap()
        .iter()
        .all(|id| decision_ids.contains(&id.as_str().unwrap())));
    let trace_ids: Vec<String> = fs::read_to_string(run.join("trace.jsonl"))
        .unwrap()
        .lines()
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line).unwrap()["event_id"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    let evidence_records: Vec<serde_json::Value> = fs::read_to_string(run.join("evidence.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let evidence_ids: Vec<&str> = evidence_records
        .iter()
        .map(|record| record["evidence_id"].as_str().unwrap())
        .collect();
    assert!(envelope["evidence_refs"]
        .as_array()
        .unwrap()
        .iter()
        .all(|id| evidence_ids.contains(&id.as_str().unwrap())));
    assert!(authority.as_array().unwrap().iter().all(|decision| {
        decision["audit_record"]["evidence_refs"]
            .as_array()
            .is_some_and(|refs| {
                !refs.is_empty()
                    && refs
                        .iter()
                        .all(|id| evidence_ids.contains(&id.as_str().unwrap()))
            })
    }));
    for record in &evidence_records {
        match record["kind"].as_str().unwrap() {
            "artifact" => {
                let uri = record["uri"].as_str().unwrap();
                assert_eq!(
                    sea_forge_evidence::sha256_file(&run.join(uri)).unwrap(),
                    record["sha256"].as_str().unwrap()
                );
            }
            "authority_decision" => {
                assert!(decision_ids.contains(&record["uri"].as_str().unwrap()))
            }
            "execution_result" => assert!(trace_ids
                .iter()
                .any(|id| id == record["uri"].as_str().unwrap())),
            other => panic!("unexpected evidence kind {other}"),
        }
    }
    let plan_record: sea_forge_core::types::CasePlan =
        serde_json::from_slice(&fs::read(run.join("plan.json")).unwrap()).unwrap();
    let validate_op = plan_record.items[0]
        .operations
        .iter()
        .find_map(|op| match op {
            sea_forge_core::types::Operation::ExecuteCommand { argv, .. } => Some(argv),
            _ => None,
        })
        .expect("demo plan must include execute_command validate");
    assert_eq!(
        validate_op.as_slice(),
        [env!("CARGO_BIN_EXE_sea-forge"), "validate", "model.sea"]
    );
    assert_eq!(
        envelope["plan_ref"].as_str().unwrap(),
        plan_record.plan_id.to_string()
    );
    assert_eq!(
        serde_json::from_slice::<sea_forge_core::types::CasePlan>(
            &serde_json::to_vec(&plan_record).unwrap()
        )
        .unwrap(),
        plan_record
    );
    let authority_records: Vec<sea_forge_core::types::AuthorityDecision> =
        serde_json::from_slice(&fs::read(run.join("authority.json")).unwrap()).unwrap();
    assert_eq!(
        serde_json::from_slice::<Vec<sea_forge_core::types::AuthorityDecision>>(
            &serde_json::to_vec(&authority_records).unwrap()
        )
        .unwrap(),
        authority_records
    );
    for line in fs::read_to_string(run.join("trace.jsonl")).unwrap().lines() {
        let record: sea_forge_core::types::TraceEvent = serde_json::from_str(line).unwrap();
        assert_eq!(
            serde_json::from_str::<sea_forge_core::types::TraceEvent>(
                &serde_json::to_string(&record).unwrap()
            )
            .unwrap(),
            record
        );
    }
    for line in fs::read_to_string(run.join("evidence.jsonl"))
        .unwrap()
        .lines()
    {
        let record: sea_forge_core::types::EvidenceRecord = serde_json::from_str(line).unwrap();
        assert_eq!(
            serde_json::from_str::<sea_forge_core::types::EvidenceRecord>(
                &serde_json::to_string(&record).unwrap()
            )
            .unwrap(),
            record
        );
    }
    let settlement_record: sea_forge_core::types::SettlementEvent =
        serde_json::from_slice(&fs::read(run.join("settlement.json")).unwrap()).unwrap();
    assert_eq!(
        serde_json::from_slice::<sea_forge_core::types::SettlementEvent>(
            &serde_json::to_vec(&settlement_record).unwrap()
        )
        .unwrap(),
        settlement_record
    );
    let envelope_record: sea_forge_core::types::SemanticEnvelope =
        serde_json::from_slice(&fs::read(run.join("semantic-envelope.json")).unwrap()).unwrap();
    assert_eq!(
        serde_json::from_slice::<sea_forge_core::types::SemanticEnvelope>(
            &serde_json::to_vec(&envelope_record).unwrap()
        )
        .unwrap(),
        envelope_record
    );
    assert_eq!(
        fs::read_to_string(root.join("capabilities.jsonl"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    let inspect = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["inspect", run_id, "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(inspect.status.code(), Some(0));
    let inspected = String::from_utf8(inspect.stdout).unwrap();
    for name in [
        "plan.json",
        "authority.json",
        "trace.jsonl",
        "evidence.jsonl",
        "settlement.json",
        "semantic-envelope.json",
    ] {
        assert!(inspected.contains(&format!("== {name} ==")));
    }
    let no_match = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["recall", "does-not-match", "--root", root.to_str().unwrap()])
        .status()
        .unwrap();
    assert_eq!(no_match.code(), Some(3));
    let case_path = root
        .join("cases")
        .join(format!("{}.json", envelope["case_ref"].as_str().unwrap()));
    let case: serde_json::Value = serde_json::from_slice(&fs::read(case_path).unwrap()).unwrap();
    assert_eq!(case["state"], "completed");
    assert_eq!(case["case_id"], envelope["case_ref"]);
    assert_eq!(case["plan_ref"], envelope["plan_ref"]);
    assert_eq!(case["run_ids"][0], run_id);
    let case_record: sea_forge_core::types::Case = serde_json::from_value(case).unwrap();
    assert_eq!(
        serde_json::from_slice::<sea_forge_core::types::Case>(
            &serde_json::to_vec(&case_record).unwrap()
        )
        .unwrap(),
        case_record
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn authority_denial_settles_rejected_without_command_start() {
    let parent = temp_root("denied");
    let root = parent.join("state");
    let policy = policy(&parent, "  []\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let run_id = stdout
        .lines()
        .find_map(|l| l.strip_prefix("run_id="))
        .unwrap();
    let run = root.join("runs").join(run_id);
    let trace_kinds: Vec<String> = fs::read_to_string(run.join("trace.jsonl"))
        .unwrap()
        .lines()
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line).unwrap()["kind"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(
        trace_kinds,
        [
            "case_created",
            "run_started",
            "plan_created",
            "authority_evaluated",
            "authority_evaluated",
            "run_halted",
            "settlement_recorded",
            "run_finished",
            "case_closed"
        ]
    );
    assert!(run.join("workspace").is_dir());
    assert_eq!(fs::read_dir(run.join("workspace")).unwrap().count(), 0);
    let decisions: Vec<serde_json::Value> =
        serde_json::from_slice(&fs::read(run.join("authority.json")).unwrap()).unwrap();
    assert!(decisions
        .iter()
        .all(|decision| decision["verdict"] == "deny"
            && decision["matched_rule"].is_null()
            && decision["audit_record"]["engine"].is_string()
            && decision["audit_record"]["disposition"].is_string()
            && decision["audit_record"]["subject"].is_string()));
    let authority_evidence = fs::read_to_string(run.join("evidence.jsonl"))
        .unwrap()
        .lines()
        .filter(|line| {
            serde_json::from_str::<serde_json::Value>(line).unwrap()["kind"] == "authority_decision"
        })
        .count();
    assert_eq!(authority_evidence, decisions.len());
    let envelope: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("semantic-envelope.json")).unwrap()).unwrap();
    let plan: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("plan.json")).unwrap()).unwrap();
    assert_eq!(envelope["plan_ref"], plan["plan_id"]);
    let case_file = root
        .join("cases")
        .join(format!("{}.json", envelope["case_ref"].as_str().unwrap()));
    let case: serde_json::Value = serde_json::from_slice(&fs::read(case_file).unwrap()).unwrap();
    assert_eq!(case["state"], "terminated");
    assert_eq!(case["close_reason"], "rejected");
    assert_eq!(case["case_id"], envelope["case_ref"]);
    assert_eq!(case["plan_ref"], envelope["plan_ref"]);
    assert_eq!(case["run_ids"], serde_json::json!([run_id]));
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn escalation_halts_and_requires_review() {
    let parent = temp_root("escalated");
    let root = parent.join("state");
    let policy = policy(&parent, "  - name: review-write\n    verdict: escalate\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: review-command\n    verdict: escalate\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let run_id = stdout
        .lines()
        .find_map(|line| line.strip_prefix("run_id="))
        .unwrap();
    let run = root.join("runs").join(run_id);
    let settlement: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("settlement.json")).unwrap()).unwrap();
    assert_eq!(settlement["status"], "escalated");
    assert_eq!(settlement["review_required"], true);
    let decisions: Vec<serde_json::Value> =
        serde_json::from_slice(&fs::read(run.join("authority.json")).unwrap()).unwrap();
    assert!(decisions
        .iter()
        .all(|decision| decision["audit_record"]["engine"].is_string()
            && decision["audit_record"]["disposition"] == "escalate"));
    let trace_kinds: Vec<String> = fs::read_to_string(run.join("trace.jsonl"))
        .unwrap()
        .lines()
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line).unwrap()["kind"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(
        trace_kinds,
        [
            "case_created",
            "run_started",
            "plan_created",
            "authority_evaluated",
            "authority_evaluated",
            "run_halted",
            "settlement_recorded",
            "run_finished",
            "case_closed"
        ]
    );
    let envelope: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("semantic-envelope.json")).unwrap()).unwrap();
    let plan: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("plan.json")).unwrap()).unwrap();
    assert_eq!(envelope["plan_ref"], plan["plan_id"]);
    let case_file = root
        .join("cases")
        .join(format!("{}.json", envelope["case_ref"].as_str().unwrap()));
    let case: serde_json::Value = serde_json::from_slice(&fs::read(case_file).unwrap()).unwrap();
    assert_eq!(case["state"], "terminated");
    assert_eq!(case["close_reason"], "escalated");
    assert_eq!(case["case_id"], envelope["case_ref"]);
    assert_eq!(case["plan_ref"], envelope["plan_ref"]);
    assert_eq!(case["run_ids"], serde_json::json!([run_id]));
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn repeated_runs_have_stable_artifact_identity_and_recall_is_read_only() {
    let parent = temp_root("repeat");
    let root = parent.join("state");
    let policy = policy(&parent, "  - name: allow-model-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-self-validate\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n");
    let mut identities = Vec::new();
    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
            .args([
                "run",
                "--root",
                root.to_str().unwrap(),
                "--policy",
                policy.to_str().unwrap(),
                "--entity",
                "team_a",
                "--process",
                "agent_1",
                "--intent",
                "Generate and validate a simple DomainForge .sea model",
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = String::from_utf8(output.stdout).unwrap();
        let run_id = stdout
            .lines()
            .find_map(|line| line.strip_prefix("run_id="))
            .unwrap();
        let run = root.join("runs").join(run_id);
        let envelope: serde_json::Value =
            serde_json::from_slice(&fs::read(run.join("semantic-envelope.json")).unwrap()).unwrap();
        let work_product: serde_json::Value = fs::read_to_string(run.join("evidence.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .find(|record| record["uri"] == "artifacts/model.sea")
            .unwrap();
        let artifact = &work_product["metadata"]["artifact"];
        assert_eq!(artifact["content_sha256"], work_product["sha256"]);
        let identity_input = serde_json::json!({"artifact_type":artifact["artifact_type"],"stage":artifact["stage"],"owner":artifact["owner"],"license":artifact["license"],"review_status":artifact["review_status"],"content_sha256":artifact["content_sha256"],"source_refs":artifact["source_refs"]});
        let expected_identity = format!(
            "ifl:hash:{}",
            sea_forge_evidence::sha256_bytes(
                &sea_forge_evidence::canonical_json(&identity_input).unwrap()
            )
        );
        assert_eq!(artifact["pre_mint_identity"], expected_identity);
        assert!(envelope["artifact_refs"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |reference| reference["evidence_id"] == work_product["evidence_id"]
                    && reference["artifact_id"] == artifact["artifact_id"]
                    && reference["pre_mint_identity"] == artifact["pre_mint_identity"]
            ));
        identities.push(
            envelope["artifact_refs"][0]["pre_mint_identity"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    assert_eq!(identities[0], identities[1]);
    assert_eq!(identities[0].len(), "ifl:hash:".len() + 64);
    assert!(identities[0]
        .strip_prefix("ifl:hash:")
        .is_some_and(|hash| hash
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())));
    let deny_policy = parent.join("deny.yaml");
    fs::write(&deny_policy, "version: \"0.1\"\nrules: []\n").unwrap();
    let denied = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            deny_policy.to_str().unwrap(),
            "--entity",
            "team_b",
            "--process",
            "agent_2",
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(denied.status.code(), Some(3));
    fs::OpenOptions::new()
        .append(true)
        .open(root.join("capabilities.jsonl"))
        .unwrap()
        .write_all(b"not-json\n")
        .unwrap();
    let before = fs::read(root.join("capabilities.jsonl")).unwrap();
    let recall = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "recall",
            "generate",
            "--root",
            root.to_str().unwrap(),
            "--entity",
            "team_a",
            "--process",
            "agent_1",
        ])
        .output()
        .unwrap();
    assert_eq!(recall.status.code(), Some(0));
    assert_eq!(String::from_utf8(recall.stdout).unwrap().lines().count(), 2);
    let warning: serde_json::Value = serde_json::from_slice(&recall.stderr).unwrap();
    assert_eq!(warning["error_class"], "capability_parse_error");
    let rejected = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "recall",
            "generate",
            "--root",
            root.to_str().unwrap(),
            "--entity",
            "team_b",
            "--process",
            "agent_2",
            "--result",
            "rejected",
        ])
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(rejected.stdout).unwrap().lines().count(),
        1
    );
    let newest = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "recall",
            "generate",
            "--root",
            root.to_str().unwrap(),
            "--limit",
            "1",
        ])
        .output()
        .unwrap();
    let newest_envelope: serde_json::Value = serde_json::from_slice(&newest.stdout).unwrap();
    assert_eq!(newest_envelope["attribution"]["entity_id"], "team_b");
    assert_eq!(before, fs::read(root.join("capabilities.jsonl")).unwrap());
    assert_eq!(fs::read_dir(root.join("runs")).unwrap().count(), 3);
    fs::remove_dir_all(parent).unwrap();
}

// ── Task 8: compatibility capability recall is byte-for-structure and read-only ──

#[test]
fn compat_capability_recall_is_byte_for_structure_and_read_only() {
    let parent = temp_root("compat-recall");
    let root = parent.join("state");
    let policy = policy(&parent, "  - name: allow-model-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-self-validate\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--entity",
            "team_a",
            "--process",
            "agent_1",
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "run stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stored_lines: Vec<serde_json::Value> = fs::read_to_string(root.join("capabilities.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        !stored_lines.is_empty(),
        "run must record at least one capability envelope"
    );
    let before = fs::read(root.join("capabilities.jsonl")).unwrap();

    let recall = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "recall",
            "generate",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        recall.status.code(),
        Some(0),
        "recall stderr: {}",
        String::from_utf8_lossy(&recall.stderr)
    );

    assert_eq!(
        before,
        fs::read(root.join("capabilities.jsonl")).unwrap(),
        "compatibility recall must never mutate capabilities.jsonl"
    );

    let printed: Vec<serde_json::Value> = String::from_utf8(recall.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        !printed.is_empty(),
        "recall must match the run's own envelope"
    );
    for envelope in &printed {
        assert!(
            envelope.as_object().unwrap().get("assurance").is_none(),
            "compatibility recall must never inject an assurance field: {envelope}"
        );
        assert!(
            stored_lines.contains(envelope),
            "printed envelope must equal a stored capabilities.jsonl envelope \
             byte-for-structure, with no added or removed fields: {envelope}"
        );
    }
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn validator_accepts_only_the_stub_contract() {
    let parent = temp_root("validate");
    let valid = parent.join("valid.sea");
    fs::write(
        &valid,
        r#"{"domain":"demo","entities":[{"name":"Sample"}]}"#,
    )
    .unwrap();
    let mut before: Vec<_> = fs::read_dir(&parent)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    before.sort();
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["validate", valid.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"sea-forge: model valid\n");
    // Config-free: no .sea-forge state, no policy/root required.
    assert!(!parent.join(".sea-forge").exists());
    assert!(!valid.parent().unwrap().join(".sea-forge").exists());
    let mut after: Vec<_> = fs::read_dir(&parent)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    after.sort();
    assert_eq!(before, after, "validate must not create or mutate state");
    let invalid = parent.join("invalid.sea");
    fs::write(&invalid, r#"{"domain":"demo","entities":[]}"#).unwrap();
    let invalid_output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["validate", invalid.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(invalid_output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&invalid_output.stderr).contains("sea-forge: model invalid:"));
    assert!(!parent.join(".sea-forge").exists());
    // Policy/root flags are rejected — the hidden validator is config-free.
    let with_policy = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "validate",
            valid.to_str().unwrap(),
            "--policy",
            parent.join("missing-policy.yaml").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        !with_policy.status.success(),
        "validate must not accept --policy"
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn false_success_and_nonzero_execution_settle_rejected_with_evidence() {
    let parent = temp_root("failures");
    let policy = policy(&parent, "  - name: allow-model-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-self\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n");
    for (intent, expected_basis) in [
        (
            "TEST_ONLY: false success",
            "required_artifact_missing:model.sea",
        ),
        ("TEST_ONLY: nonzero", "exit_nonzero"),
    ] {
        let root = parent.join(intent.replace([':', ' '], "-"));
        let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
            .args([
                "run",
                "--root",
                root.to_str().unwrap(),
                "--policy",
                policy.to_str().unwrap(),
                "--intent",
                intent,
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(3));
        let stdout = String::from_utf8(output.stdout).unwrap();
        let run_id = stdout
            .lines()
            .find_map(|line| line.strip_prefix("run_id="))
            .unwrap();
        let run = root.join("runs").join(run_id);
        let settlement: serde_json::Value =
            serde_json::from_slice(&fs::read(run.join("settlement.json")).unwrap()).unwrap();
        assert!(settlement["basis"]
            .as_array()
            .unwrap()
            .iter()
            .any(|basis| basis == expected_basis));
        if intent.ends_with("nonzero") {
            assert!(fs::read_to_string(run.join("artifacts/stderr.txt"))
                .unwrap()
                .contains("model invalid"));
        }
    }
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn pipeline_timeout_is_governed_and_rejected() {
    let parent = temp_root("pipeline-timeout");
    let root = parent.join("state");
    let policy = policy(&parent, "  - name: allow-self\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--timeout",
            "0",
            "--intent",
            "TEST_ONLY: timeout",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("execution=timed_out"));
    let run_id = stdout
        .lines()
        .find_map(|line| line.strip_prefix("run_id="))
        .unwrap();
    let settlement: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("runs").join(run_id).join("settlement.json")).unwrap(),
    )
    .unwrap();
    assert!(settlement["basis"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("timed_out")));
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(unix)]
#[test]
fn kill_9_leaves_a_valid_jsonl_prefix_without_capability_corruption() {
    use std::{collections::HashSet, process::Stdio, thread, time::Duration};
    let parent = temp_root("kill9");
    let root = parent.join("state");
    let policy = policy(&parent, "  - name: allow-model-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-self\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n");
    let seed = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(seed.status.code(), Some(0));
    let capability_before = fs::read(root.join("capabilities.jsonl")).unwrap();
    let existing: HashSet<_> = fs::read_dir(root.join("runs"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    let mut child = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--timeout",
            "30",
            "--intent",
            "TEST_ONLY: timeout",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut trace = None;
    // Wait for the run to reach `command_started`. This is a precondition for
    // the assertions below, not one of them — so the budget only has to be long
    // enough to distinguish "slow to start" from "never starts". At 2s it
    // failed roughly one run in three under a loaded machine, reporting a
    // startup delay as a lifecycle defect.
    for _ in 0..3_000 {
        if let Ok(runs) = fs::read_dir(root.join("runs")) {
            for run in runs
                .flatten()
                .filter(|run| !existing.contains(&run.file_name()))
            {
                let candidate = run.path().join("trace.jsonl");
                if fs::read_to_string(&candidate).is_ok_and(|text| text.contains("command_started"))
                {
                    trace = Some(candidate);
                    break;
                }
            }
        }
        if trace.is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let trace = trace.expect("command should start before kill timeout");
    let status = Command::new("kill")
        .args(["-9", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(status.success());
    let _ = child.wait().unwrap();
    let text = fs::read_to_string(trace).unwrap();
    assert!(!text.is_empty());
    for line in text.lines() {
        serde_json::from_str::<serde_json::Value>(line).unwrap();
    }
    assert_eq!(
        fs::read(root.join("capabilities.jsonl")).unwrap(),
        capability_before
    );
    for line in String::from_utf8(capability_before).unwrap().lines() {
        serde_json::from_str::<serde_json::Value>(line).unwrap();
    }
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn post_start_internal_error_is_run_correlated_and_traced() {
    let parent = temp_root("internal-error");
    let root = parent.join("state");
    fs::create_dir_all(root.join("capabilities.jsonl")).unwrap();
    let policy = policy(&parent, "  - name: allow-model-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n  - name: allow-self\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n");
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--intent",
            "Generate and validate a simple DomainForge .sea model",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    let run_id = diagnostic["run_id"].as_str().unwrap();
    assert!(run_id.starts_with("run_"));
    assert_eq!(diagnostic["error_class"], "internal_error");
    let trace = fs::read_to_string(root.join("runs").join(run_id).join("trace.jsonl")).unwrap();
    let last: serde_json::Value = serde_json::from_str(trace.lines().last().unwrap()).unwrap();
    assert_eq!(last["kind"], "internal_error");
    fs::remove_dir_all(parent).unwrap();
}
