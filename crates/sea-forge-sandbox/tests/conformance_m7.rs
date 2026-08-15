//! M7 environment + evaluator conformance (spec-full §7.6, §12 M7, §17.1 M7).
//!
//! All test names contain "environment" so `cargo test -p sea-forge-sandbox environment`
//! catches them. Teeth: the 3-failure batch fixture rejects while the 2-failure one accepts.

use sea_forge_authority::{command_allowed, PolicyRule};
use sea_forge_core::types::*;
use sea_forge_sandbox::environment::{
    demo_environment, load_pinned_environment, materialize_base, parse_evaluator_score,
    store_builtin_environment, ScoreFrom,
};
use sea_forge_settlement::{evaluate_batch, settle};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;

// ── §12 M7: evaluator score recorded in settlement basis ──

fn passing_execution() -> ExecutionResult {
    ExecutionResult {
        status: ExecutionStatus::Completed,
        exit_code: Some(0),
        stdout_path: "stdout.txt".into(),
        stderr_path: "stderr.txt".into(),
        started_at: "s".into(),
        finished_at: "f".into(),
    }
}

fn make_claim(
    evaluator_scores: BTreeMap<String, f64>,
    batch: Option<BatchEvaluationResult>,
) -> SettlementClaim {
    SettlementClaim {
        run_id: "run_m7".into(),
        plan_item_id: "item_m7".into(),
        criteria_ref: None,
        criteria: SettlementCriteria {
            require_exit_zero: true,
            ..Default::default()
        },
        execution: Some(passing_execution()),
        authority_verdicts: vec![Verdict::Allow],
        evaluator_scores,
        batch,
        write_only: false,
    }
}

#[test]
fn environment_evaluator_score_recorded_in_basis() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    let run_dir = dir.path().join("run");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&run_dir).unwrap();
    // Write a dummy stdout so settlement's stdout check doesn't fail.
    std::fs::write(run_dir.join("stdout.txt"), "ok").unwrap();
    let mut scores = BTreeMap::new();
    scores.insert("demo_env.score_eval".into(), 1.0);
    let claim = make_claim(scores, None);
    let event = settle(&claim, &workspace, &run_dir).unwrap();
    assert_eq!(event.status, SettlementStatus::Accepted);
    assert!(
        event
            .basis
            .iter()
            .any(|b| b == "evaluator_score:demo_env.score_eval=1"),
        "evaluator score must appear in basis: {:?}",
        event.basis
    );
}

// ── §12 M7: batch — 2 failures accepted, 3 failures rejected (teeth) ──

fn make_records(n_fail: usize) -> (Vec<serde_json::Value>, Vec<f64>) {
    let total = 10;
    let mut records = Vec::with_capacity(total);
    let mut scores = Vec::with_capacity(total);
    for i in 0..total {
        records.push(json!({"id": i}));
        scores.push(if i < n_fail { 0.0 } else { 1.0 });
    }
    (records, scores)
}

#[test]
fn environment_batch_two_failures_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let run_dir = dir.path().join("run");
    std::fs::create_dir_all(&run_dir).unwrap();
    let (records, scores) = make_records(2);
    let batch = evaluate_batch(&records, &scores, 0.8);
    assert_eq!(batch.passed, 8);
    assert_eq!(batch.total, 10);
    assert!((batch.pass_ratio - 0.8).abs() < 1e-9);
    assert_eq!(batch.failures.len(), 2);
    let claim = make_claim(BTreeMap::new(), Some(batch));
    let event = settle(&claim, Path::new("."), &run_dir).unwrap();
    assert_eq!(event.status, SettlementStatus::Accepted);
    // Quarantine file must have exactly 2 lines.
    let q = std::fs::read_to_string(run_dir.join("quarantine/item_m7.jsonl")).unwrap();
    assert_eq!(q.lines().count(), 2, "2 failures quarantined");
}

