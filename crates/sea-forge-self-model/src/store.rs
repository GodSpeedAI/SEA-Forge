//! Self-model persistence and lifecycle (spec-adlc-thoth §3.1, §7.1, §8.5,
//! slice 1.0 + 1.5).
//!
//! Layout under `<root>/self-model/` (F-12 state-root convention):
//!   manifest.json                     — installation manifest (release, current
//!                                        snapshot id, staleness); the lifecycle
//!                                        discriminator, never an immutable record.
//!   realization/release.json          — release realization (verified, not regenerated).
//!   realization/cell.json             — cell realization (rebuildable).
//!   snapshots/<snapshot_id>.json      — immutable SelfModelSnapshot records.
//!   projections/<kind>/...            — rebuildable KG/CALM/JSON projections.
//!
//! Init/upgrade is detected by the manifest's release_id/schema_version. Upgrade
//! never rewrites prior snapshots. Extension mutations mark the current snapshot
//! stale via the manifest and request a rebuild; immutable snapshot files are
//! never mutated. A failed rebuild preserves the prior verifiable snapshot.

use crate::projections::{project_self, verify_projection, SelfModelProjection};
use crate::{
    build_cell_realization, build_snapshot, bundled, load_composed, realization_sha256,
    release_realization, verify_bundled, verify_snapshot, CellRealization, ComposedModel,
    ExtensionState, ReleaseRealization, SelfModelSnapshot, SnapshotFreshness, ToolchainProbe,
    KERNEL_VERSION, RELEASE_ID,
};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::ids;
use sea_forge_core::types::{
    ActorRole, ActorType, AuditRecord, AuthorityAction, AuthorityDecision, AuthorityRequest,
    BindingResolution, Determinism, IdentityBinding, NormalizedDisposition, SettlementEvent,
    SettlementStatus, Verdict,
};
use sea_forge_ledger::LedgerStream;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Ledger stream id for all self-model governance and realization records
/// (spec-adlc-thoth §7.1/§10.1). One stream per installation root.
const LEDGER_STREAM: &str = "self-model";

const SCHEMA_VERSION: &str = "self_model.v1";

fn dir(root: &Path) -> PathBuf {
    root.join("self-model")
}
fn snapshots_dir(root: &Path) -> PathBuf {
    dir(root).join("snapshots")
}
fn projections_dir(root: &Path) -> PathBuf {
    dir(root).join("projections")
}
fn realization_dir(root: &Path) -> PathBuf {
    dir(root).join("realization")
}
fn manifest_path(root: &Path) -> PathBuf {
    dir(root).join("manifest.json")
}
fn snapshot_path(root: &Path, snapshot_id: &str) -> PathBuf {
    snapshots_dir(root).join(format!("{snapshot_id}.json"))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfModelManifest {
    pub schema_version: String,
    pub release_id: String,
    pub kernel_version: String,
    pub current_snapshot_id: Option<String>,
    #[serde(default)]
    pub stale: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_reason: Option<String>,
    pub initialized_at: String,
}

/// Inputs for a snapshot rebuild. The caller (CLI/server) supplies extension
/// state, environment contracts, and EVIDENCED probe results; the store never
/// executes probes itself (spec-adlc-thoth §7.2, slice 1.4).
#[derive(Clone, Debug, Default)]
pub struct RebuildInputs<'a> {
    pub cell_id: &'a str,
    pub active_extensions: Vec<ExtensionState>,
    pub environments_present: Vec<String>,
    pub probes: Vec<ToolchainProbe>,
    pub sandbox_classes_available: Vec<String>,
    pub created_at: &'a str,
    pub capability_projection_sha256: &'a str,
    /// Identity of the operator/service triggering this rebuild. Recorded on
    /// the governance records committed for this rebuild (Task 11).
    pub actor_id: &'a str,
}

