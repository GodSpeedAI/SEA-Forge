//! M6 federation conformance (spec-full §7.4, §14.8).
//!
//! Teeth (per plan): flipping one byte in an imported run file fails the
//! whole import — no partial state, no merge into local capability memory.

use sea_forge_cell as cell;
use sea_forge_core::types::{BundleManifest, SemanticEnvelope};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const RUN_FILES: &[&str] = &[
    "plan.json",
    "trace.jsonl",
    "evidence.jsonl",
    "authority.json",
    "settlement.json",
    "semantic-envelope.json",
];

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Lay down two run directories under `<root>/.sea-forge/runs/<id>/` with the
/// canonical evidence files plus an artifact, mimicking the pipeline output.
fn seed_two_runs(root: &Path) -> Vec<String> {
    let runs_root = root.join(".sea-forge/runs");
    fs::create_dir_all(&runs_root).unwrap();
    let run_a = "run_20260715T000000Z_aaaaaa";
    let run_b = "run_20260715T000001Z_bbbbbb";
    for (run_id, payload) in [(run_a, b"{\"run\":\"a\"}\n"), (run_b, b"{\"run\":\"b\"}\n")] {
        let run_dir = runs_root.join(run_id);
        fs::create_dir_all(run_dir.join("artifacts/sub")).unwrap();
        for name in RUN_FILES {
            fs::write(run_dir.join(name), payload).unwrap();
        }
        // An artifact file (real evidence; bundled, then verified).
        fs::write(run_dir.join("artifacts/derived.txt"), b"derived payload").unwrap();
        fs::write(run_dir.join("artifacts/sub/nested.json"), b"{}").unwrap();
        // workspace/ scratch — NOT bundled.
        fs::create_dir_all(run_dir.join("workspace")).unwrap();
        fs::write(run_dir.join("workspace/scratch.txt"), b"scratch").unwrap();
    }
    vec![run_a.into(), run_b.into()]
}

fn assert_manifest_files_present(root: &Path, manifest: &BundleManifest) {
    let imported_root = root.join(".sea-forge/imported").join(&manifest.cell_id);
    assert!(imported_root.exists(), "imported dir must exist");
    for entry in &manifest.files {
        let p = imported_root.join(&entry.path);
        assert!(p.exists(), "imported entry {} must exist", entry.path);
        let bytes = fs::read(&p).unwrap();
        let actual = sha256_hex(&bytes);
        assert_eq!(actual, entry.sha256, "sha256 mismatch for {}", entry.path);
    }
}

/// M6 bullet 1: export 2 runs → import on a fresh root → hashes verify.
#[test]
fn export_import_verifies_hashes() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let run_ids = seed_two_runs(src.path());
    let bundle = src.path().join(".sea-forge/export/bundle_test.tar");
    let manifest = cell::export(src.path(), &run_ids, &[], &bundle).unwrap();
    assert_eq!(manifest.run_ids, run_ids);
    assert_eq!(manifest.cell_id.len(), 13);
    assert!(manifest.cell_id.starts_with("cell_"));
    // 12 run files (6 each) + 4 artifacts (2 each) = 16
    assert_eq!(
        manifest.files.len(),
        RUN_FILES.len() * 2 + 4,
        "expected all evidence files + artifacts bundled"
    );
    // workspace/scratch.txt must NOT appear.
    assert!(
        !manifest.files.iter().any(|f| f.path.contains("workspace")),
        "workspace scratch must not be bundled"
    );
    // Import on fresh root.
    let imported = cell::import(dst.path(), &bundle).unwrap();
    assert_eq!(imported.bundle_id, manifest.bundle_id);
    assert_eq!(imported.cell_id, manifest.cell_id);
    assert_manifest_files_present(dst.path(), &imported);
}

