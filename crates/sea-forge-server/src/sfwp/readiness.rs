//! `readiness.get` — an SFWP **inspect** method (ADR-003 additive verb).
//!
//! # Contract
//!
//! This is a *read-only projection* over already-proven kernel truth. It
//! introduces **no new truth** and performs **no mutation**: it calls
//! [`sea_forge_self_model::store::validate`] and the agent-endpoint config
//! accessor, then *shapes* their results into a [`ReadinessView`]. It never
//! forks governance/validation logic into the server handler.
//!
//! Per the workbench invariant `unknown ≠ unavailable`, [`get`] is **infallible**:
//! a self-model validation failure is a *reported* `blocked`/`integrity_halted`
//! status inside the returned view, never a propagated `Err`. An inspect method
//! must always answer with a shaped view.
//!
//! # Grounding / honesty notes
//!
//! - `ReadinessItem` and `Invalidation` are referenced by the API spec
//!   (`sea-forge-workbench-api-spec-v0.1.md:1177-1197`) but never defined there.
//!   The shapes below are the minimal honest fields backed by the Readiness UI
//!   kit's `ReadinessConditionTable` / capability-row markup
//!   (`.agents/specs/frontend/ui_kits/app/index.html`): `id`, `name`,
//!   `category`, `status`, `reason`, and a typed committed source reference.
//! - `recent_invalidations` is shipped as an **empty array** in this slice: no
//!   honest source for invalidation records exists yet (no cache/invalidation
//!   layer). This is intentional partial scope, recorded as `PARTIAL` in the
//!   task report — never fabricated.
//! - `overall: stale` is derived only from the self-model manifest's explicit
//!   stale standing; renderer cache freshness is a separate concern.

use schemars::JsonSchema;
use sea_forge_ledger::LedgerStream;
use serde::{Deserialize, Serialize};

use crate::config::ServerConfig;

/// Coarse status of a single readiness condition. Mirrors the four states the
/// Readiness kit's capability rows can render.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessStatus {
    /// The condition is fully satisfied.
    Ready,
    /// Usable but with a named limitation.
    Degraded,
    /// The condition prevents the associated capability.
    Blocked,
    /// Genuinely undetermined — distinct from `Blocked` (`unknown ≠ unavailable`).
    Unknown,
}

/// Freshness of the canonical record behind a projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceFreshness {
    Current,
    Stale,
}

/// Whether the committed source can presently be rebuilt and verified.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RebuildStanding {
    Verified,
    RebuildRequired,
}

/// A content-addressed, ledger-committed source record behind a view.
///
/// This deliberately cannot hold a source file path or code citation.  A
/// renderer may show implementation location as explanatory material, but a
/// readiness standing must resolve to a committed record and digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceRecordRef {
    pub ledger_id: String,
    pub entry_id: String,
    pub record_kind: String,
    pub record_id: String,
    pub digest: String,
    pub freshness: SourceFreshness,
    pub rebuild_standing: RebuildStanding,
}

/// A single readiness condition, projected from kernel truth. Shapes the
/// wireframe's `ReadinessConditionTable` row: an identity, a human label, a
/// grouping category, a status, a plain-language reason, and the committed
/// source record that proves it. A missing source is explicit and means the
/// standing is not proven; it never falls back to a code citation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReadinessItem {
    /// Stable machine identity for this condition (e.g. `"self_model_integrity"`).
    pub id: String,
    /// Human-facing label (e.g. `"Self-model integrity"`).
    pub name: String,
    /// Grouping bucket — `"foundation"` or `"operational_capability"`.
    pub category: String,
    /// Coarse condition status.
    pub status: ReadinessStatus,
    /// Plain-language reason for the status. Empty string when self-evidently ready.
    pub reason: String,
    /// The committed source record that proves this standing, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRecordRef>,
    /// The next lawful action when this condition is not ready.
    pub next_lawful_action: String,
}

/// The operation the operator intends to perform next, if any. Lets the view
/// foreground the capabilities that operation actually depends on
/// (operation-sensitivity). Mirrors the request example at
/// `sea-forge-workbench-api-spec-v0.1.md:2037-2041`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IntendedOperation {
    /// The SFWP method (or method family) the operator intends to invoke next.
    pub method: String,
    /// Optional resource the operation targets. A citation string in this
    /// slice; not resolved to a canonical record digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_ref: Option<String>,
}

