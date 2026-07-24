//! M5 (E5) spec-to-code pipeline CLI integration — spec-audit-remediation
//! Task 10B. See `.agents/specs/spec-full.md` §10.7, §12 M5 proofs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "sea-forge-m5-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn cli() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sea-forge"))
}

fn write_policy(root: &Path) -> PathBuf {
    let path = root.join("policy.yaml");
    fs::write(
        &path,
        "version: \"0.1\"\nrules:\n  - name: allow-stage-execute\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    sandbox_class: jail\n",
    )
    .unwrap();
    path
}

const DEMO_SEA: &str = "@namespace \"demo\"\n@version \"1.0.0\"\n\nEntity \"Sample\" in demo\nResource \"Artifact\" units in demo\nFlow \"Artifact\" from \"Sample\" to \"Sample\" quantity 1\n";

fn write_entry(root: &Path) -> PathBuf {
    let path = root.join("model.sea");
    fs::write(&path, DEMO_SEA).unwrap();
    path
}

fn run_project(root: &Path, policy: &Path, entry: &Path) -> std::process::Output {
    Command::new(cli())
        .args([
            "project",
            entry.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ])
        .output()
        .unwrap()
}

fn stdout_field<'a>(stdout: &'a str, key: &str) -> &'a str {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("missing {key}= in stdout:\n{stdout}"))
}

// ── 1. End-to-end: ADR→PRD→SDS→SEA→AST/IR→manifest→generated-contract ──

#[test]
fn cli_project_runs_full_m5_chain_and_settles_independent_projections() {
    let root = temp_root("chain");
    let policy = write_policy(&root);
    let entry = write_entry(&root);

    let output = run_project(&root, &policy, &entry);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_eq!(
        output.status.code(),
        Some(0),
        "project must succeed for a valid model:\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let case_id = stdout_field(&stdout, "case_id");
    assert_eq!(stdout_field(&stdout, "case_state"), "completed");
    assert_eq!(
        stdout_field(&stdout, "proof_classification"),
        "GeneratedContract",
        "no last-mile/runtime/acceptance stages were run, so the ceiling is generated-contract"
    );
    assert_eq!(stdout_field(&stdout, "projections_accepted"), "2");
    assert_eq!(stdout_field(&stdout, "projections_quarantined"), "0");

    // Ledger links: the case's ledger stream contains the case plan, stage
    // authority decisions/settlements, the spec_pipeline_run record, and
    // independent projection_record/settlement_event pairs.
    let ledger_dir = root.join("ledgers").join(format!("case-{case_id}"));
    assert!(ledger_dir.is_dir(), "case ledger stream must exist");
    let mut kinds = std::collections::BTreeSet::new();
    for entry in fs::read_dir(&ledger_dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            let content = fs::read_to_string(&path).unwrap();
            for line in content.lines() {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(kind) = value.get("record_kind").and_then(|v| v.as_str()) {
                        kinds.insert(kind.to_string());
                    }
                }
            }
        }
    }
    for expected in [
        "case_plan",
        "authority_decision",
        "settlement_event",
        "spec_pipeline_run",
        "projection_record",
    ] {
        assert!(
            kinds.contains(expected),
            "expected ledger record kind {expected} in {kinds:?}"
        );
    }

    // No DomainForge-owned filesystem side effects: DomainForge writes
    // nothing itself (§10.4a). Every file under root traces to SEA Forge's
    // own authorized writes (ledgers/, cases/, spec-pipelines/, policy.yaml,
    // model.sea).
    let mut top_level: Vec<String> = fs::read_dir(&root)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    top_level.sort();
    for name in &top_level {
        assert!(
            matches!(
                name.as_str(),
                "ledgers" | "cases" | "spec-pipelines" | "policy.yaml" | "model.sea"
            ),
            "unexpected top-level entry {name} suggests an uncontrolled side effect"
        );
    }
}

// ── 2. Negative proof: an unsupported projection kind quarantines, never silently drops ──

#[test]
fn cli_project_quarantines_unsupported_projection_kind_without_accepting_or_dropping() {
    let root = temp_root("bad-projection");
    let policy = write_policy(&root);
    let entry = write_entry(&root);

    let output = Command::new(cli())
        .args([
            "project",
            entry.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
            "--projection",
            "sbvr",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    assert_ne!(
        output.status.code(),
        Some(0),
        "an unsupported projection kind must not report overall success:\nstdout: {stdout}"
    );
    assert_eq!(stdout_field(&stdout, "projections_accepted"), "0");
    assert_eq!(
        stdout_field(&stdout, "projections_quarantined"),
        "1",
        "the failed projection must be recorded as quarantined, not silently dropped"
    );

    let case_id = stdout_field(&stdout, "case_id");
    let ledger_dir = root.join("ledgers").join(format!("case-{case_id}"));
    let mut found_rejected_projection = false;
    for entry in fs::read_dir(&ledger_dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            let content = fs::read_to_string(&path).unwrap();
            for line in content.lines() {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                    if value.get("record_kind").and_then(|v| v.as_str())
                        == Some("projection_record")
                    {
                        if let Some(payload) = value.get("payload") {
                            if payload.get("projection_kind").and_then(|v| v.as_str())
                                == Some("sbvr")
                            {
                                assert_eq!(
                                    payload["validation"]["status"], "rejected",
                                    "the quarantined projection record must carry rejected status, not be omitted"
                                );
                                found_rejected_projection = true;
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(
        found_rejected_projection,
        "expected a rejected sbvr projection_record in the ledger, found none"
    );
}