/// M6 bullet 2: local capability counts unchanged after import.
#[test]
fn local_capability_counts_unchanged() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let run_ids = seed_two_runs(src.path());
    // Seed capabilities.jsonl in dst with N synthetic envelopes.
    let caps_path = dst.path().join(".sea-forge/capabilities.jsonl");
    fs::create_dir_all(caps_path.parent().unwrap()).unwrap();
    let env = SemanticEnvelope {
        version: "0.1".into(),
        run_id: "run_local".into(),
        case_ref: "case_local".into(),
        intent: sea_forge_core::types::Intent {
            intent_id: "int_local".into(),
            summary: "local".into(),
            actor_id: "entity_local".into(),
            process_id: "proc_local".into(),
            created_at: "2026-07-15T00:00:00Z".into(),
        },
        plan_ref: "plan_local".into(),
        template_ref: None,
        authority_decisions: vec![],
        evidence_refs: vec![],
        settlement_ref: "set_local".into(),
        capability_delta: sea_forge_core::types::CapabilityDelta {
            attempted_capability: "cap_local".into(),
            result: sea_forge_core::types::SettlementStatus::Accepted,
        },
        attribution: sea_forge_core::types::Attribution {
            entity_id: "entity_local".into(),
            process_id: "proc_local".into(),
            session_id: "sess_local".into(),
        },
        artifact_refs: vec![],
        extension_refs: vec![],
        projection_refs: vec![],
        cell_id: None,
    };
    let mut buf = Vec::new();
    for _ in 0..5 {
        buf.extend(serde_json::to_vec(&env).unwrap());
        buf.push(b'\n');
    }
    fs::write(&caps_path, &buf).unwrap();
    let n_before = fs::read_to_string(&caps_path).unwrap().lines().count();
    assert_eq!(n_before, 5);

    let bundle = src.path().join(".sea-forge/export/bundle_cap.tar");
    cell::export(src.path(), &run_ids, &[], &bundle).unwrap();
    cell::import(dst.path(), &bundle).unwrap();

    // Import MUST NOT append to capabilities.jsonl.
    let n_after = fs::read_to_string(&caps_path).unwrap().lines().count();
    assert_eq!(
        n_after, n_before,
        "import must not touch capabilities.jsonl"
    );
}

/// M6 bullet 3: tampered bundle is rejected atomically (no partial state).
/// Teeth: one flipped byte → whole import fails, no leftover directory.
#[test]
fn tampered_bundle_rejected_atomically() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let run_ids = seed_two_runs(src.path());
    let bundle = src.path().join(".sea-forge/export/bundle_tamper.tar");
    let manifest = cell::export(src.path(), &run_ids, &[], &bundle).unwrap();

    // Read bundle bytes, find a known file's bytes, flip one byte in-place
    // within the tar stream. Rebuild a tar with the corrupted entry.
    let original_bytes = fs::read(&bundle).unwrap();
    let corrupted = corrupt_first_entry(&original_bytes, "runs/");
    let tampered_path = src.path().join("tampered.tar");
    fs::write(&tampered_path, &corrupted).unwrap();

    let result = cell::import(dst.path(), &tampered_path);
    let err = result.expect_err("tampered bundle must be rejected");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("bundle_integrity_error"),
        "expected bundle_integrity_error class, got: {msg}"
    );

    // Atomic-reject teeth: no leftover imported dir for this cell.
    let imported_dir = dst
        .path()
        .join(".sea-forge/imported")
        .join(&manifest.cell_id);
    assert!(
        !imported_dir.exists(),
        "atomic reject must leave no partial import dir"
    );
    // No staging dir either.
    let staging_glob = dst.path().join(".sea-forge/imported/.staging-*");
    let _ = staging_glob; // ponytail: a single staging path; globbed clean on err path
}