/// Verified inputs behind a rebuild, committed as ledger evidence before the
/// snapshot/projections are minted (Task 11 audit remediation). This is NOT
/// the kernel `EvidenceRecord` type — its closed `EvidenceKind` enum has no
/// variant for "verified installation state" — so self-model records its own
/// small, self-contained evidence shape here instead of forcing a bad fit.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct RebuildVerificationEvidence {
    cell_id: String,
    snapshot_id: String,
    bundled_model_hashes: Vec<String>,
    active_extension_refs: Vec<String>,
    toolchain_probe_results: Vec<String>,
    sandbox_classes_available: Vec<String>,
    capability_projection_sha256: String,
    verified_at: String,
}

fn build_verification_evidence(
    inputs: &RebuildInputs<'_>,
    snapshot: &SelfModelSnapshot,
) -> RebuildVerificationEvidence {
    RebuildVerificationEvidence {
        cell_id: inputs.cell_id.to_string(),
        snapshot_id: snapshot.snapshot_id.clone(),
        bundled_model_hashes: vec![
            snapshot.system_model_ref.semantic_model_sha256.clone(),
            snapshot
                .seed_model_refs
                .first()
                .map(|r| r.semantic_model_sha256.clone())
                .unwrap_or_default(),
        ],
        active_extension_refs: inputs
            .active_extensions
            .iter()
            .map(|e| format!("{}:{}", e.descriptor_ref, e.status))
            .collect(),
        toolchain_probe_results: inputs
            .probes
            .iter()
            .map(|p| format!("{}:{:?}", p.tool, p.result))
            .collect(),
        sandbox_classes_available: inputs.sandbox_classes_available.clone(),
        capability_projection_sha256: inputs.capability_projection_sha256.to_string(),
        verified_at: inputs.created_at.to_string(),
    }
}

/// Construct a well-formed `AuthorityDecision` for a self-model rebuild.
///
/// ponytail: self-model rebuild is an always-available local lifecycle
/// operation over installation state the operator already controls (bundled
/// models, the local extension registry, local probes) — unlike case/run
/// operations it is never gated by an operator-authored policy surface in the
/// spec. Rather than route through `sea_forge_authority::PolicyAuthorityEngine`
/// (which requires a YAML policy bundle and is designed for external-risk
/// operations like executing commands or agent tasks), this builds a real,
/// hash-linked, ledgered `AuthorityDecision` directly: same type, same
/// verifiability, no invented policy-file requirement for a bookkeeping
/// operation the spec never gates. If self-model rebuild ever needs to be
/// denyable, route this through the authority crate instead of adding a
/// second decision shape.
fn build_rebuild_decision(
    actor_id: &str,
    cell_id: &str,
    snapshot_id: &str,
    decided_at: &str,
) -> Result<AuthorityDecision, ForgeError> {
    let operation = AuthorityAction::Reserved {
        resource_type: "self_model_rebuild".into(),
        resource_id: cell_id.into(),
        parameters: serde_json::json!({"snapshot_id": snapshot_id}),
    };
    let identity_binding = IdentityBinding {
        identity_id: None,
        principal: actor_id.into(),
        roles: vec![ActorRole::Operator],
        actor_type: ActorType::System,
        binding_resolution: BindingResolution::LocalDefault,
        identity_binding_source: "self_model_lifecycle".into(),
        source: None,
        sponsor: None,
        issued_at: None,
        expires_at: None,
        identity_binding_hash: None,
    };
    let action_id = ids::random_id("act")?;
    let action_request = AuthorityRequest {
        schema_version: SCHEMA_VERSION.into(),
        action_id: action_id.clone(),
        correlation_id: snapshot_id.into(),
        timestamp_utc: decided_at.into(),
        actor: serde_json::json!({"actor_id": actor_id, "role": "operator"}),
        action: serde_json::to_value(&operation)?,
        context: serde_json::json!({"cell_id": cell_id, "snapshot_id": snapshot_id}),
        evidence: serde_json::json!({}),
    };
    let determinism = Determinism {
        policy_bundle_hash: "self_model.builtin".into(),
        action_request_hash: crate::canonical_sha256(&action_request)?,
        identity_binding_hash: crate::canonical_sha256(&identity_binding)?,
    };
    let audit_record = AuditRecord {
        engine: "sea-forge-self-model".into(),
        disposition: "allow".into(),
        subject: format!("self_model_rebuild:{cell_id}"),
        reason: "self_model_rebuild_lifecycle_operation".into(),
        evidence_refs: vec![],
        recorded_at: decided_at.into(),
        decision_id: None,
        case_id: None,
        run_id: None,
        policy_bundle_hash: None,
        action_request_hash: None,
        identity_binding_hash: None,
    };
    Ok(AuthorityDecision {
        version: KERNEL_VERSION.into(),
        decision_id: ids::random_id("dec")?,
        run_id: snapshot_id.into(),
        plan_item_id: "self_model_rebuild".into(),
        action_id,
        correlation_id: snapshot_id.into(),
        operation,
        outcome: Verdict::Allow,
        verdict: Verdict::Allow,
        normalized_disposition: NormalizedDisposition::Allow,
        matched_rule: None,
        reason_codes: vec!["self_model_lifecycle".into()],
        reason: "self-model rebuild is an always-available local lifecycle operation".into(),
        policy_refs: vec![],
        required_next_steps: vec![],
        identity_binding,
        determinism,
        action_request,
        audit_record,
        decided_at: decided_at.into(),
        candidate_verdicts: vec![],
        winning_source: None,
        precedence_reason: None,
        sandbox_class_granted: None,
        approval_request_id: None,
        opaque_constraint_id: None,
        boundary_constraints: Default::default(),
        compensating_controls: vec![],
    })
}