/// Request params for `readiness.get`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReadinessGetParams {
    /// Optional intended operation to foreground relevant capabilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intended_operation: Option<IntendedOperation>,
}

/// Overall readiness verdict, derived deterministically from the foundational
/// source standing and the worst operational condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OverallReadiness {
    /// All foundations pass and no operational capability is degraded/blocked.
    Ready,
    /// Foundations pass but at least one operational capability is degraded/blocked.
    ReadyWithLimitations,
    /// A foundation is blocked for a non-integrity reason.
    Blocked,
    /// A foundation failed specifically on ledger/snapshot integrity verification.
    IntegrityHalted,
    /// A committed source remains readable but requires a rebuild before use.
    Stale,
    /// No committed source record can prove the current standing.
    Unknown,
}

/// The `readiness.get` view body — a read-only projection, not source truth.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReadinessView {
    /// Echo of the intended operation this view was shaped for, if provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intended_operation: Option<IntendedOperation>,
    /// Deterministically-derived overall verdict.
    pub overall: OverallReadiness,
    /// Foundational conditions (self-model / integrity substrate).
    pub foundations: Vec<ReadinessItem>,
    /// Operational capabilities (what the operator can actually do now).
    pub operational_capabilities: Vec<ReadinessItem>,
    /// Recent invalidations. Empty in this slice (no honest source yet).
    pub recent_invalidations: Vec<Invalidation>,
}

/// A recorded invalidation of a prior readiness determination. Defined minimally
/// because the spec references but never defines it. Not populated in this slice
/// (no cache/invalidation layer exists to source real records).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Invalidation {
    /// Which condition was invalidated.
    pub item_id: String,
    /// Plain-language reason the prior determination no longer holds.
    pub reason: String,
    /// The committed record that invalidated the prior standing, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRecordRef>,
}

/// Distinguish an integrity-verification failure (ledger hash-chain / snapshot
/// hash / projection rebuild) from other self-model validation failures. The
/// error class is coarse (`self_model_error` covers both), so we inspect the
/// rendered message for the verification markers those code paths emit.
fn is_integrity_failure(message: &str) -> bool {
    let lowered = message.to_ascii_lowercase();
    [
        "hash chain",
        "hash-chain",
        "rebuild_hash",
        "snapshot",
        "verify",
        "chain",
    ]
    .iter()
    .any(|marker| lowered.contains(marker))
}

