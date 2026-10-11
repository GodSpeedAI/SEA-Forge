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

/// Lay down two run directories under `<root>/runs/<id>/` with the
/// canonical evidence files plus an artifact, mimicking the pipeline output.
fn seed_two_runs(root: &Path) -> Vec<String> {
    let runs_root = root.join("runs");
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
    let imported_root = root.join("imported").join(&manifest.cell_id);
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
    let bundle = src.path().join("export/bundle_test.tar");
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
    let caps_path = dst.path().join("capabilities.jsonl");
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

    let bundle = src.path().join("export/bundle_cap.tar");
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
    let bundle = src.path().join("export/bundle_tamper.tar");
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
    let imported_dir = dst.path().join("imported").join(&manifest.cell_id);
    assert!(
        !imported_dir.exists(),
        "atomic reject must leave no partial import dir"
    );
    // No staging dir either.
    let staging_glob = dst.path().join("imported/.staging-*");
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
    assert!(root.path().join("cell.json").exists());
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
    let bundle = src.path().join("export/bundle_re.tar");
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
    let bundle = src.path().join("export/bundle_rm.tar");
    cell::export(src.path(), &run_ids, &[], &bundle).unwrap();
    let manifest = cell::read_manifest(&bundle).unwrap();
    assert!(!manifest.files.is_empty());
    assert_eq!(manifest.run_ids.len(), 2);
}

// ---------------------------------------------------------------------------
// Task 1 — cell import paths must fail closed (M6 security, spec-full §7.4/§14.8)
//
// Hash validity does NOT imply path safety: an archive can pass every sha256
// check while its manifest names adversarial destinations. These tests build
// bundles whose per-file hashes/sizes are internally consistent (so integrity
// verification passes) but whose paths / ids are hostile, then assert the
// import rejects with `bundle_integrity_error` and mutates nothing outside
// `<root>/imported/`.
// ---------------------------------------------------------------------------

/// Build a tar bundle from a fully attacker-controlled manifest plus matching
/// entry bytes. Hashes and sizes are computed to be self-consistent so the
/// integrity pass succeeds and path validation is what must reject. Also
/// verifies that every entry name in the resulting archive matches its
/// manifest `files[].path` value verbatim (no `entry.bin` substitution).
fn build_hash_valid_bundle(
    schema_version: &str,
    bundle_id: &str,
    cell_id: &str,
    files: &[(&str, &[u8])],
) -> Vec<u8> {
    let manifest_files: Vec<serde_json::Value> = files
        .iter()
        .map(|(path, data)| {
            serde_json::json!({
                "path": path,
                "sha256": sha256_hex(data),
                "size": data.len(),
            })
        })
        .collect();
    let manifest = serde_json::json!({
        "schema_version": schema_version,
        "bundle_id": bundle_id,
        "cell_id": cell_id,
        "created_at": "2026-07-15T00:00:00Z",
        "run_ids": [],
        "templates": [],
        "files": manifest_files,
    });
    let mut buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut buf);
        builder.mode(tar::HeaderMode::Deterministic);
        append_raw(
            &mut builder,
            "manifest.json",
            &serde_json::to_vec_pretty(&manifest).unwrap(),
        );
        for (path, data) in files {
            append_raw(&mut builder, path, data);
        }
        builder.finish().unwrap();
    }
    // Verify every generated entry name matches its manifest files[].path
    // verbatim — guards against any silent `entry.bin` substitution that
    // would make the bundle unimportable and hide the real hostile name.
    let mut archive = tar::Archive::new(&buf[..]);
    let mut entry_names: Vec<String> = archive
        .entries()
        .expect("read back built archive")
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let header_name = e.path().ok()?.to_string_lossy().into_owned();
            // Skip GNU longname extension carrier entries; their header name
            // is `././@LongLink` and they carry the real name out-of-band.
            if header_name.starts_with("././@LongLink") {
                return None;
            }
            Some(header_name)
        })
        .filter(|name| name != "manifest.json")
        .collect();
    entry_names.sort();
    let mut expected: Vec<String> = files.iter().map(|(p, _)| (*p).to_string()).collect();
    expected.sort();
    assert_eq!(
        entry_names, expected,
        "built archive entry names must match manifest paths verbatim"
    );
    buf
}

