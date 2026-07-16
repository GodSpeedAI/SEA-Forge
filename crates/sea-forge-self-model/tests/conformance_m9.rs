//! M9 (E11) core conformance — Genesis Self-Model (spec-adlc-thoth §17.1).
//!
//! T9.3–T9.5 (projections, extension-disable rebuild, cell probes/V5) are added
//! by later M9 slices; this file covers the model/snapshot foundation (T9.1,
//! T9.2) that slice 1.2b+1.3 delivers.

use sea_forge_self_model::{
    build_snapshot, bundled, load_composed, release_realization, verify_bundled, verify_snapshot,
    CellRealization,
};

/// T9.1: bundled models verify hashes and pass load_validate; a snapshot is
/// creatable with all five digest fields populated.
#[test]
fn t91_bundled_models_verify_load_and_snapshot_has_all_digests() {
    let models = bundled();
    verify_bundled(&models).expect("bundled model hashes must match pinned constants");

    let composed = load_composed(&models).expect("both bundled models must validate");
    assert!(!composed.system_model_ref().semantic_model_sha256.is_empty());
    assert!(!composed.seed_model_ref().semantic_model_sha256.is_empty());

    let realization = release_realization();
    let cell = CellRealization {
        cell_id: "cell_00000000".into(),
        active_extensions: vec![],
        environments_present: vec![],
        toolchain_probes: vec![],
        sandbox_classes_available: vec!["local".into()],
        degraded_components: vec![],
        probed_at: "2026-07-16T00:00:00Z".into(),
    };
    let snap = build_snapshot(
        &composed,
        &realization,
        &cell,
        "sha256:capability-projection",
        "2026-07-16T00:00:00Z",
    )
    .expect("snapshot must build from verified sources");

    // All five digest fields populated (spec §7.1).
    assert!(!snap.release_realization_sha256.is_empty());
    assert!(!snap.cell_realization_sha256.is_empty());
    assert!(!snap.capability_projection_sha256.is_empty());
    assert!(!snap.snapshot_hash.is_empty());
    assert!(!snap.system_model_ref.semantic_model_sha256.is_empty());
    assert!(snap.snapshot_id.starts_with("smsnap_"));
    verify_snapshot(&snap).expect("snapshot hash must verify");
}

/// T9.2: a one-byte model tamper surfaces as `self_model_error` BEFORE
/// validation. Blast radius: self-model consumers only (runs do not invoke the
/// self-model crate, so they are structurally unaffected).
#[test]
fn t92_tampered_model_is_self_model_error_before_validation() {
    let mut tampered = bundled();
    let mut bad: Vec<u8> = tampered.system.bytes.to_vec();
    let last = bad.len() - 1;
    bad[last] = if bad[last] == b'a' { b'b' } else { b'a' };
    tampered.system.bytes = Box::leak(bad.into_boxed_slice());

    let err = verify_bundled(&tampered).expect_err("hash drift must be rejected");
    assert_eq!(err.class(), "self_model_error");
    assert!(err.to_string().contains("hash drift"));

    // Validation is never reached: load_composed calls verify_bundled first.
    let err2 = load_composed(&tampered).expect_err("load must refuse drifted bytes");
    assert_eq!(err2.class(), "self_model_error");
}
