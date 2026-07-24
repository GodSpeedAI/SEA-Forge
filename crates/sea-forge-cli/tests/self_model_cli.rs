//! CLI integration for `sea-forge self-model validate|rebuild|show`
//! (spec-adlc-thoth §11.1, M9 slice 1.5b). Drives the built binary end-to-end
//! against a temporary root.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn sea_forge_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sea-forge"))
}

fn run(root: &std::path::Path, action: &str, extra: &[&str]) -> (u8, String) {
    let mut args = vec!["self-model", "--root", root.to_str().unwrap(), action];
    args.extend_from_slice(extra);
    let output = Command::new(sea_forge_bin()).args(&args).output().unwrap();
    let code = output.status.code().unwrap_or(1) as u8;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (code, stdout)
}

fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sea-forge-self-model-{nonce}"));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn rebuild_validate_show_round_trip() {
    let root = temp_root();
    // Rebuild builds the initial snapshot (exit 0).
    let (rc, out) = run(&root, "rebuild", &[]);
    assert_eq!(rc, 0, "rebuild stdout: {out}");
    assert!(out.contains("snapshot_id=smsnap_"));

    // Validate verifies bundled models + snapshot + projections (exit 0).
    let (rc, out) = run(&root, "validate", &[]);
    assert_eq!(rc, 0, "validate stdout: {out}");

    // Show --json emits the full snapshot record.
    let (rc, out) = run(&root, "show", &["--json"]);
    assert_eq!(rc, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["release_id"], "sea-forge@0.1.0");
    assert!(v["snapshot_hash"].as_str().unwrap().starts_with("sha256:"));

    // A second rebuild creates a distinct snapshot; validate stays green.
    let first = v["snapshot_id"].as_str().unwrap().to_string();
    let (rc, _) = run(&root, "rebuild", &[]);
    assert_eq!(rc, 0);
    let (_, out) = run(&root, "show", &[]);
    assert!(
        !out.contains(&first),
        "second rebuild must mint a new snapshot id"
    );
    let (rc, _) = run(&root, "validate", &[]);
    assert_eq!(rc, 0);

    let _ = fs::remove_dir_all(&root);
}

/// Task 11: rebuild ledgers real governance records (authority decision,
/// evidence, settlement, snapshot) under `.sea-forge/ledgers/self-model/`, and
/// every persisted projection record carries non-empty authority/evidence
/// refs plus a settlement ref, resolving `Accepted`.
#[test]
fn rebuild_commits_ledgered_governance_records() {
    let root = temp_root();
    let (rc, _) = run(&root, "rebuild", &[]);
    assert_eq!(rc, 0);

    let entries_path = root.join(".sea-forge/ledgers/self-model/entries.jsonl");
    assert!(
        entries_path.exists(),
        "self-model rebuild must commit to a real ledger stream"
    );
    let text = fs::read_to_string(&entries_path).unwrap();
    let kinds: Vec<serde_json::Value> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    for expected in [
        "authority_decision",
        "self_model_verification_evidence",
        "settlement_event",
        "self_model_snapshot",
        "self_model_release_realization",
        "self_model_cell_realization",
        "self_model_projection",
    ] {
        assert!(
            kinds.iter().any(|e| e["record_kind"] == expected),
            "missing ledgered record_kind {expected}: {kinds:#?}"
        );
    }

    // Every persisted projection record has real, non-empty governance refs.
    let pdir = root.join(".sea-forge/self-model/projections");
    let mut found_any = false;
    for kind_dir in fs::read_dir(&pdir).unwrap() {
        let record_path = kind_dir.unwrap().path().join("projection-record.json");
        if !record_path.exists() {
            continue;
        }
        found_any = true;
        let record: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&record_path).unwrap()).unwrap();
        assert!(
            !record["authority_refs"].as_array().unwrap().is_empty(),
            "authority_refs must be non-empty: {record}"
        );
        assert!(
            !record["evidence_refs"].as_array().unwrap().is_empty(),
            "evidence_refs must be non-empty: {record}"
        );
        assert!(
            record["settlement_ref"].is_string(),
            "settlement_ref must be set: {record}"
        );
        assert_eq!(record["validation"]["status"], "accepted");
    }
    assert!(found_any, "expected at least one projection record");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn tampered_snapshot_is_self_model_error_exit1() {
    let root = temp_root();
    run(&root, "rebuild", &[]);
    // Corrupt the snapshot file on disk.
    let snap_dir = root.join(".sea-forge/self-model/snapshots");
    let snap_file = fs::read_dir(&snap_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(&snap_file, b"{not valid json").unwrap();
    let (rc, _out) = run(&root, "validate", &[]);
    assert_eq!(rc, 1, "corrupt snapshot must fail validation with exit 1");
    let _ = fs::remove_dir_all(&root);
}