/// Build the `readiness.get` view. Infallible: validation failure is reported
/// as a status inside the view, never propagated.
///
/// `config.root` is the state root (F-12): self-model ledgers, snapshots, and
/// projections all join directly beneath it. Treating any parent or nested
/// `.sea-forge` as the root would validate a different cell and make a fresh
/// server look ready without a local committed snapshot.
pub fn get(config: &ServerConfig, params: ReadinessGetParams) -> ReadinessView {
    let intended_operation = params.intended_operation;

    // --- Foundation: self-model / integrity substrate. ---------------------
    let (self_model_item, foundation_failure) = match committed_snapshot_source(&config.root) {
        Ok(Some(source)) if source.freshness == SourceFreshness::Current => (
            ReadinessItem {
                id: "self_model_integrity".into(),
                name: "Self-model integrity".into(),
                category: "foundation".into(),
                status: ReadinessStatus::Ready,
                reason: String::new(),
                source: Some(source),
                next_lawful_action: "Continue with a governed operation".into(),
            },
            None,
        ),
        Ok(Some(source)) => (
            ReadinessItem {
                id: "self_model_integrity".into(),
                name: "Self-model integrity".into(),
                category: "foundation".into(),
                status: ReadinessStatus::Degraded,
                reason: "The committed self-model snapshot is stale".into(),
                source: Some(source),
                next_lawful_action: "Rebuild and verify the self-model snapshot".into(),
            },
            Some(OverallReadiness::Stale),
        ),
        Ok(None) => (
            ReadinessItem {
                id: "self_model_integrity".into(),
                name: "Self-model integrity".into(),
                category: "foundation".into(),
                status: ReadinessStatus::Unknown,
                reason: "No committed self-model snapshot is available to prove this standing"
                    .into(),
                source: None,
                next_lawful_action: "Initialize or rebuild the self-model, then re-check readiness"
                    .into(),
            },
            Some(OverallReadiness::Unknown),
        ),
        Err(error) => {
            let message = error.to_string();
            let integrity = is_integrity_failure(&message);
            (
                ReadinessItem {
                    id: "self_model_integrity".into(),
                    name: "Self-model integrity".into(),
                    category: "foundation".into(),
                    status: ReadinessStatus::Blocked,
                    reason: message,
                    source: None,
                    next_lawful_action:
                        "Repair the reported integrity failure, then re-check readiness".into(),
                },
                Some(if integrity {
                    OverallReadiness::IntegrityHalted
                } else {
                    OverallReadiness::Blocked
                }),
            )
        }
    };

    // --- Operational capabilities. -----------------------------------------
    // Local governed execution is ready whenever the self-model substrate is
    // trusted — that is the "core sync kernel" capability every other verb
    // already proves by being able to run.
    let local_status = if foundation_failure.is_some() {
        ReadinessStatus::Blocked
    } else {
        ReadinessStatus::Ready
    };
    let local_capability = ReadinessItem {
        id: "local_governed_execution".into(),
        name: "Local governed execution".into(),
        category: "operational_capability".into(),
        status: local_status,
        reason: if foundation_failure.is_some() {
            "Blocked: self-model foundation is not currently proven".into()
        } else {
            String::new()
        },
        source: self_model_item.source.clone(),
        next_lawful_action: if foundation_failure.is_some() {
            self_model_item.next_lawful_action.clone()
        } else {
            "Create or inspect a governed case".into()
        },
    };

    let foundations = vec![self_model_item];

    // External delegation requires a committed probe/settlement record, not
    // merely a configured endpoint.
    let external_capability = ReadinessItem {
        id: "external_delegation".into(),
        name: "External delegation".into(),
        category: "operational_capability".into(),
        // Endpoint configuration is intent, not evidence that an endpoint is
        // reachable or approved. There is no committed endpoint-probe record
        // behind this projection yet, so both configured and unconfigured
        // cases remain explicitly unknown.
        status: ReadinessStatus::Unknown,
        reason: "No committed endpoint verification record is available".into(),
        source: None,
        next_lawful_action: "Run a governed endpoint probe and inspect its committed settlement"
            .into(),
    };

    // Operation-sensitivity: when the intended operation needs external
    // delegation, foreground that capability by ordering it first.
    let operation_needs_delegation = intended_operation
        .as_ref()
        .map(|op| operation_requires_delegation(&op.method))
        .unwrap_or(false);
    let operational_capabilities = if operation_needs_delegation {
        vec![external_capability, local_capability]
    } else {
        vec![local_capability, external_capability]
    };

    // --- Overall verdict (worst-status derivation). ------------------------
    let overall = if let Some(foundation) = foundation_failure {
        foundation
    } else if operational_capabilities.iter().any(|item| {
        matches!(
            item.status,
            ReadinessStatus::Degraded | ReadinessStatus::Blocked
        )
    }) {
        OverallReadiness::ReadyWithLimitations
    } else {
        OverallReadiness::Ready
    };

    ReadinessView {
        intended_operation,
        overall,
        foundations,
        operational_capabilities,
        // PARTIAL: no invalidation source in this slice — honest empty, never fabricated.
        recent_invalidations: Vec::new(),
    }
}

/// Resolve the currently selected self-model snapshot to its ledger entry.
///
/// A materialized snapshot file alone is not authoritative: it must also be
/// present in the verified self-model ledger. `None` is therefore an honest
/// unknown, not a reason to substitute an implementation citation.
fn committed_snapshot_source(
    project_root: &std::path::Path,
) -> Result<Option<SourceRecordRef>, sea_forge_core::errors::ForgeError> {
    sea_forge_self_model::store::validate(project_root)?;
    let Some(snapshot) = sea_forge_self_model::store::current_snapshot(project_root)? else {
        return Ok(None);
    };
    let stale = sea_forge_self_model::store::is_stale(project_root)?;
    let ledger_dir = project_root.join("ledgers").join("self-model");
    if !ledger_dir.exists() {
        return Ok(None);
    }
    let stream = LedgerStream::open(project_root, "self-model", "readiness-inspect")?;
    let Some(entry) = stream.read_entries()?.into_iter().find(|entry| {
        entry.record_kind == "self_model_snapshot"
            && entry
                .payload
                .get("snapshot_id")
                .and_then(|value| value.as_str())
                == Some(snapshot.snapshot_id.as_str())
    }) else {
        return Ok(None);
    };
    Ok(Some(SourceRecordRef {
        ledger_id: entry.ledger_id,
        entry_id: entry.entry_ulid,
        record_kind: entry.record_kind,
        record_id: snapshot.snapshot_id,
        digest: entry.payload_hash,
        freshness: if stale {
            SourceFreshness::Stale
        } else {
            SourceFreshness::Current
        },
        rebuild_standing: if stale {
            RebuildStanding::RebuildRequired
        } else {
            RebuildStanding::Verified
        },
    }))
}

