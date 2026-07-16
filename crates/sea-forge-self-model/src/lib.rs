//! SEA Forge Genesis self-model (spec-adlc-thoth E11, M9).
//!
//! Bundled canonical `.sea` models, deterministic release realization,
//! installation cell realization, and immutable self-model snapshots with
//! canonical hash linking. Synchronous kernel crate: no async, no network.
//!
//! Public contract:
//!   - `bundled()` → the release-owned model bytes + pinned sha256 constants.
//!   - `verify_bundled()` → reject byte drift before validation (self_model_error).
//!   - `load_composed()` → validate both models through DomainForge.
//!   - `release_realization()` → deterministic release realization (SOURCE_DATE_EPOCH).
//!   - snapshot assembly + canonical `snapshot_hash`.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::ids;
use sea_forge_domainforge::{load_validate, DomainModel, DomainModelRef, SeaSourceSet, SourceFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;

// ── Release-owned model assets (source of truth under models/) ───────────────

/// Bundled bytes of `models/seaforge-system@0.1.0.sea`.
const SYSTEM_MODEL_BYTES: &[u8] = include_bytes!("../../../models/seaforge-system@0.1.0.sea");
/// Bundled bytes of `models/adlc-odi-case@0.1.0.sea`.
const SEED_MODEL_BYTES: &[u8] = include_bytes!("../../../models/adlc-odi-case@0.1.0.sea");

/// Pinned release sha256 of the canonical system model (spec-adlc-thoth §10.1).
/// Drift here is `self_model_error`, never a warning.
pub const SYSTEM_MODEL_SHA256: &str =
    "sha256:09ead9ac9514009d60de0da18d936b82aa6a94a86db94c00dde7f3da54b77cfa";
/// Pinned release sha256 of the ADLC/ODI methodology seed model.
pub const SEED_MODEL_SHA256: &str =
    "sha256:8c891cff33fe7bb012ebb0fda6232bc8e996d5a1af47e9c5a675f2cc7a143613";

/// Release identifier for this build of SEA Forge.
pub const RELEASE_ID: &str = "sea-forge@0.1.0";
/// Kernel record-schema version (from `sea-forge-core::RECORD_VERSION`).
pub const KERNEL_VERSION: &str = "0.1";

/// Fallback epoch (seconds) for `generated_at` when SOURCE_DATE_EPOCH is unset.
/// Keeps the release realization deterministic across non-reproduced builds.
const RELEASE_EPOCH_SECONDS: i64 = 1_752_635_200; // 2026-07-16T00:00:00Z

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Canonical JSON: object keys sorted lexicographically, strings NFC-normalized,
/// arrays ordered. Matches the ledger's `jcs-nfc-v1` profile so domain hashes
/// agree with payload hashes on the same content shape.
fn canonical_json(value: &serde_json::Value) -> Result<Vec<u8>, ForgeError> {
    fn sorted(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut entries: Vec<_> = map.into_iter().collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                let map: serde_json::Map<String, serde_json::Value> = entries.into_iter().collect();
                serde_json::Value::Object(map)
            }
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.into_iter().map(sorted).collect())
            }
            serde_json::Value::String(s) => serde_json::Value::String(s.chars().nfc().collect()),
            other => other,
        }
    }
    let sorted = sorted(value.clone());
    serde_json::to_vec(&sorted).map_err(|e| ForgeError::Serialization(e.to_string()))
}

/// Domain-separated canonical sha256 over a serializable value.
pub fn canonical_sha256<T: Serialize>(value: &T) -> Result<String, ForgeError> {
    let v = serde_json::to_value(value)?;
    let bytes = canonical_json(&v)?;
    Ok(format!("sha256:{:x}", Sha256::digest(&bytes)))
}

// ── Bundled models ───────────────────────────────────────────────────────────

/// The release-owned bundled models with their pinned hashes.
#[derive(Clone, Debug)]
pub struct BundledModels {
    pub system: ModelAsset,
    pub seed: ModelAsset,
}

#[derive(Clone, Debug)]
pub struct ModelAsset {
    pub uri: String,
    pub bytes: &'static [u8],
    pub pinned_sha256: &'static str,
}

/// Return the bundled release-owned models with pinned hashes.
pub fn bundled() -> BundledModels {
    BundledModels {
        system: ModelAsset {
            uri: "models/seaforge-system@0.1.0.sea".into(),
            bytes: SYSTEM_MODEL_BYTES,
            pinned_sha256: SYSTEM_MODEL_SHA256,
        },
        seed: ModelAsset {
            uri: "models/adlc-odi-case@0.1.0.sea".into(),
            bytes: SEED_MODEL_BYTES,
            pinned_sha256: SEED_MODEL_SHA256,
        },
    }
}

