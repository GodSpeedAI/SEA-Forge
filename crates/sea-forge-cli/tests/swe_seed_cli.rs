//! SUP-09c: the hidden `internal-test-swe-seed` arm must cap its stdin read
//! instead of buffering an unbounded request in the shipped binary.

use std::{
    io::Write,
    process::{Command, Stdio},
};

fn run_with_stdin(payload: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args(["internal-test-swe-seed"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        // The child stops reading at the cap and exits, so a broken pipe on
        // the remaining bytes is expected and not part of what is asserted.
        let _ = stdin.write_all(payload);
    }
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

#[test]
fn swe_seed_rejects_stdin_larger_than_one_mebibyte() {
    let over_cap = vec![b'x'; 1_048_576 + 1];
    let output = run_with_stdin(&over_cap);
    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("exceeds 1 MiB cap"), "stderr: {stderr}");
}

#[test]
fn swe_seed_accepts_small_valid_declaration_request() {
    let request = serde_json::json!({
        "settlement_ref": "settlement:test",
        "run_id": "run-1",
        "case_id": "case-1",
        "plan_item_id": "item-1",
        "claim_manifest_sha256": "sha256:0",
        "criteria_ref": "criteria:test",
        "criteria_sha256": "sha256:0",
        "criteria_record_hash": "record:0",
        "criteria_declared_at": "2026-01-01T00:00:00Z",
        "execution_started_at": "2026-01-01T00:00:00Z",
        "origin_refs": [],
        "verifier_ref": "verifier:test",
        "verifier_sha256": "sha256:0",
        "acting_entity_id": "operator_local",
        "requested_strength": "local",
        "declarer": {
            "actor_id": "operator_local",
            "authority_ref": "authority:test",
            "role": "operator",
            "standing_basis": "test"
        },
        "variation_tags": {},
        "disruption_tags": [],
        "source_evidence_refs": []
    });
    let output = run_with_stdin(request.to_string().as_bytes());
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("attestation:test-swe-seed"),
        "stdout: {stdout}"
    );
}