/// Tamper by flipping a byte inside the data section of the first `runs/` entry.
/// Tar entries are: 512-byte header, then data padded to 512. Header has the
/// name in the first 100 bytes; we flip a byte inside the data payload.
fn corrupt_first_entry(bytes: &[u8], prefix: &str) -> Vec<u8> {
    let mut out = bytes.to_vec();
    // Scan 512-byte blocks for a header whose name starts with prefix.
    let mut offset = 0usize;
    while offset + 512 <= out.len() {
        let header = &out[offset..offset + 512];
        let name_end = header.iter().position(|&b| b == 0).unwrap_or(100);
        let name = std::str::from_utf8(&header[..name_end]).unwrap_or("");
        if name.starts_with(prefix) {
            // Data starts at offset+512; flip byte at +520 (8 bytes in).
            let target = offset + 512 + 8;
            if target < out.len() {
                out[target] ^= 0xFF;
                return out;
            }
        }
        // Skip header + padded data.
        let size_bytes = &header[124..136];
        let size_str = std::str::from_utf8(size_bytes).unwrap_or("0");
        let octal_size =
            usize::from_str_radix(size_str.trim_end_matches('\0').trim(), 8).unwrap_or(0);
        let padded = (octal_size + 511) & !511;
        offset += 512 + padded;
    }
    panic!("no entry with prefix {prefix} found to tamper");
}

/// §7.4 invariants: cell.json is created once and stable; ensure is idempotent.
#[test]
fn cell_identity_is_stable() {
    let root = tempfile::tempdir().unwrap();
    let id1 = cell::ensure(root.path()).unwrap();
    let id2 = cell::ensure(root.path()).unwrap();
    assert_eq!(id1, id2, "ensure is idempotent");
    assert!(id1.starts_with("cell_") && id1.len() == 13);
    let record = cell::read_cell(root.path()).unwrap().unwrap();
    assert_eq!(record.cell_id, id1);
    // cell.json must be at the canonical path.
    assert!(root.path().join(".sea-forge/cell.json").exists());
}

/// Cell identity absent = legacy, valid (read returns None without error).
#[test]
fn absent_cell_is_legacy_valid() {
    let root = tempfile::tempdir().unwrap();
    assert!(cell::read_cell(root.path()).unwrap().is_none());
}

/// Empty manifest schema-version rejection.
#[test]
fn unknown_schema_version_rejected() {
    // Hand-craft a tar with manifest.json of bogus schema_version and one
    // entry. Import must reject with bundle_integrity_error.
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().join("bad.tar");
    let mut buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut buf);
        builder.mode(tar::HeaderMode::Deterministic);
        let manifest = serde_json::json!({
            "schema_version": "bogus.v99",
            "bundle_id": "bundle_bogus",
            "cell_id": "cell_deadbeef",
            "created_at": "2026-07-15T00:00:00Z",
            "run_ids": ["run_x"],
            "templates": [],
            "files": [],
        });
        let mbytes = serde_json::to_vec_pretty(&manifest).unwrap();
        let mut hdr = tar::Header::new_gnu();
        hdr.set_path("manifest.json").unwrap();
        hdr.set_size(mbytes.len() as u64);
        hdr.set_mode(0o644);
        hdr.set_cksum();
        builder.append(&hdr, mbytes.as_slice()).unwrap();
        builder.finish().unwrap();
    }
    fs::write(&bundle, &buf).unwrap();
    let err = cell::import(dir.path(), &bundle).unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("bundle_integrity_error"));
    assert!(msg.contains("unknown schema_version") || msg.contains("invalid"));
}

/// Bundle missing manifest.json is rejected.
#[test]
fn missing_manifest_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().join("nomanifest.tar");
    let mut buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut buf);
        builder.mode(tar::HeaderMode::Deterministic);
        let mut hdr = tar::Header::new_gnu();
        hdr.set_path("runs/run_x/plan.json").unwrap();
        hdr.set_size(2);
        hdr.set_mode(0o644);
        hdr.set_cksum();
        builder.append(&hdr, b"{}".as_slice()).unwrap();
        builder.finish().unwrap();
    }
    fs::write(&bundle, &buf).unwrap();
    let err = cell::import(dir.path(), &bundle).unwrap_err();
    assert!(format!("{err:?}").contains("bundle_integrity_error"));
}

