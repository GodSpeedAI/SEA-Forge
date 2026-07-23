use sea_forge_case_runner::run_stage_case;
use sea_forge_core::types::{SpecPipelineStage, StageFile, StageKind, StageStatus};
use sea_forge_planner::stage_case_plan;
use std::fs;
use std::path::Path;

fn policy(root: &Path) -> std::path::PathBuf {
    let path = root.join("policy.yaml");
    fs::write(
        &path,
        "version: \"0.1\"\nrules:\n  - name: allow-execute\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    sandbox_class: jail\n",
    )
    .unwrap();
    path
}

fn stage(stage_id: &str, kind: StageKind, command: Vec<String>) -> SpecPipelineStage {
    SpecPipelineStage {
        stage_id: stage_id.into(),
        kind,
        inputs: vec![],
        outputs: vec![],
        command: Some(command),
        status: StageStatus::Pending,
        quarantine_ref: None,
        settlement_basis: vec![],
    }
}

fn placeholder_command() -> Vec<String> {
    // Only stages that reach real authority/execution in a test need a
    // trustable self-invoking argv[0] (ExecuteCommand may only re-invoke
    // this exact process's own executable, never an arbitrary command —
    // see `sea_forge_authority::untrusted_executable`). Stages whose test
    // scenario is denied/quarantined before execution never spawn this.
    vec!["sh".into(), "-c".into(), "exit 0".into()]
}

fn self_exe() -> String {
    std::env::current_exe()
        .unwrap()
        .canonicalize()
        .unwrap()
        .display()
        .to_string()
}

/// A self-invoking command that succeeds: re-runs this exact test binary,
/// selecting only the trivial `#[ignore]`d no-op test below (which exits 0).
/// `--ignored --exact` is required so the normal test run never executes it.
fn self_invoke_pass() -> Vec<String> {
    vec![
        self_exe(),
        "self_invoke_noop_pass".into(),
        "--exact".into(),
        "--ignored".into(),
    ]
}

/// A self-invoking command that fails: re-runs this exact test binary,
/// selecting only the `#[ignore]`d panicking test below (non-zero exit).
fn self_invoke_fail() -> Vec<String> {
    vec![
        self_exe(),
        "self_invoke_noop_fail".into(),
        "--exact".into(),
        "--ignored".into(),
    ]
}

// Both helpers are `#[ignore]`d so a normal `cargo test` run never executes
// them; they only run when explicitly re-invoked with `--ignored --exact`
// (mirroring `sea-forge-sandbox`'s `net_probe_helper` self-invocation idiom).
#[test]
#[ignore]
fn self_invoke_noop_pass() {}

#[test]
#[ignore]
fn self_invoke_noop_fail() {
    panic!("intentional failure for M5 self-invocation fixture");
}

// ── 1. An accepted stage settles and completes the case ──

#[test]
fn accepted_stage_settles_and_completes_case() {
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path());
    let stages = vec![stage("stage_adr", StageKind::Adr, self_invoke_pass())];
    let plan = stage_case_plan(&stages, "case_m5_1", "run_m5_1", "int_m5_1").unwrap();

    let outcome =
        run_stage_case(root.path(), &policy_path, "operator_local", &plan, stages).unwrap();

    assert_eq!(outcome.state, "completed");
    assert_eq!(outcome.stages[0].status, StageStatus::Accepted);
    assert!(outcome.stages[0].quarantine_ref.is_none());
}

// ── 2. A denied generated-zone edit is quarantined before authority ──

#[test]
fn denied_generated_zone_edit_is_quarantined_before_authority() {
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path());
    let mut adr_stage = stage("stage_adr", StageKind::Adr, placeholder_command());
    adr_stage.outputs = vec![StageFile {
        path: "src/gen/model.rs".into(),
        sha256: "sha256:should-not-be-written".into(),
        generated: true,
        ..Default::default()
    }];
    let stages = vec![adr_stage];
    let plan = stage_case_plan(&stages, "case_m5_2", "run_m5_2", "int_m5_2").unwrap();

    let outcome =
        run_stage_case(root.path(), &policy_path, "operator_local", &plan, stages).unwrap();

    assert_eq!(
        outcome.state, "terminated",
        "a required stage's denial must terminate the case, not silently pass"
    );
    assert_eq!(
        outcome.stages[0].status,
        StageStatus::Quarantined,
        "a non-generator stage declaring a generated-zone output must be denied before authority"
    );
    assert!(outcome.stages[0].quarantine_ref.is_some());
    assert!(outcome.stages[0]
        .settlement_basis
        .iter()
        .any(|b| b.contains("generated_zone_direct_edit")));
}

// ── 3. A quarantined invalid stage (broken prerequisite chain) ──

#[test]
fn invalid_predecessor_chain_quarantines_the_stage() {
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path());
    let mut ast_stage = stage("stage_ast", StageKind::Ast, placeholder_command());
    // Declares an input with no local predecessor AND no self-verified
    // schema — this must be rejected by Task 9's validator before any
    // authority decision or execution occurs.
    ast_stage.inputs = vec![StageFile {
        path: "sea/model.sea".into(),
        sha256: "sha256:aaa".into(),
        ..Default::default()
    }];
    let stages = vec![ast_stage];
    let plan = stage_case_plan(&stages, "case_m5_3", "run_m5_3", "int_m5_3").unwrap();

    let outcome =
        run_stage_case(root.path(), &policy_path, "operator_local", &plan, stages).unwrap();

    assert_eq!(outcome.state, "terminated");
    assert_eq!(outcome.stages[0].status, StageStatus::Quarantined);
    assert!(outcome.stages[0].quarantine_ref.is_some());
    assert!(outcome.stages[0]
        .settlement_basis
        .iter()
        .any(|b| b.contains("prerequisite")));
}

// ── 4. A downstream sentry is blocked by a rejected predecessor ──

#[test]
fn downstream_sentry_blocked_by_rejected_predecessor() {
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path());
    let sea_stage = stage("stage_sea", StageKind::Sea, self_invoke_fail());
    let ast_stage = stage("stage_ast", StageKind::Ast, self_invoke_pass());
    let stages = vec![sea_stage, ast_stage];
    let plan = stage_case_plan(&stages, "case_m5_4", "run_m5_4", "int_m5_4").unwrap();

    let outcome =
        run_stage_case(root.path(), &policy_path, "operator_local", &plan, stages).unwrap();

    assert_eq!(
        outcome.state, "terminated",
        "the case must never falsely complete when a required predecessor was rejected"
    );
    assert_eq!(outcome.stages[0].status, StageStatus::Quarantined);
    assert_eq!(
        outcome.stages[1].status,
        StageStatus::Pending,
        "the downstream stage's settlement-accepted sentry must never fire when its predecessor did not accept"
    );
}