/// Verify each bundled model's bytes against its pinned release hash.
/// Mismatch is `self_model_error` and MUST occur before `load_validate`.
pub fn verify_bundled(models: &BundledModels) -> Result<(), ForgeError> {
    for asset in [&models.system, &models.seed] {
        let computed = format!("sha256:{}", sha256_hex(asset.bytes));
        if computed != asset.pinned_sha256 {
            return Err(ForgeError::SelfModel(format!(
                "self_model_error: bundled model hash drift for {}: declared={} computed={computed}",
                asset.uri, asset.pinned_sha256
            )));
        }
    }
    Ok(())
}

fn load_one(asset: &ModelAsset) -> Result<DomainModel, ForgeError> {
    let content = std::str::from_utf8(asset.bytes).map_err(|e| {
        ForgeError::SelfModel(format!(
            "self_model_error: bundled model {} is not utf-8: {e}",
            asset.uri
        ))
    })?;
    let declared = asset
        .pinned_sha256
        .strip_prefix("sha256:")
        .unwrap_or(asset.pinned_sha256);
    let source_set = SeaSourceSet {
        entry_uri: asset.uri.clone(),
        files: vec![SourceFile {
            uri: asset.uri.clone(),
            sha256: declared.to_string(),
            content: content.to_string(),
        }],
    };
    load_validate(&source_set).map_err(|e| ForgeError::SelfModel(e.to_string()))
}

// ── Composed model + typed read-only concept lookup ──────────────────────────

/// Validated, composed view over the system model and the methodology seed.
/// Composition is multi-model (spec-adlc-thoth §6.2/§7.0a): the system model is
/// the entry; the seed is an overlaid namespace. Concept lookup never exposes a
/// raw graph handle.
#[derive(Clone, Debug)]
pub struct ComposedModel {
    pub system: DomainModel,
    pub seed: DomainModel,
}

impl ComposedModel {
    pub fn system_model_ref(&self) -> &DomainModelRef {
        &self.system.model_ref
    }
    pub fn seed_model_ref(&self) -> &DomainModelRef {
        &self.seed.model_ref
    }

    /// Whether a concept name is declared in either composed source.
    pub fn concept_exists(&self, name: &str) -> bool {
        self.system.model_ref.concept_refs.iter().any(|c| c == name)
            || self.seed.model_ref.concept_refs.iter().any(|c| c == name)
    }

    /// Sorted, de-duplicated concept names across both composed sources.
    pub fn concepts(&self) -> Vec<String> {
        let mut all: Vec<String> = self
            .system
            .model_ref
            .concept_refs
            .iter()
            .chain(self.seed.model_ref.concept_refs.iter())
            .cloned()
            .collect();
        all.sort();
        all.dedup();
        all
    }
}

/// Verify bundled bytes, then validate both models through DomainForge.
/// Any drift or validation failure is `self_model_error`.
pub fn load_composed(models: &BundledModels) -> Result<ComposedModel, ForgeError> {
    verify_bundled(models)?;
    let system = load_one(&models.system)?;
    let seed = load_one(&models.seed)?;
    Ok(ComposedModel { system, seed })
}

// ── Release realization (deterministic) ──────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectionTarget {
    pub target: String,
    pub adapter_ref: String,
    pub limitations: Vec<String>,
    pub required_toolchain: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundledModelRef {
    pub uri: String,
    pub sha256: String,
}

/// Release realization (spec-adlc-thoth §7.2). Generated at release-build time;
/// installations verify, never regenerate, it. Deterministic: `generated_at`
/// derives from `SOURCE_DATE_EPOCH` (or a release constant), never wall-clock.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReleaseRealization {
    pub release_id: String,
    pub kernel_version: String,
    pub record_versions: BTreeMap<String, String>,
    pub crates: BTreeMap<String, String>,
    pub built_in_extensions: Vec<String>,
    pub projection_targets: Vec<ProjectionTarget>,
    pub bundled_templates: Vec<String>,
    pub bundled_environments: Vec<String>,
    pub bundled_models: Vec<BundledModelRef>,
    pub generated_at: String,
}

/// Deterministic `generated_at`: SOURCE_DATE_EPOCH if set, else the release
/// constant. Never reads wall-clock build time.
fn deterministic_generated_at() -> String {
    let epoch = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(RELEASE_EPOCH_SECONDS);
    DateTime::<Utc>::from_timestamp(epoch, 0)
        .map(|t| t.to_rfc3339())
        .unwrap_or_else(|| format!("unix:{epoch}"))
}

