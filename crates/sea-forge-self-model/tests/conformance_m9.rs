//! M9 (E11) core conformance — Genesis Self-Model (spec-adlc-thoth §17.1).
//!
//! T9.3–T9.5 (projections, extension-disable rebuild, cell probes/V5) are added
//! by later M9 slices; this file covers the model/snapshot foundation (T9.1,
//! T9.2) that slice 1.2b+1.3 delivers.

use sea_forge_self_model::{
    build_cell_realization, build_snapshot, bundled, load_composed, release_realization, store,
    verify_bundled, verify_snapshot, CellRealization, ExtensionState, ProbeResult, ToolchainProbe,
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

/// T9.3: KG/CALM/JSON self-projections rebuild byte-identically (modulo
/// created_at) and carry full ProjectionRecord provenance. A pre-generated
/// projection whose rebuild_hash no longer matches its bytes is rejected, and a
/// rebuild regenerates trusted output over it.
#[test]
fn t93_projections_rebuild_deterministically_and_reject_drift() {
    use sea_forge_self_model::projections::{project_self, verify_projection};

    let composed = load_composed(&bundled()).unwrap();
    let realization = release_realization();
    let cell = build_cell_realization(
        "cell_00000000",
        vec![],
        vec![],
        vec![],
        vec!["local".into()],
        "2026-07-16T00:00:00Z",
    );
    let snap = build_snapshot(
        &composed,
        &realization,
        &cell,
        "sha256:cap",
        "2026-07-16T00:00:00Z",
    )
    .unwrap();

    let p1 = project_self(&composed, &snap, "2026-07-16T00:00:00Z").unwrap();
    let p2 = project_self(&composed, &snap, "2026-07-16T00:01:00Z").unwrap();

    assert_eq!(p1.len(), 3, "KG, CALM, self_model_snapshot");
    let kinds: Vec<_> = p1
        .iter()
        .map(|p| p.record.projection_kind.clone())
        .collect();
    assert!(kinds
        .iter()
        .any(|k| matches!(k, sea_forge_core::types::ProjectionKind::Kg)));
    assert!(kinds
        .iter()
        .any(|k| matches!(k, sea_forge_core::types::ProjectionKind::SelfModelSnapshot)));

    for (a, b) in p1.iter().zip(p2.iter()) {
        // Same projection id (deterministic), same rebuild_hash, same output bytes.
        assert_eq!(a.record.projection_id, b.record.projection_id);
        assert_eq!(a.record.rebuild_hash, b.record.rebuild_hash);
        assert_eq!(
            a.outputs, b.outputs,
            "output bytes must be byte-identical across rebuilds"
        );
        // created_at differs (the only allowed divergence).
        assert_ne!(a.record.created_at, b.record.created_at);
        // Full provenance present.
        assert!(!a.record.output_refs.is_empty());
        assert!(!a.record.rebuild_hash.is_empty());
        assert_eq!(a.record.adapter_ref, "sea-forge-self-model");
        verify_projection(&a.record).unwrap();
    }

    // A pre-generated projection whose rebuild_hash drifts is rejected, not trusted.
    let mut tampered = p1[0].record.clone();
    tampered.rebuild_hash = "sha256:deadbeef".into();
    let err = verify_projection(&tampered).unwrap_err();
    assert_eq!(err.class(), "self_model_error");
}

/// T9.4: disabling an extension then rebuild produces a new snapshot; the cell
/// realization reflects the disabled state; the prior snapshot remains verifiable.
/// Init/upgrade/rebuild flow through the store; failure atomicity preserves the
/// prior snapshot. (spec §7.1, §8.5, slice 1.0+1.5)
#[test]
fn t94_extension_disable_rebuild_keeps_prior_snapshot() {
    let root = tempfile::tempdir().unwrap();
    let ext = ExtensionState {
        descriptor_ref: "ext:E11-Genesis-Self-Model".into(),
        status: "active".into(),
    };
    let inputs_active = store::RebuildInputs {
        cell_id: "cell_00000000",
        active_extensions: vec![ext.clone()],
        environments_present: vec!["demo_env@0.1.0".into()],
        probes: vec![],
        sandbox_classes_available: vec!["local".into()],
        created_at: "2026-07-16T00:00:00Z",
        capability_projection_sha256: "sha256:cap1",
    };
    let snap1 = store::ensure_init(root.path(), &inputs_active).unwrap();
    assert!(!store::is_stale(root.path()).unwrap());
    store::validate(root.path()).unwrap();

    // Disable the extension: mark stale, then rebuild with disabled state.
    store::mark_current_stale(root.path(), "extension_disabled").unwrap();
    assert!(store::is_stale(root.path()).unwrap());

    let mut disabled = ext.clone();
    disabled.status = "disabled_by_policy".into();
    let inputs_disabled = store::RebuildInputs {
        cell_id: "cell_00000000",
        active_extensions: vec![disabled],
        created_at: "2026-07-16T00:01:00Z",
        capability_projection_sha256: "sha256:cap2",
        ..inputs_active.clone()
    };
    let snap2 = store::rebuild(root.path(), &inputs_disabled).unwrap();
    assert_ne!(snap1.snapshot_id, snap2.snapshot_id);
    assert!(!store::is_stale(root.path()).unwrap());

    // Both snapshots still verify; the prior one remains on disk and readable.
    verify_snapshot(&snap1).unwrap();
    verify_snapshot(&snap2).unwrap();
    assert_eq!(
        store::current_snapshot(root.path())
            .unwrap()
            .unwrap()
            .snapshot_id,
        snap2.snapshot_id
    );

    // Cell realization reflects the disabled state after rebuild.
    let cell = store::read_cell_realization(root.path()).unwrap().unwrap();
    let recorded = cell
        .active_extensions
        .iter()
        .find(|e| e.descriptor_ref == "ext:E11-Genesis-Self-Model")
        .expect("extension must be present in cell realization");
    assert_eq!(recorded.status, "disabled_by_policy");

    // The prior snapshot file is still present (immutable, not rewritten).
    let prior_path = root
        .path()
        .join(".sea-forge/self-model/snapshots")
        .join(format!("{}.json", snap1.snapshot_id));
    assert!(prior_path.exists(), "prior snapshot must remain on disk");

    store::validate(root.path()).unwrap();
}

/// Init idempotency: ensure_init on an already-initialized root returns the
/// current snapshot without rebuilding (slice 1.0 lifecycle ingress).
#[test]
fn init_is_idempotent_for_same_release() {
    let root = tempfile::tempdir().unwrap();
    let inputs = store::RebuildInputs {
        cell_id: "cell_00000000",
        sandbox_classes_available: vec!["local".into()],
        created_at: "2026-07-16T00:00:00Z",
        capability_projection_sha256: "sha256:cap",
        ..Default::default()
    };
    let snap1 = store::ensure_init(root.path(), &inputs).unwrap();
    let snap2 = store::ensure_init(root.path(), &inputs).unwrap();
    assert_eq!(
        snap1.snapshot_id, snap2.snapshot_id,
        "same release must not rebuild"
    );
}