/// Whether an intended-operation method depends on external agent delegation.
/// Kept intentionally small for v1; extended as the method catalog grows.
fn operation_requires_delegation(method: &str) -> bool {
    let lowered = method.to_ascii_lowercase();
    lowered.contains("delegate")
        || lowered.contains("agent_run")
        || lowered.contains("agent_task")
        || lowered.contains("agent.")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(root: &std::path::Path) -> ServerConfig {
        ServerConfig {
            root: root.to_path_buf(),
            ..ServerConfig::default()
        }
    }

    #[test]
    fn an_uninitialized_cell_is_unknown_without_a_synthetic_source_citation() {
        let root = tempfile::tempdir().unwrap();
        let view = get(&config(root.path()), ReadinessGetParams::default());
        let foundation = &view.foundations[0];

        assert_eq!(view.overall, OverallReadiness::Unknown);
        assert_eq!(foundation.status, ReadinessStatus::Unknown);
        assert!(foundation.source.is_none());
        assert!(foundation
            .reason
            .contains("No committed self-model snapshot"));
        assert!(foundation
            .next_lawful_action
            .contains("Initialize or rebuild"));
    }

    #[test]
    fn a_verified_snapshot_is_projected_as_a_committed_digest_not_a_code_location() {
        let root = tempfile::tempdir().unwrap();
        let snapshot = sea_forge_self_model::store::rebuild(
            root.path(),
            &sea_forge_self_model::store::RebuildInputs {
                cell_id: "cell_readiness_test",
                created_at: "2026-08-02T00:00:00Z",
                capability_projection_sha256: "sha256:readiness-capability-projection",
                actor_id: "operator_test",
                ..Default::default()
            },
        )
        .unwrap();

        let view = get(&config(root.path()), ReadinessGetParams::default());
        let source = view.foundations[0]
            .source
            .as_ref()
            .expect("a rebuilt snapshot must resolve to its committed ledger entry");

        assert_eq!(view.foundations[0].status, ReadinessStatus::Ready);
        assert_eq!(source.ledger_id, "self-model");
        assert_eq!(source.record_kind, "self_model_snapshot");
        assert_eq!(source.record_id, snapshot.snapshot_id);
        assert_eq!(source.entry_id.len(), 26, "{source:?}");
        assert!(
            source
                .entry_id
                .chars()
                .all(|character| character.is_ascii_alphanumeric()),
            "{source:?}"
        );
        assert!(source.digest.starts_with("sha256:"), "{source:?}");
        assert_eq!(source.freshness, SourceFreshness::Current);
        assert_eq!(source.rebuild_standing, RebuildStanding::Verified);
    }

    #[test]
    fn a_stale_snapshot_never_reports_readiness_as_success() {
        let root = tempfile::tempdir().unwrap();
        sea_forge_self_model::store::rebuild(
            root.path(),
            &sea_forge_self_model::store::RebuildInputs {
                cell_id: "cell_readiness_test",
                created_at: "2026-08-02T00:00:00Z",
                capability_projection_sha256: "sha256:readiness-capability-projection",
                actor_id: "operator_test",
                ..Default::default()
            },
        )
        .unwrap();
        sea_forge_self_model::store::mark_current_stale(root.path(), "test_drift").unwrap();

        let view = get(&config(root.path()), ReadinessGetParams::default());
        assert_eq!(view.overall, OverallReadiness::Stale);
        assert_eq!(view.foundations[0].status, ReadinessStatus::Degraded);
        assert_eq!(
            view.foundations[0]
                .source
                .as_ref()
                .unwrap()
                .rebuild_standing,
            RebuildStanding::RebuildRequired
        );
    }
}