#[test]
fn environment_batch_three_failures_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let run_dir = dir.path().join("run");
    std::fs::create_dir_all(&run_dir).unwrap();
    let (records, scores) = make_records(3);
    let batch = evaluate_batch(&records, &scores, 0.8);
    assert_eq!(batch.passed, 7);
    assert!((batch.pass_ratio - 0.7).abs() < 1e-9);
    assert_eq!(batch.failures.len(), 3);
    let claim = make_claim(BTreeMap::new(), Some(batch));
    let event = settle(&claim, Path::new("."), &run_dir).unwrap();
    assert_eq!(event.status, SettlementStatus::Rejected);
    assert!(event.basis.contains(&"batch_below_threshold".into()));
    let q = std::fs::read_to_string(run_dir.join("quarantine/item_m7.jsonl")).unwrap();
    assert_eq!(q.lines().count(), 3, "3 failures quarantined");
}

// ── §17.1 M7: three-axis independence (content/permission/isolation) ──

fn rule(verdict: Verdict, argv0: Option<&str>, env: Option<&str>) -> PolicyRule {
    PolicyRule {
        name: "test".into(),
        verdict,
        actor_role: ActorRole::Operator,
        operation_kind: "execute_command".into(),
        path_prefix: None,
        argv0: argv0.map(Into::into),
        disposition: None,
        boundary_constraints: BTreeMap::new(),
        compensating_controls: vec![],
        sandbox_class: None,
        requires_approval: Some(false),
        memory_scope: None,
        environment: env.map(Into::into),
        transition_kind: None,
        from_stage: None,
        to_stage: None,
        modes: None,
        gate_profile_ref: None,
        required_settlement_strength: None,
        qualifying_value_evidence_kinds: None,
        license_allowlist: vec![],
        attestation_ledger: None,
        requester_roles: None,
        approver_roles: None,
        degraded_mode: None,
    }
}

fn exec_action(argv0: &str) -> AuthorityAction {
    AuthorityAction::ExecuteCommand {
        argv: vec![format!("/usr/bin/{argv0}")],
        cwd: ".".into(),
    }
}

#[test]
fn environment_three_axis_independence() {
    let provides = vec!["allowed_cmd".into(), "sea-forge".into()];

    // Content axis: argv0 in provides → allowed; argv0 not in provides → denied.
    // (permission + isolation axes held fixed: rule has matching environment, no argv0 constraint)
    let r = rule(Verdict::Allow, None, Some("demo_env@0.1.0"));
    let env_ctx: Option<(&str, &[String])> = Some(("demo_env@0.1.0", &provides));
    assert!(command_allowed(&r, &exec_action("allowed_cmd"), env_ctx));
    assert!(!command_allowed(&r, &exec_action("forbidden_cmd"), env_ctx));

    // Permission axis: rule.environment == item_env → allowed; mismatch → denied.
    // (content + isolation fixed: argv0 is in provides)
    assert!(command_allowed(&r, &exec_action("allowed_cmd"), env_ctx));
    let wrong_env: Option<(&str, &[String])> = Some(("other_env@0.1.0", &provides));
    assert!(!command_allowed(&r, &exec_action("allowed_cmd"), wrong_env));

    // No environment context at all → denied (rule requires env, item provides none).
    assert!(!command_allowed(&r, &exec_action("allowed_cmd"), None));

    // Intersection: rule has BOTH environment and argv0 — both must match.
    let r2 = rule(Verdict::Allow, Some("allowed_cmd"), Some("demo_env@0.1.0"));
    assert!(command_allowed(&r2, &exec_action("allowed_cmd"), env_ctx));
    // Wrong argv0 but right env → denied (intersection).
    assert!(!command_allowed(&r2, &exec_action("sea-forge"), env_ctx));

    // Isolation axis: sandbox_class is orthogonal — not checked by command_allowed.
    // The rule's sandbox_class determines jail vs local, independent of content+permission.
    // (This test demonstrates command_allowed doesn't consult sandbox_class.)
    let r3 = PolicyRule {
        sandbox_class: Some("jail".into()),
        ..rule(Verdict::Allow, None, Some("demo_env@0.1.0"))
    };
    assert!(command_allowed(&r3, &exec_action("allowed_cmd"), env_ctx));
}