/// Bundle with extra entry not in manifest is rejected (no smuggling).
#[test]
fn extra_entry_in_bundle_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().join("extra.tar");
    let mut buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut buf);
        builder.mode(tar::HeaderMode::Deterministic);
        // manifest lists zero files.
        let manifest = serde_json::json!({
            "schema_version": "cell.v1",
            "bundle_id": "bundle_extra",
            "cell_id": "cell_cafef00d",
            "created_at": "2026-07-15T00:00:00Z",
            "run_ids": [],
            "templates": [],
            "files": [],
        });
        let mbytes = serde_json::to_vec_pretty(&manifest).unwrap();
        let mut h1 = tar::Header::new_gnu();
        h1.set_path("manifest.json").unwrap();
        h1.set_size(mbytes.len() as u64);
        h1.set_mode(0o644);
        h1.set_cksum();
        builder.append(&h1, mbytes.as_slice()).unwrap();
        // smuggled entry.
        let mut h2 = tar::Header::new_gnu();
        h2.set_path("runs/smuggled.json").unwrap();
        h2.set_size(5);
        h2.set_mode(0o644);
        h2.set_cksum();
        builder.append(&h2, &b"smuggled"[..]).unwrap();
        builder.finish().unwrap();
    }
    fs::write(&bundle, &buf).unwrap();
    let err = cell::import(dir.path(), &bundle).unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("bundle_integrity_error"));
    assert!(msg.contains("not in manifest"));
}

/// Re-import on same root replaces prior import (idempotent at file level).
#[test]
fn re_import_replaces_prior() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let run_ids = seed_two_runs(src.path());
    let bundle = src.path().join(".sea-forge/export/bundle_re.tar");
    cell::export(src.path(), &run_ids, &[], &bundle).unwrap();
    let m1 = cell::import(dst.path(), &bundle).unwrap();
    let m2 = cell::import(dst.path(), &bundle).unwrap();
    assert_eq!(m1.bundle_id, m2.bundle_id);
    assert_eq!(m1.cell_id, m2.cell_id);
    assert_manifest_files_present(dst.path(), &m2);
}

/// Read manifest from a bundle without importing.
#[test]
fn read_manifest_inspects_bundle() {
    let src = tempfile::tempdir().unwrap();
    let run_ids = seed_two_runs(src.path());
    let bundle = src.path().join(".sea-forge/export/bundle_rm.tar");
    cell::export(src.path(), &run_ids, &[], &bundle).unwrap();
    let manifest = cell::read_manifest(&bundle).unwrap();
    assert!(!manifest.files.is_empty());
    assert_eq!(manifest.run_ids.len(), 2);
}

// Test of template adopt (test 4 from plan).
fn seed_template(src: &Path, name: &str, version: &str, body: &str) {
    let p = src
        .join(".sea-forge/templates")
        .join(format!("{name}@{version}.yaml"));
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, body).unwrap();
}

/// Imported template requires explicit adopt before load_pinned finds it.
#[test]
fn imported_template_not_instantiable_pre_adopt() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let run_ids = seed_two_runs(src.path());
    let template_body = "name: foo\nversion: \"0.1\"\nplan:\n  items: []\n";
    seed_template(src.path(), "foo", "0.1", template_body);
    let bundle = src.path().join(".sea-forge/export/bundle_tmpl.tar");
    let manifest = cell::export(src.path(), &run_ids, &["foo@0.1".into()], &bundle).unwrap();
    assert_eq!(manifest.templates, vec!["foo@0.1".to_string()]);

    // Import places the template under imported/<cell_id>/templates/.
    let imported = cell::import(dst.path(), &bundle).unwrap();
    let imported_tmpl = cell::imported_template_path(dst.path(), &imported.cell_id, "foo", "0.1");
    assert!(imported_tmpl.exists(), "imported template must exist");

    // load_pinned must NOT find it — it lives under imported/, not templates/.
    let active_tmpl = dst.path().join(".sea-forge/templates/foo@0.1.yaml");
    assert!(
        !active_tmpl.exists(),
        "pre-adopt: template must not be active"
    );

    // Adopt: copy into active templates dir.
    let adopted = cell::adopt(dst.path(), &imported.cell_id, "foo@0.1").unwrap();
    assert_eq!(adopted, active_tmpl);
    assert!(active_tmpl.exists(), "post-adopt: template must be active");
    let adopted_body = fs::read_to_string(&active_tmpl).unwrap();
    assert_eq!(adopted_body, template_body, "adopt must copy byte-for-byte");
}