/// Build the release realization from release constants.
pub fn release_realization() -> ReleaseRealization {
    let mut crates = BTreeMap::new();
    for name in [
        "sea-forge-core",
        "sea-forge-cli",
        "sea-forge-authority",
        "sea-forge-planner",
        "sea-forge-sandbox",
        "sea-forge-runtime",
        "sea-forge-trace",
        "sea-forge-evidence",
        "sea-forge-settlement",
        "sea-forge-capability",
        "sea-forge-extension",
        "sea-forge-ledger",
        "sea-forge-domainforge",
        "sea-forge-spec-pipeline",
        "sea-forge-cell",
        "sea-forge-artifact-ip",
        "sea-forge-self-model",
        "sea-forge-server",
    ] {
        crates.insert(name.to_string(), "0.1.0".to_string());
    }
    let mut record_versions = BTreeMap::new();
    record_versions.insert("core".to_string(), KERNEL_VERSION.to_string());
    record_versions.insert("ledger".to_string(), "0.2".to_string());

    let models = bundled();
    ReleaseRealization {
        release_id: RELEASE_ID.into(),
        kernel_version: KERNEL_VERSION.into(),
        record_versions,
        crates,
        built_in_extensions: vec!["E11 Genesis Self-Model".into()],
        projection_targets: vec![
            ProjectionTarget {
                target: "kg".into(),
                adapter_ref: "sea-forge-domainforge".into(),
                limitations: vec!["knowledge-graph projection; rdf/turtle output".into()],
                required_toolchain: vec![],
            },
            ProjectionTarget {
                target: "calm".into(),
                adapter_ref: "sea-forge-domainforge".into(),
                limitations: vec![],
                required_toolchain: vec![],
            },
        ],
        bundled_templates: vec![],
        bundled_environments: vec!["demo_env@0.1.0".into()],
        bundled_models: vec![
            BundledModelRef {
                uri: models.system.uri.clone(),
                sha256: models.system.pinned_sha256.into(),
            },
            BundledModelRef {
                uri: models.seed.uri.clone(),
                sha256: models.seed.pinned_sha256.into(),
            },
        ],
        generated_at: deterministic_generated_at(),
    }
}

/// Canonical sha256 over a release realization (all fields; it is immutable).
pub fn realization_sha256(r: &ReleaseRealization) -> Result<String, ForgeError> {
    canonical_sha256(r)
}

// ── Cell realization ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeResult {
    Available,
    Unavailable,
    VersionMismatch,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolchainProbe {
    pub tool: String,
    pub required_by: String,
    pub probe_command_ref: String,
    pub result: ProbeResult,
    pub evidence_ref: Option<String>,
}

/// Installation/cell realization (spec-adlc-thoth §7.2): registry state,
/// environment contracts, toolchain probes, sandbox classes, degraded components.
/// Rebuildable from the extension registry plus evidenced probe runs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CellRealization {
    pub cell_id: String,
    pub active_extensions: Vec<ExtensionState>,
    pub environments_present: Vec<String>,
    pub toolchain_probes: Vec<ToolchainProbe>,
    pub sandbox_classes_available: Vec<String>,
    pub degraded_components: Vec<String>,
    pub probed_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionState {
    pub descriptor_ref: String,
    pub status: String,
}

/// Canonical sha256 over a cell realization.
pub fn cell_realization_sha256(c: &CellRealization) -> Result<String, ForgeError> {
    canonical_sha256(c)
}

// ── Self-model snapshot ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotFreshness {
    Current,
    Stale,
}

/// Immutable self-model snapshot (spec-adlc-thoth §7.1).
/// Created on init, upgrade, extension install/adopt/disable, and explicit rebuild.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfModelSnapshot {
    pub snapshot_id: String,
    pub release_id: String,
    pub created_at: String,
    pub freshness: SnapshotFreshness,
    pub system_model_ref: DomainModelRef,
    pub seed_model_refs: Vec<DomainModelRef>,
    pub release_realization_sha256: String,
    pub cell_realization_sha256: String,
    pub capability_projection_sha256: String,
    pub snapshot_hash: String,
}

/// Canonical snapshot hash over the documented tuple (§7.1), excluding the
/// self-hash and `created_at`/`snapshot_id` timestamp fields.
pub fn snapshot_hash(
    system_model_ref: &DomainModelRef,
    seed_model_refs: &[DomainModelRef],
    release_realization_sha256: &str,
    cell_realization_sha256: &str,
    capability_projection_sha256: &str,
    release_id: &str,
) -> Result<String, ForgeError> {
    let tuple = serde_json::json!({
        "system_model_sha256": system_model_ref.semantic_model_sha256,
        "seed_model_sha256": seed_model_refs
            .iter()
            .map(|r| r.semantic_model_sha256.clone())
            .collect::<Vec<_>>(),
        "release_realization_sha256": release_realization_sha256,
        "cell_realization_sha256": cell_realization_sha256,
        "capability_projection_sha256": capability_projection_sha256,
        "release_id": release_id,
    });
    let bytes = canonical_json(&tuple)?;
    Ok(format!("sha256:{:x}", Sha256::digest(&bytes)))
}