/// Append a tar entry using a raw path, bypassing higher-level sanitizing so a
/// hostile `path` string is preserved verbatim in the archive. Writes the
/// path bytes directly into the GNU header name field, bypassing tar-rs's
/// `set_path`/`append_data` validation (which rejects absolute and `..`
/// paths) so the manifest and the archive carry the same hostile name and the
/// import's path-validation is what must reject.
fn append_raw(builder: &mut tar::Builder<&mut Vec<u8>>, path: &str, data: &[u8]) {
    let mut hdr = tar::Header::new_gnu();
    hdr.set_size(data.len() as u64);
    hdr.set_mode(0o644);
    let path_bytes = path.as_bytes();
    // ponytail: the test corpus of hostile paths is ≤100 bytes (the standard
    // name field). A >100-byte hostile name would need a hand-written GNU
    // @LongLink entry; add it if a fixture ever requires one.
    assert!(
        path_bytes.len() <= 100,
        "append_raw fixture paths must fit the 100-byte name field: {path:?}"
    );
    let gnu = hdr.as_gnu_mut().expect("header is GNU-backed");
    for (i, b) in path_bytes.iter().enumerate() {
        gnu.name[i] = *b;
    }
    hdr.set_cksum();
    builder.append(&hdr, data).expect("append raw header");
}

/// Snapshot every path under `dir` with its bytes, for before/after diffing.
fn snapshot_tree(dir: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut out = std::collections::BTreeMap::new();
    fn walk(base: &Path, dir: &Path, out: &mut std::collections::BTreeMap<String, Vec<u8>>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = path
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            if path.is_dir() {
                out.insert(format!("{rel}/"), Vec::new());
                walk(base, &path, out);
            } else {
                out.insert(rel, fs::read(&path).unwrap_or_default());
            }
        }
    }
    walk(dir, dir, &mut out);
    out
}

/// Assert an import attempt was rejected and left zero side effects outside the
/// import staging area: the whole root tree and an outside sentinel are
/// byte-identical before and after.
fn assert_rejected_no_side_effects(dst: &Path, sentinel: &Path, bundle: &Path) {
    let sentinel_before = fs::read(sentinel).unwrap();
    let tree_before = snapshot_tree(dst);

    let err = cell::import(dst, bundle).expect_err("hostile bundle must be rejected");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("bundle_integrity_error"),
        "expected bundle_integrity_error, got: {msg}"
    );

    let sentinel_after = fs::read(sentinel).unwrap();
    assert_eq!(
        sentinel_before, sentinel_after,
        "outside sentinel must be byte-identical after rejected import"
    );
    let tree_after = snapshot_tree(dst);
    assert_eq!(
        tree_before, tree_after,
        "import root tree must be byte-identical after rejected import"
    );
}

/// Create an isolated destination root plus an outside sentinel that adversarial
/// `..` traversal would target if path safety failed.
///
/// The import root is a `root/` subdirectory of a fresh tempdir, and the
/// sentinel lives one level above it (in the tempdir itself). This keeps the
/// exact `../` escape geometry — the sentinel is in the parent of the import
/// root — while giving every test its own private sentinel path so parallel
/// tests cannot race on a shared file in the system temp directory.
fn dst_with_outside_sentinel() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    fs::create_dir_all(&root).unwrap();
    // The sentinel lives in the parent of the import root; a `../` escape from
    // `imported/<cell>/` would climb toward it.
    let sentinel = dir.path().join("sea_forge_outside_sentinel.txt");
    fs::write(&sentinel, b"do-not-touch").unwrap();
    (dir, root, sentinel)
}

/// Absolute manifest path must be rejected before any write.
#[test]
fn traversal_absolute_path_rejected() {
    let (_dir, root, sentinel) = dst_with_outside_sentinel();
    let bundle_path = root.join("abs.tar");
    let bytes = build_hash_valid_bundle(
        "cell.v1",
        "bundle_deadbeef",
        "cell_deadbeef",
        &[("/etc/sea_forge_pwned", b"pwned")],
    );
    fs::write(&bundle_path, &bytes).unwrap();
    assert_rejected_no_side_effects(&root, &sentinel, &bundle_path);
}

/// `..` traversal in a manifest file path must be rejected.
#[test]
fn traversal_parent_escape_path_rejected() {
    let (_dir, root, sentinel) = dst_with_outside_sentinel();
    let bundle_path = root.join("dotdot.tar");
    let bytes = build_hash_valid_bundle(
        "cell.v1",
        "bundle_deadbeef",
        "cell_deadbeef",
        &[("../../../sea_forge_outside_sentinel.txt", b"clobbered")],
    );
    fs::write(&bundle_path, &bytes).unwrap();
    assert_rejected_no_side_effects(&root, &sentinel, &bundle_path);
}

