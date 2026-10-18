//! SEC-09 export boundary: a federation bundle carries each run's
//! `artifacts/`, which an isolated workload writes after receiving classified
//! plaintext. The export's authority request states the union of its runs'
//! exposure labels, so once `export_bundle` is an evaluable reserved surface
//! a rule must permit every label (`exposure_permitted`).
//!
//! Today the engine does not list `export_bundle` among its reserved
//! surfaces, so every export is denied as unclassified whatever the policy
//! says: the channel is closed. This test pins both facts.

use sea_forge_ledger::exposure::ExposureRegistry;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: allow-confidential-export\n    verdict: allow\n    actor_role: operator\n    operation_kind: export_bundle\n    boundary_constraints:\n      permitted_exposure: [confidential]\n";

fn run(root: &Path, run_id: &str, case_id: &str) {
    let dir = root.join("runs").join(run_id);
    fs::create_dir_all(dir.join("artifacts")).unwrap();
    fs::write(
        dir.join("plan.json"),
        serde_json::to_vec(&serde_json::json!({
            "version": "1", "plan_id": format!("plan-{run_id}"), "case_id": case_id,
            "run_id": run_id, "intent_id": "intent-1", "items": [],
        }))
        .unwrap(),
    )
    .unwrap();
    // What a workload could have written after exposure.
    fs::write(dir.join("artifacts/out.txt"), format!("{case_id} data")).unwrap();
}

fn export(root: &Path, run_ids: &[&str], out: &Path) -> bool {
    let policy = root.join("policy.yaml");
    fs::write(&policy, POLICY).unwrap();
    Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .arg("export")
        .arg("--root")
        .arg(root)
        .arg("--policy")
        .arg(&policy)
        .arg("--out")
        .arg(out)
        .arg("--run-ids")
        .args(run_ids)
        .output()
        .unwrap()
        .status
        .success()
}

/// The `exposure` parameter of every recorded export authority request.
fn requested_exposure(root: &Path) -> Vec<Value> {
    let mut seen = Vec::new();
    for entry in fs::read_dir(root.join("ledgers/authority-reads")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "jsonl") {
            continue;
        }
        for line in fs::read_to_string(&path).unwrap().lines() {
            let record: Value = serde_json::from_str(line).unwrap();
            if record["record_kind"] == "authority_evidence" {
                seen.push(record["payload"]["action"]["parameters"]["exposure"].clone());
            }
        }
    }
    seen
}

#[test]
fn exports_state_their_exposure_and_the_channel_is_closed() {
    let base = tempfile::tempdir().unwrap();
    let root = base.path().join("cell");
    run(&root, "r-clean", "c-clean");
    run(&root, "r-conf", "c-conf");
    run(&root, "r-child", "c-child");
    let registry = ExposureRegistry::open(&root, "test-observer").unwrap();
    registry
        .record("case:c-conf", "confidential", "mcp:records/read", "ev-1")
        .unwrap();
    registry.derive("case:c-conf", "case:c-child").unwrap();

    for (name, runs) in [
        ("clean.bundle", &["r-clean"][..]),
        ("conf.bundle", &["r-conf"][..]),
        ("mixed.bundle", &["r-clean", "r-conf"][..]),
    ] {
        let out = base.path().join(name);
        assert!(!export(&root, runs, &out), "{name} exported");
        assert!(!out.exists(), "{name} written despite refusal");
    }
    // A delegated child's run carries its ancestor's label.
    assert!(!export(
        &root,
        &["r-child"],
        &base.path().join("child.bundle")
    ));
    assert_eq!(
        requested_exposure(&root),
        [
            serde_json::json!([]),
            serde_json::json!(["confidential"]),
            serde_json::json!(["confidential"]),
            serde_json::json!(["confidential"]),
        ]
    );

    // A run whose case cannot be established never reaches authority.
    fs::create_dir_all(root.join("runs/r-noplan/artifacts")).unwrap();
    let before = requested_exposure(&root).len();
    for (name, runs) in [
        ("noplan.bundle", &["r-noplan"][..]),
        ("traversal.bundle", &["../cell/runs/r-conf"][..]),
    ] {
        let out = base.path().join(name);
        assert!(!export(&root, runs, &out), "{name} exported");
        assert!(!out.exists());
    }
    assert_eq!(requested_exposure(&root).len(), before);
}