fn read_manifest(root: &Path) -> Result<Option<SelfModelManifest>, ForgeError> {
    let path = manifest_path(root);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| ForgeError::io("read self-model manifest", e))?;
    let manifest: SelfModelManifest = serde_json::from_slice(&bytes)
        .map_err(|e| ForgeError::SelfModel(format!("self_model_error: bad manifest: {e}")))?;
    Ok(Some(manifest))
}

fn write_manifest(root: &Path, manifest: &SelfModelManifest) -> Result<(), ForgeError> {
    let path = manifest_path(root);
    fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))
        .map_err(|e| ForgeError::io("create self-model dir", e))?;
    let bytes = serde_json::to_vec_pretty(manifest)?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &bytes).map_err(|e| ForgeError::io("write manifest tmp", e))?;
    fs::rename(&tmp, &path).map_err(|e| ForgeError::io("rename manifest", e))?;
    Ok(())
}

/// Read the persisted cell realization (rebuildable projection of registry+probes).
pub fn read_cell_realization(root: &Path) -> Result<Option<CellRealization>, ForgeError> {
    let path = realization_dir(root).join("cell.json");
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| ForgeError::io("read cell realization", e))?;
    let cell: CellRealization = serde_json::from_slice(&bytes).map_err(|e| {
        ForgeError::SelfModel(format!("self_model_error: bad cell realization: {e}"))
    })?;
    Ok(Some(cell))
}

pub fn read_release_realization(root: &Path) -> Result<Option<ReleaseRealization>, ForgeError> {
    let path = realization_dir(root).join("release.json");
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| ForgeError::io("read release realization", e))?;
    let r: ReleaseRealization = serde_json::from_slice(&bytes).map_err(|e| {
        ForgeError::SelfModel(format!("self_model_error: bad release realization: {e}"))
    })?;
    Ok(Some(r))
}

