//! M9 (E11) core conformance — Genesis Self-Model (spec-adlc-thoth §17.1).
//!
//! T9.3–T9.5 (projections, extension-disable rebuild, cell probes/V5) are added
//! by later M9 slices; this file covers the model/snapshot foundation (T9.1,
//! T9.2) that slice 1.2b+1.3 delivers.

use sea_forge_self_model::{
    build_cell_realization, build_snapshot, bundled, load_composed, release_realization,
    verify_bundled, verify_snapshot, CellRealization, ProbeResult, ToolchainProbe,
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

/// T9.5: a cell realization with a failing toolchain probe records `unavailable`
/// plus the probe evidence ref; the tool is listed under degraded_components.
#[test]
fn t95_failing_probe_records_unavailable_and_evidence() {
    let cell = build_cell_realization(
        "cell_00000000",
        vec![],
        vec!["demo_env@0.1.0".into()],
        vec![ToolchainProbe {
            tool: "tla2tools".into(),
            required_by: "projection_target:kg".into(),
            probe_command_ref: "env:demo_env:command:java -version".into(),
            result: ProbeResult::Unavailable,
            evidence_ref: Some("evi_probe_tla_0001".into()),
        }],
        vec!["local".into()],
        "2026-07-16T00:00:00Z",
    );
    let probe = cell
        .toolchain_probes
        .iter()
        .find(|p| p.tool == "tla2tools")
        .expect("failing probe must be recorded");
    assert_eq!(probe.result, ProbeResult::Unavailable);
    assert_eq!(probe.evidence_ref.as_deref(), Some("evi_probe_tla_0001"));
    assert!(
        cell.degraded_components.contains(&"tla2tools".to_string()),
        "unavailable tool must be a degraded component: {:?}",
        cell.degraded_components
    );
    // A missing probe (no entry) is not fabricated; it simply caps later.
    assert!(cell.toolchain_probes.len() == 1);
}

/// V5: probe flap — a toolchain removed then re-added across two snapshots
/// tracks fresh evidence and is never cached. Snapshot 1 sees the tool
/// unavailable (degraded); snapshot 2 (rebuilt from fresh Available evidence)
/// sees it available and not degraded. The crate holds no probe state between
/// builds, so availability follows the evidence passed to each call.
#[test]
fn v5_probe_flap_availability_follows_fresh_evidence() {
    let composed = load_composed(&bundled()).unwrap();
    let realization = release_realization();

    let probe_unavailable = vec![ToolchainProbe {
        tool: "cargo-tarpaulin".into(),
        required_by: "env:demo_env".into(),
        probe_command_ref: "command:cargo-tarpaulin --version".into(),
        result: ProbeResult::Unavailable,
        evidence_ref: Some("evi_flap_0001".into()),
    }];
    let cell1 = build_cell_realization(
        "cell_00000000",
        vec![],
        vec![],
        probe_unavailable,
        vec!["local".into()],
        "2026-07-16T00:00:00Z",
    );
    let snap1 = build_snapshot(
        &composed,
        &realization,
        &cell1,
        "sha256:cap1",
        "2026-07-16T00:00:00Z",
    )
    .unwrap();
    assert!(cell1
        .degraded_components
        .contains(&"cargo-tarpaulin".to_string()));

    // Re-add: fresh evidence says Available. No cached 'unavailable' carries over.
    let probe_available = vec![ToolchainProbe {
        tool: "cargo-tarpaulin".into(),
        required_by: "env:demo_env".into(),
        probe_command_ref: "command:cargo-tarpaulin --version".into(),
        result: ProbeResult::Available,
        evidence_ref: Some("evi_flap_0002".into()),
    }];
    let cell2 = build_cell_realization(
        "cell_00000000",
        vec![],
        vec![],
        probe_available,
        vec!["local".into()],
        "2026-07-16T00:01:00Z",
    );
    let snap2 = build_snapshot(
        &composed,
        &realization,
        &cell2,
        "sha256:cap2",
        "2026-07-16T00:01:00Z",
    )
    .unwrap();

    assert!(
        !cell2
            .degraded_components
            .contains(&"cargo-tarpaulin".to_string()),
        "re-added tool must NOT remain degraded: {:?}",
        cell2.degraded_components
    );
    assert_ne!(snap1.snapshot_hash, snap2.snapshot_hash);
    verify_snapshot(&snap1).unwrap();
    verify_snapshot(&snap2).unwrap();
}
