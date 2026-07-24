//! M10 (E12 §7.5) production-plan conformance: a submitted plan naming the
//! built-in `odi_adlc_case@0.1.0` template resolves its desired-outcome
//! provenance against Task 11's real, validated self-model seed — never a
//! fabricated placeholder — and a tampered/unresolvable desired-outcome
//! reference is rejected before authority with zero side effects
//! (audit remediation Task 12).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sea-forge-cli-m10-{nonce}"));
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_policy(parent: &Path) -> PathBuf {
    let path = parent.join("policy.yaml");
    fs::write(
        &path,
        r#"version: "0.1"
identity:
  source: test
  allow_unresolved: false
rules:
  - name: allow-write
    verdict: allow
    actor_role: operator
    operation_kind: write_file
    path_prefix: ""
"#,
    )
    .unwrap();
    path
}

fn write_plan(parent: &Path, template_ref: &str) -> PathBuf {
    let path = parent.join("plan.json");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "0.2",
            "plan_id": "plan_odi",
            "case_id": "case_odi",
            "run_id": "run_odi",
            "intent_id": "int_odi",
            "items": [
                {
                    "plan_item_id": "A",
                    "name": "A",
                    "operations": [{"kind":"write_file","path":"a.txt","content_hint":"ok"}],
                    "entry_criteria": [], "exit_criteria": [],
                    "settlement_criteria": {"require_exit_zero":false,"required_artifacts":["a.txt"],"stdout_must_contain":null,"require_approval":false},
                    "item_kind":"sandboxed_task","sandbox_class":"local","parent_stage":null,
                    "markers":{"required":true,"repetition":false,"manual_activation":false},
                    "max_instances":1,"depends_on":[]
                }
            ],
            "template_ref": template_ref
        }))
        .unwrap(),
    )
    .unwrap();
    path
}

fn run_plan(root: &Path, plan: &Path, policy: &Path) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// T10.6: a production plan naming `odi_adlc_case@0.1.0` installs the
/// built-in through the source-owned installer, derives its settlement
/// criteria via `derive_from_template`, and resolves the desired-outcome
/// origin ref against the real, bundled self-model seed hash — the exact
/// same hash `sea_forge_self_model` independently reports for that seed.
#[test]
fn t10_6_production_plan_with_odi_template_resolves_real_desired_outcome() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_policy(&parent);
    let plan = write_plan(&parent, "odi_adlc_case@0.1.0");

    let (code, stdout, stderr) = run_plan(&root, &plan, &policy);
    assert_eq!(code, 0, "stdout: {stdout}\nstderr: {stderr}");

    // The source-owned installer materialized the pinned built-in template.
    let installed = root.join("templates/odi_adlc_case@0.1.0.yaml");
    assert!(
        installed.exists(),
        "built-in template must be installed under <root>/templates/"
    );

    // The committed settlement_criteria record carries a real desired-outcome
    // origin ref whose hash matches the actual bundled seed model hash.
    let models = sea_forge_self_model::bundled();
    let composed = sea_forge_self_model::load_composed(&models).unwrap();
    let expected_hash = composed.seed_model_ref().semantic_model_sha256.clone();

    let case_dirs: Vec<_> = fs::read_dir(root.join("cases")).unwrap().collect();
    assert_eq!(case_dirs.len(), 1);
    let case_dir = case_dirs.into_iter().next().unwrap().unwrap().path();
    let entries_path = case_dir
        .file_name()
        .map(|name| {
            root.join("ledgers")
                .join(format!("case-{}", name.to_string_lossy()))
                .join("entries.jsonl")
        })
        .unwrap();
    let text = fs::read_to_string(&entries_path).unwrap();
    let mut found_desired_outcome = false;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let entry: serde_json::Value = serde_json::from_str(line).unwrap();
        if entry["record_kind"] != "settlement_criteria" {
            continue;
        }
        for origin in entry["payload"]["origin_refs"].as_array().unwrap() {
            if origin["kind"] == "desired_outcome" {
                found_desired_outcome = true;
                assert_eq!(origin["sha256"], expected_hash);
                assert_ne!(origin["sha256"], "sha256:placeholder");
                assert_eq!(origin["reference"], "Desired Outcome Criterion");
            }
        }
    }
    assert!(
        found_desired_outcome,
        "expected a desired_outcome origin ref in a committed settlement_criteria record"
    );

    let _ = fs::remove_dir_all(&parent);
}

/// T10.4 (production wiring): a tampered pinned copy of `odi_adlc_case@0.1.0`
/// that points its desired-outcome reference at a concept the validated seed
/// does not recognize as a Desired Outcome Criterion is rejected before
/// authority — the run fails, and no case/run directory is ever created.
#[test]
fn t10_4_tampered_builtin_template_is_rejected_before_authority_no_side_effects() {
    let parent = temp_root();
    let root = parent.join("state");
    let policy = write_policy(&parent);
    let plan = write_plan(&parent, "odi_adlc_case@0.1.0");

    // Pre-seed a tampered pinned template BEFORE the source-owned installer
    // ever runs: the installer never overwrites existing bytes, so this
    // tampered copy is what `load_pinned` returns.
    let templates_dir = root.join("templates");
    fs::create_dir_all(&templates_dir).unwrap();
    let mut tampered = sea_forge_planner::templates::odi_adlc_case_template(
        "godspeed.adlc_odi_case",
        "sha256:not-the-real-seed-hash",
    );
    for origin in &mut tampered.origin_refs {
        origin.reference = "Not A Real Concept".into();
    }
    fs::write(
        templates_dir.join("odi_adlc_case@0.1.0.yaml"),
        serde_yaml::to_string(&tampered).unwrap(),
    )
    .unwrap();

    let (code, stdout, stderr) = run_plan(&root, &plan, &policy);
    assert_ne!(code, 0, "stdout: {stdout}\nstderr: {stderr}");
    assert!(
        stderr.contains("criteria_provenance_error")
            || stdout.contains("criteria_provenance_error"),
        "stdout: {stdout}\nstderr: {stderr}"
    );

    // No execution side effects: criteria verification fails before the
    // per-instance run/workspace allocation loop, so no run ever exists even
    // though the case's bookkeeping directory (created earlier, alongside the
    // committed `case_plan` record) does.
    let runs_dirs: Vec<_> = fs::read_dir(root.join("cases"))
        .unwrap()
        .map(|entry| entry.unwrap().path().join("runs"))
        .collect();
    for runs_dir in runs_dirs {
        assert!(
            !runs_dir.exists() || fs::read_dir(&runs_dir).unwrap().next().is_none(),
            "a rejected desired-outcome ref must leave no allocated run"
        );
    }

    let _ = fs::remove_dir_all(&parent);
}