/// Ledger the projection records + materialize their view files. Each
/// `ProjectionRecord` is committed under `self_model_projection` (spec-adlc-thoth
/// §10.1: "Persisted ONLY under the `self_model_projection` ledger record_kind"),
/// then its own record and output bytes are materialized as rebuildable views.
fn materialize_projections(
    root: &Path,
    stream: &LedgerStream,
    projections: &[SelfModelProjection],
) -> Result<(), ForgeError> {
    let pdir = projections_dir(root);
    // Clear prior projection outputs so a wrong-hash pre-generated file is
    // regenerated, not trusted (spec §10.1, T9.3).
    if pdir.exists() {
        fs::remove_dir_all(&pdir).map_err(|e| ForgeError::io("clear projections dir", e))?;
    }
    fs::create_dir_all(&pdir).map_err(|e| ForgeError::io("create projections dir", e))?;
    for proj in projections {
        let kind_label = match proj.record.projection_kind {
            sea_forge_core::types::ProjectionKind::Kg => "kg",
            sea_forge_core::types::ProjectionKind::Calm => "calm",
            sea_forge_core::types::ProjectionKind::SelfModelSnapshot => "self_model_snapshot",
            _ => continue,
        };
        let kind_dir = pdir.join(kind_label);
        for (path, content) in &proj.outputs {
            let full = pdir.join(path);
            stream.materialize_view(
                &stream.commit_typed(
                    "self_model_projection_output",
                    vec![proj.record.projection_id.clone()],
                    content,
                    proj.record.authority_refs.clone(),
                )?,
                &full,
                content.as_bytes(),
            )?;
        }
        let record_committed = stream.commit_typed(
            "self_model_projection",
            vec![proj.record.projection_id.clone()],
            &proj.record,
            proj.record.authority_refs.clone(),
        )?;
        stream.materialize_view(
            &record_committed,
            &kind_dir.join("projection-record.json"),
            &serde_json::to_vec_pretty(&proj.record)?,
        )?;
    }
    Ok(())
}

fn assemble(
    inputs: &RebuildInputs<'_>,
) -> Result<(ComposedModel, SelfModelSnapshot, CellRealization), ForgeError> {
    let models = bundled();
    verify_bundled(&models)?;
    let composed = load_composed(&models)?;
    let release = release_realization();
    let cell = build_cell_realization(
        inputs.cell_id,
        inputs.active_extensions.clone(),
        inputs.environments_present.clone(),
        inputs.probes.clone(),
        inputs.sandbox_classes_available.clone(),
        inputs.created_at,
    );
    let snapshot = build_snapshot(
        &composed,
        &release,
        &cell,
        inputs.capability_projection_sha256,
        inputs.created_at,
    )?;
    Ok((composed, snapshot, cell))
}