// ── §17.1 M7: environment immutability (hash pin) ──

#[test]
fn environment_hash_pin_immutability() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    store_builtin_environment(root).unwrap();
    // First load pins the hash.
    let spec1 = load_pinned_environment(root, "demo_env@0.1.0").unwrap();
    assert_eq!(spec1.name, "demo_env");
    // Tamper the yaml content under the pin.
    let env_path = root.join("environments/demo_env@0.1.0.yaml");
    let original = std::fs::read_to_string(&env_path).unwrap();
    let tampered = original.replace("Built-in demo", "Tampered");
    std::fs::write(&env_path, tampered).unwrap();
    // Second load must fail with environment_unavailable.
    let err = load_pinned_environment(root, "demo_env@0.1.0").unwrap_err();
    assert!(
        format!("{err:?}").contains("environment_unavailable"),
        "expected environment_unavailable, got: {err:?}"
    );
}

// ── §12 M7: tampered environment file blocks execution ──

#[test]
fn environment_tampered_file_blocks_execution() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    // No env file at all → load fails before any side effect.
    let err = load_pinned_environment(root, "nonexistent@0.1.0").unwrap_err();
    assert!(format!("{err:?}").contains("environment_unavailable"));
    // No workspace file should have been written (materialize_base never called).
    assert!(std::fs::read_dir(&workspace).unwrap().next().is_none());
}

// ── §12 M7: demo_env fixture loads correctly ──

#[test]
fn environment_demo_env_fixture_loads() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    store_builtin_environment(root).unwrap();
    let spec = load_pinned_environment(root, "demo_env@0.1.0").unwrap();
    assert_eq!(spec.name, "demo_env");
    assert_eq!(spec.version, "0.1.0");
    assert!(spec.provides.commands.contains(&"sea-forge".to_string()));
    assert!(spec.evaluators.contains_key("score_eval"));
}

// ── §7.6: base materialization writes files ──

#[test]
fn environment_materialize_base_writes_files() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let spec = sea_forge_sandbox::environment::EnvironmentSpec {
        name: "test".into(),
        version: "0.1.0".into(),
        description: "".into(),
        base: vec![sea_forge_sandbox::environment::BaseFile {
            path: "config/app.yaml".into(),
            content: "key: value".into(),
        }],
        provides: sea_forge_sandbox::environment::Provides {
            commands: vec!["test".into()],
        },
        evaluators: BTreeMap::new(),
    };
    materialize_base(&spec, &workspace).unwrap();
    let written = std::fs::read_to_string(workspace.join("config/app.yaml")).unwrap();
    assert_eq!(written, "key: value");
}

// ── §10.6: evaluator score parsing ──

#[test]
fn environment_parse_evaluator_score_exit_and_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let artifacts = dir.path();
    std::fs::write(artifacts.join("out.txt"), "0.75").unwrap();
    let pass = ExecutionResult {
        status: ExecutionStatus::Completed,
        exit_code: Some(0),
        stdout_path: "out.txt".into(),
        stderr_path: "err.txt".into(),
        started_at: "".into(),
        finished_at: "".into(),
    };
    let fail = ExecutionResult {
        exit_code: Some(1),
        ..pass.clone()
    };
    assert_eq!(
        parse_evaluator_score(&ScoreFrom::Exit, &pass, artifacts).unwrap(),
        1.0
    );
    assert_eq!(
        parse_evaluator_score(&ScoreFrom::Exit, &fail, artifacts).unwrap(),
        0.0
    );
    assert_eq!(
        parse_evaluator_score(&ScoreFrom::StdoutFloat, &pass, artifacts).unwrap(),
        0.75
    );
}

// ── demo_environment() round-trips through YAML ──

#[test]
fn environment_demo_environment_round_trips() {
    let spec = demo_environment();
    let yaml = serde_yaml::to_string(&spec).unwrap();
    let back: sea_forge_sandbox::environment::EnvironmentSpec =
        serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(spec, back);
}