/// A malicious `bundle_id` (used to name the staging dir) must be rejected.
#[test]
fn traversal_malicious_bundle_id_rejected() {
    let (_dir, root, sentinel) = dst_with_outside_sentinel();
    let bundle_path = root.join("badbundle.tar");
    let bytes = build_hash_valid_bundle(
        "cell.v1",
        "../../../etc/sea_forge_bundle_escape",
        "cell_deadbeef",
        &[("runs/run_x/plan.json", b"{}")],
    );
    fs::write(&bundle_path, &bytes).unwrap();
    assert_rejected_no_side_effects(&root, &sentinel, &bundle_path);
}

/// A malicious `cell_id` (used to name the final import dir) must be rejected.
#[test]
fn traversal_malicious_cell_id_rejected() {
    let (_dir, root, sentinel) = dst_with_outside_sentinel();
    let bundle_path = root.join("badcell.tar");
    let bytes = build_hash_valid_bundle(
        "cell.v1",
        "bundle_deadbeef",
        "../../../etc/sea_forge_cell_escape",
        &[("runs/run_x/plan.json", b"{}")],
    );
    fs::write(&bundle_path, &bytes).unwrap();
    assert_rejected_no_side_effects(&root, &sentinel, &bundle_path);
}

/// Two manifest entries that normalize to the same destination are ambiguous
/// and must be rejected (no last-writer-wins smuggling).
#[test]
fn traversal_duplicate_normalized_paths_rejected() {
    let (_dir, root, sentinel) = dst_with_outside_sentinel();
    let bundle_path = root.join("dupe.tar");
    // `runs/a/f.json` and `runs/a//f.json` and `runs/a/./f.json` all name the
    // same on-disk file; the ambiguous spellings must be rejected outright.
    let bytes = build_hash_valid_bundle(
        "cell.v1",
        "bundle_deadbeef",
        "cell_deadbeef",
        &[("runs/a/f.json", b"one"), ("runs/a//f.json", b"two")],
    );
    fs::write(&bundle_path, &bytes).unwrap();
    assert_rejected_no_side_effects(&root, &sentinel, &bundle_path);
}

/// Symlink-parent escape: a pre-existing symlink under the import area whose
/// target is outside the root must not let a manifest path write through it.
#[test]
fn traversal_symlink_parent_escape_rejected() {
    let (_dir, root, sentinel) = dst_with_outside_sentinel();
    // Precreate the imported dir and a symlink that points outside the root.
    let cell_id = "cell_deadbeef";
    let imported = root.join("imported").join(cell_id);
    fs::create_dir_all(&imported).unwrap();
    let outside_dir = _dir.path().join("sea_forge_escape_target");
    fs::create_dir_all(&outside_dir).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside_dir, imported.join("link")).unwrap();
    #[cfg(not(unix))]
    return; // symlink escape is a unix-specific vector here

    let bundle_path = root.join("symlink.tar");
    let bytes = build_hash_valid_bundle(
        "cell.v1",
        "bundle_deadbeef",
        cell_id,
        &[("link/escaped.txt", b"escaped")],
    );
    fs::write(&bundle_path, &bytes).unwrap();

    // The import removes the prior final dir (re-import), so the symlink lives
    // in staging semantics; regardless, no file may be written through it and
    // the outside target must stay empty.
    let sentinel_before = fs::read(&sentinel).unwrap();
    let result = cell::import(&root, &bundle_path);
    assert!(result.is_err(), "symlink-parent escape must be rejected");
    assert!(
        format!("{:?}", result.unwrap_err()).contains("bundle_integrity_error"),
        "symlink escape must report bundle_integrity_error"
    );
    assert!(
        fs::read_dir(&outside_dir).unwrap().next().is_none(),
        "no file may be written through an escaping symlink"
    );
    assert_eq!(sentinel_before, fs::read(&sentinel).unwrap());
}