/// Build a fresh snapshot and commit it: assembles the pure snapshot/cell from
/// verified sources, commits validation evidence, an authority decision, and a
/// settlement to the self-model ledger stream, threads those governance refs
/// into every `ProjectionRecord`, then materializes all views. On any failure
/// the prior snapshot remains verifiable (failure atomicity) — nothing is
/// written until the snapshot has been built and independently re-verified.
pub fn rebuild(root: &Path, inputs: &RebuildInputs<'_>) -> Result<SelfModelSnapshot, ForgeError> {
    fs::create_dir_all(snapshots_dir(root))
        .map_err(|e| ForgeError::io("create snapshots dir", e))?;
    let (composed, snapshot, cell) = assemble(inputs)?;
    // Integrity gate before any commit: a snapshot that fails to re-verify its
    // own hash is never ledgered or settled accepted.
    verify_snapshot(&snapshot)?;

    let stream = LedgerStream::open(root, LEDGER_STREAM, inputs.actor_id)?;

    let evidence = build_verification_evidence(inputs, &snapshot);
    let evidence_committed = stream.commit_typed(
        "self_model_verification_evidence",
        vec![snapshot.snapshot_id.clone()],
        &evidence,
        vec![],
    )?;

    let decision = build_rebuild_decision(
        inputs.actor_id,
        inputs.cell_id,
        &snapshot.snapshot_id,
        inputs.created_at,
    )?;
    let authority_committed = stream.commit_typed(
        "authority_decision",
        vec![snapshot.snapshot_id.clone()],
        &decision,
        vec![evidence_committed.entry_ulid().to_string()],
    )?;

    let settlement = SettlementEvent {
        version: KERNEL_VERSION.into(),
        settlement_id: ids::random_id("set")?,
        run_id: snapshot.snapshot_id.clone(),
        status: SettlementStatus::Accepted,
        basis: vec!["self_model_snapshot_verified".into()],
        review_required: false,
        settled_at: inputs.created_at.to_string(),
        criteria_ref: None,
    };
    let settlement_committed = stream.commit_typed(
        "settlement_event",
        vec![snapshot.snapshot_id.clone()],
        &settlement,
        vec![authority_committed.entry_ulid().to_string()],
    )?;

    let authority_refs = vec![authority_committed.entry_ulid().to_string()];
    let evidence_refs = vec![evidence_committed.entry_ulid().to_string()];
    let settlement_ref = Some(settlement_committed.entry_ulid().to_string());

    let projections = project_self(
        &composed,
        &snapshot,
        inputs.created_at,
        authority_refs.clone(),
        evidence_refs,
        settlement_ref,
    )?;

    // Commit order: projections, then release/cell realization, then the
    // immutable snapshot, then the manifest pointer. A failure before the
    // manifest write leaves the prior current snapshot authoritative.
    materialize_projections(root, &stream, &projections)?;

    let release = release_realization();
    // SUP-09f: pin the idempotency key to the realization's canonical
    // *content*, not the constant release id. The payload legitimately varies
    // between build environments (`generated_at` honors SOURCE_DATE_EPOCH) and
    // across realization edits; keying on the constant turned every such
    // difference into an unrecoverable idempotency-conflict error, while
    // keying on content makes identical realizations dedupe (same key, same
    // bytes → existing record returned) and differing ones commit cleanly.
    let release_committed = stream.commit_typed_once(
        "self_model_release_realization",
        realization_sha256(&release)?,
        vec![],
        &release,
        vec![],
    )?;
    stream.materialize_view(
        &release_committed,
        &realization_dir(root).join("release.json"),
        &serde_json::to_vec_pretty(&release)?,
    )?;

    let cell_committed = stream.commit_typed(
        "self_model_cell_realization",
        vec![snapshot.snapshot_id.clone()],
        &cell,
        authority_refs.clone(),
    )?;
    stream.materialize_view(
        &cell_committed,
        &realization_dir(root).join("cell.json"),
        &serde_json::to_vec_pretty(&cell)?,
    )?;

    let snapshot_committed = stream.commit_typed_new(
        "self_model_snapshot",
        snapshot.snapshot_id.clone(),
        vec![snapshot.snapshot_id.clone()],
        &snapshot,
        authority_refs,
    )?;
    stream.materialize_view(
        &snapshot_committed,
        &snapshot_path(root, &snapshot.snapshot_id),
        &serde_json::to_vec_pretty(&snapshot)?,
    )?;

    let manifest = SelfModelManifest {
        schema_version: SCHEMA_VERSION.into(),
        release_id: RELEASE_ID.into(),
        kernel_version: KERNEL_VERSION.into(),
        current_snapshot_id: Some(snapshot.snapshot_id.clone()),
        stale: false,
        stale_reason: None,
        initialized_at: inputs.created_at.to_string(),
    };
    write_manifest(root, &manifest)?;
    Ok(snapshot)
}

/// First-root initialization: if no manifest exists, build the initial snapshot.
/// If a manifest exists for the same release, return the current snapshot. A
/// release-id change is an upgrade (builds a new snapshot; prior ones remain).
pub fn ensure_init(
    root: &Path,
    inputs: &RebuildInputs<'_>,
) -> Result<SelfModelSnapshot, ForgeError> {
    if let Some(manifest) = read_manifest(root)? {
        if manifest.release_id == RELEASE_ID
            && manifest.schema_version == SCHEMA_VERSION
            && manifest.current_snapshot_id.is_some()
        {
            return current_snapshot(root)?.ok_or_else(|| {
                ForgeError::SelfModel(
                    "self_model_error: manifest names a missing current snapshot".into(),
                )
            });
        }
        // Release/schema change ⇒ upgrade: rebuild (never rewrite prior snapshots).
    }
    rebuild(root, inputs)
}