/// Assemble a snapshot from its verified sources. Computes `snapshot_hash` and
/// mints the `smsnap_` id. `capability_projection_sha256` is the canonical hash
/// of the demonstrated-self join at snapshot time (empty-string placeholder when
/// no demonstrated capability view exists yet).
pub fn build_snapshot(
    composed: &ComposedModel,
    realization: &ReleaseRealization,
    cell: &CellRealization,
    capability_projection_sha256: &str,
    created_at: &str,
) -> Result<SelfModelSnapshot, ForgeError> {
    let release_rh = realization_sha256(realization)?;
    let cell_rh = cell_realization_sha256(cell)?;
    let sh = snapshot_hash(
        composed.system_model_ref(),
        &[composed.seed_model_ref().clone()],
        &release_rh,
        &cell_rh,
        capability_projection_sha256,
        &realization.release_id,
    )?;
    Ok(SelfModelSnapshot {
        snapshot_id: ids::snapshot_id()?,
        release_id: realization.release_id.clone(),
        created_at: created_at.to_string(),
        freshness: SnapshotFreshness::Current,
        system_model_ref: composed.system_model_ref().clone(),
        seed_model_refs: vec![composed.seed_model_ref().clone()],
        release_realization_sha256: release_rh,
        cell_realization_sha256: cell_rh,
        capability_projection_sha256: capability_projection_sha256.to_string(),
        snapshot_hash: sh,
    })
}

/// Recompute a snapshot's `snapshot_hash` from its stored fields and confirm it
/// matches the recorded value. Drift ⇒ `self_model_error`.
pub fn verify_snapshot(snapshot: &SelfModelSnapshot) -> Result<(), ForgeError> {
    let expected = snapshot_hash(
        &snapshot.system_model_ref,
        &snapshot.seed_model_refs,
        &snapshot.release_realization_sha256,
        &snapshot.cell_realization_sha256,
        &snapshot.capability_projection_sha256,
        &snapshot.release_id,
    )?;
    if expected != snapshot.snapshot_hash {
        return Err(ForgeError::SelfModel(format!(
            "self_model_error: snapshot hash mismatch for {}: declared={} computed={expected}",
            snapshot.snapshot_id, snapshot.snapshot_hash
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_models_verify_and_load() {
        let models = bundled();
        verify_bundled(&models).expect("bundled hashes must match pinned constants");
        let composed = load_composed(&models).expect("both models must validate");
        assert!(composed.concept_exists("SEA Forge"));
        assert!(composed.concept_exists("Desired Outcome Criterion"));
        assert!(composed.concepts().len() > 100);
    }

    #[test]
    fn tampered_bytes_fail_before_validation() {
        let mut tampered = bundled();
        // Inject a one-byte drift in the in-memory copy via a leaked static slice.
        let mut bad: Vec<u8> = tampered.system.bytes.to_vec();
        let last = bad.len() - 1;
        bad[last] = if bad[last] == b'a' { b'b' } else { b'a' };
        let leaked: &'static [u8] = Box::leak(bad.into_boxed_slice());
        tampered.system.bytes = leaked;
        let err = verify_bundled(&tampered).unwrap_err();
        assert_eq!(err.class(), "self_model_error");
    }

    #[test]
    fn release_realization_is_deterministic() {
        let r1 = release_realization();
        let r2 = release_realization();
        assert_eq!(
            realization_sha256(&r1).unwrap(),
            realization_sha256(&r2).unwrap()
        );
        assert_eq!(r1.generated_at, r2.generated_at);
    }

    #[test]
    fn snapshot_round_trips_and_verifies() {
        let composed = load_composed(&bundled()).unwrap();
        let realization = release_realization();
        let cell = CellRealization {
            cell_id: "cell_00000000".into(),
            active_extensions: vec![],
            environments_present: vec!["demo_env@0.1.0".into()],
            toolchain_probes: vec![],
            sandbox_classes_available: vec!["local".into()],
            degraded_components: vec![],
            probed_at: deterministic_generated_at(),
        };
        let snap = build_snapshot(
            &composed,
            &realization,
            &cell,
            "sha256:capability",
            "2026-07-16T00:00:00Z",
        )
        .unwrap();
        assert!(snap.snapshot_id.starts_with("smsnap_"));
        verify_snapshot(&snap).unwrap();
        // All five digest fields populated.
        assert!(!snap.release_realization_sha256.is_empty());
        assert!(!snap.cell_realization_sha256.is_empty());
        assert!(!snap.capability_projection_sha256.is_empty());
        assert!(!snap.snapshot_hash.is_empty());
        assert!(!snap.system_model_ref.semantic_model_sha256.is_empty());
    }
}