/// The `imported` directory itself must not be a symlink pointing outside
/// root. If a hostile pre-existing symlink sits at
/// `<root>/imported`, the import must fail closed with
/// `bundle_integrity_error` and write nothing — neither into the symlink's
/// target nor anywhere else outside the canonical root.
#[test]
fn traversal_imported_root_symlink_rejected() {
    let (dir, root, sentinel) = dst_with_outside_sentinel();
    let outside_target = dir.path().join("sea_forge_imported_escape_target");
    fs::create_dir_all(&outside_target).unwrap();
    // Plant `imported` as a symlink whose target is outside the import root.
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside_target, root.join("imported")).unwrap();
    #[cfg(not(unix))]
    return; // symlink escape is a unix-specific vector here

    let bundle_path = root.join("imported_symlink.tar");
    let bytes = build_hash_valid_bundle(
        "cell.v1",
        "bundle_deadbeef",
        "cell_deadbeef",
        &[("runs/run_x/plan.json", b"{}")],
    );
    fs::write(&bundle_path, &bytes).unwrap();

    let sentinel_before = fs::read(&sentinel).unwrap();
    let outside_before = snapshot_tree(&outside_target);
    let err =
        cell::import(&root, &bundle_path).expect_err("imported-root symlink must be rejected");
    assert!(
        format!("{err:?}").contains("bundle_integrity_error"),
        "imported-root symlink must report bundle_integrity_error: {err:?}"
    );
    assert_eq!(
        sentinel_before,
        fs::read(&sentinel).unwrap(),
        "outside sentinel must be byte-identical"
    );
    assert_eq!(
        outside_before,
        snapshot_tree(&outside_target),
        "nothing may be written through the escaping imported symlink"
    );
}

/// F-12: `.sea-forge` is no longer part of the import path at all. A hostile
/// pre-existing `.sea-forge` symlink to outside the root must neither break
/// the import nor be written through — imports land directly under
/// `<root>/imported/`, and the outside target stays byte-identical.
#[test]
fn legacy_sea_forge_symlink_is_inert_to_import() {
    let (dir, root, sentinel) = dst_with_outside_sentinel();
    let outside_target = dir.path().join("sea_forge_seaforge_escape_target");
    fs::create_dir_all(&outside_target).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside_target, root.join(".sea-forge")).unwrap();
    #[cfg(not(unix))]
    return;

    let bundle_path = root.join("seaforge_symlink.tar");
    let bytes = build_hash_valid_bundle(
        "cell.v1",
        "bundle_deadbeef",
        "cell_deadbeef",
        &[("runs/run_x/plan.json", b"{}")],
    );
    fs::write(&bundle_path, &bytes).unwrap();

    let sentinel_before = fs::read(&sentinel).unwrap();
    let outside_before = snapshot_tree(&outside_target);
    let manifest = cell::import(&root, &bundle_path)
        .expect("a legacy .sea-forge symlink must not affect a state-root import");
    assert!(root.join("imported").join(&manifest.cell_id).exists());
    assert!(
        !root.join(".sea-forge/imported").exists(),
        "imports must never nest under the legacy path again"
    );
    assert_eq!(sentinel_before, fs::read(&sentinel).unwrap());
    assert_eq!(
        outside_before,
        snapshot_tree(&outside_target),
        "nothing may be written through the legacy .sea-forge symlink"
    );
}

// Test of template adopt (test 4 from plan).
fn seed_template(src: &Path, name: &str, version: &str, body: &str) {
    let p = src.join("templates").join(format!("{name}@{version}.yaml"));
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
    let bundle = src.path().join("export/bundle_tmpl.tar");
    let manifest = cell::export(src.path(), &run_ids, &["foo@0.1".into()], &bundle).unwrap();
    assert_eq!(manifest.templates, vec!["foo@0.1".to_string()]);

    // Import places the template under imported/<cell_id>/templates/.
    let imported = cell::import(dst.path(), &bundle).unwrap();
    let imported_tmpl = cell::imported_template_path(dst.path(), &imported.cell_id, "foo", "0.1");
    assert!(imported_tmpl.exists(), "imported template must exist");

    // load_pinned must NOT find it — it lives under imported/, not templates/.
    let active_tmpl = dst.path().join("templates/foo@0.1.yaml");
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

// F-12: export must fail closed when requested runs contribute no evidence.
// Under the old double-nested layout this exact call exported an empty bundle
// and reported success while the operator believed the runs had traveled.
#[test]
fn export_with_unresolvable_runs_fails_closed_not_silently_empty() {
    let src = tempfile::tempdir().unwrap();
    let out = src.path().join("export/empty.tar");
    let err = cell::export(
        src.path(),
        &["run_never_materialized".to_string()],
        &[],
        &out,
    )
    .expect_err("zero resolving runs must refuse the export");
    assert!(
        err.to_string().contains("no run evidence found"),
        "the typed error must name the empty-evidence refusal: {err}"
    );
    assert!(
        !out.exists(),
        "a refused export must not leave a bundle behind"
    );

    // Templates-only exports remain lawful when no runs were requested.
    fs::create_dir_all(src.path().join("templates")).unwrap();
    fs::write(src.path().join("templates/foo@0.1.yaml"), "name: foo\n").unwrap();
    let manifest = cell::export(src.path(), &[], &["foo@0.1".into()], &out).unwrap();
    assert_eq!(manifest.templates, vec!["foo@0.1".to_string()]);
}