/// The current (newest committed) snapshot, or None if uninitialized.
pub fn current_snapshot(root: &Path) -> Result<Option<SelfModelSnapshot>, ForgeError> {
    let Some(manifest) = read_manifest(root)? else {
        return Ok(None);
    };
    let Some(sid) = manifest.current_snapshot_id else {
        return Ok(None);
    };
    // The manifest is persisted data and therefore untrusted; a traversal-shaped
    // snapshot id must fail closed rather than escape the snapshots directory
    // (SUP-09g).
    if !sea_forge_core::path::valid_id_segment(&sid, 128) {
        return Err(ForgeError::SelfModel(format!(
            "self_model_error: unsafe current_snapshot_id in manifest: {sid}"
        )));
    }
    let path = snapshot_path(root, &sid);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| ForgeError::io("read snapshot", e))?;
    let snap: SelfModelSnapshot = serde_json::from_slice(&bytes).map_err(|e| {
        ForgeError::SelfModel(format!("self_model_error: bad snapshot record: {e}"))
    })?;
    Ok(Some(snap))
}

/// Mark the current snapshot stale (e.g. after an extension mutation) without
/// mutating any immutable snapshot file. Records the reason in the manifest.
pub fn mark_current_stale(root: &Path, reason: &str) -> Result<(), ForgeError> {
    let Some(mut manifest) = read_manifest(root)? else {
        return Err(ForgeError::SelfModel(
            "self_model_error: cannot mark stale before init".into(),
        ));
    };
    manifest.stale = true;
    manifest.stale_reason = Some(reason.to_string());
    write_manifest(root, &manifest)?;
    Ok(())
}

/// Whether the current snapshot is marked stale.
pub fn is_stale(root: &Path) -> Result<bool, ForgeError> {
    Ok(read_manifest(root)?.map(|m| m.stale).unwrap_or(false))
}

/// Verify bundled models, validate the composed model, verify the self-model
/// ledger's hash chain (when one has been committed), verify the current
/// snapshot's hash, and verify each persisted projection's rebuild_hash.
pub fn validate(root: &Path) -> Result<(), ForgeError> {
    let models = bundled();
    verify_bundled(&models)?;
    let _composed = load_composed(&models)?;
    let ledger_dir = root.join("ledgers").join(LEDGER_STREAM);
    if ledger_dir.exists() {
        LedgerStream::open(root, LEDGER_STREAM, "validator")?.verify()?;
    }
    if let Some(snap) = current_snapshot(root)? {
        verify_snapshot(&snap)?;
        if snap.freshness == SnapshotFreshness::Stale {
            // Stale is allowed but disclosed; not a validation failure.
        }
    }
    // Verify persisted projections.
    let pdir = projections_dir(root);
    if pdir.exists() {
        for entry in fs::read_dir(&pdir).map_err(|e| ForgeError::io("read projections dir", e))? {
            let entry = entry.map_err(|e| ForgeError::io("projection dir entry", e))?;
            let record_path = entry.path().join("projection-record.json");
            if record_path.exists() {
                let bytes = fs::read(&record_path)
                    .map_err(|e| ForgeError::io("read projection record", e))?;
                let record: sea_forge_core::types::ProjectionRecord =
                    serde_json::from_slice(&bytes).map_err(|e| {
                        ForgeError::SelfModel(format!(
                            "self_model_error: bad projection record: {e}"
                        ))
                    })?;
                // SUP-04: validation hashes the materialized outputs against
                // the record's refs — a replaced or corrupted view file is a
                // validation failure, not a clean pass.
                verify_projection(&record, &pdir)?;
            }
        }
    }
    Ok(())
}
