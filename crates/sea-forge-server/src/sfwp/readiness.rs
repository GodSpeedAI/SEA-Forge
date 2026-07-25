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
//!   `category`, `status`, `reason`, `source_ref`.
//! - `recent_invalidations` is shipped as an **empty array** in this slice: no
//!   honest source for invalidation records exists yet (no cache/invalidation
//!   layer). This is intentional partial scope, recorded as `PARTIAL` in the
//!   task report — never fabricated.
//! - The `stale` value of `overall` is deliberately **not derived**: this slice
//!   has no cache layer to make that determination honestly. Recorded as a gap.

use schemars::JsonSchema;
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

/// A single readiness condition, projected from kernel truth. Shapes the
/// wireframe's `ReadinessConditionTable` row: an identity, a human label, a
/// grouping category, a status, a plain-language reason, and a short citation
/// (`source_ref`) — a spec section or file, matching the kit's "Atomic §4.4" /
/// "DESIGN.md" source column. `source_ref` is a citation string, **not** a
/// content-addressed `RecordRef` digest (which this slice cannot honestly back).
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
    /// Short citation string (spec section or file), not a record digest.
    pub source_ref: String,
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

/// Overall readiness verdict, derived deterministically from the worst status
/// among conditions. `Stale` is a spec value this slice never derives (no cache
/// layer to determine it honestly).
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
    /// Reserved spec value; never derived in this slice (no cache layer).
    Stale,
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
    /// Short citation string for the invalidating change.
    pub source_ref: String,
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
/// `config.root` is the server's `.sea-forge` directory; `store::validate`
/// expects a *project root* and appends `.sea-forge` itself, so we hand it the
/// parent (falling back to `config.root` when there is no parent).
pub fn get(config: &ServerConfig, params: ReadinessGetParams) -> ReadinessView {
    let intended_operation = params.intended_operation;

    // --- Foundation: self-model / integrity substrate. ---------------------
    let project_root = config
        .root
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| config.root.clone());

    let (self_model_item, foundation_failure) =
        match sea_forge_self_model::store::validate(&project_root) {
            Ok(()) => (
                ReadinessItem {
                    id: "self_model_integrity".into(),
                    name: "Self-model integrity".into(),
                    category: "foundation".into(),
                    status: ReadinessStatus::Ready,
                    reason: String::new(),
                    source_ref: "sea-forge-self-model/src/store.rs::validate".into(),
                },
                None,
            ),
            Err(error) => {
                let message = error.to_string();
                let integrity = is_integrity_failure(&message);
                // A blocked foundation renders `blocked` at the item level
                // regardless; the integrity distinction only sharpens `overall`
                // into `integrity_halted`.
                (
                    ReadinessItem {
                        id: "self_model_integrity".into(),
                        name: "Self-model integrity".into(),
                        category: "foundation".into(),
                        status: ReadinessStatus::Blocked,
                        reason: message,
                        source_ref: "sea-forge-self-model/src/store.rs::validate".into(),
                    },
                    Some(if integrity {
                        OverallReadiness::IntegrityHalted
                    } else {
                        OverallReadiness::Blocked
                    }),
                )
            }
        };

    let foundations = vec![self_model_item];

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
            "Blocked: self-model foundation is not trusted".into()
        } else {
            String::new()
        },
        source_ref: "sea-forge-server::case_dispatch".into(),
    };

    // External delegation readiness: derived from whether any agent endpoint is
    // configured. No endpoints ⇒ degraded (external delegation cannot proceed)
    // with a named reason sourced from the endpoint-verification requirement.
    let external_configured = !config.agent.endpoints.is_empty();
    let external_capability = ReadinessItem {
        id: "external_delegation".into(),
        name: "External delegation".into(),
        category: "operational_capability".into(),
        status: if external_configured {
            ReadinessStatus::Ready
        } else {
            ReadinessStatus::Degraded
        },
        reason: if external_configured {
            String::new()
        } else {
            "Endpoint verification has not been recorded".into()
        },
        source_ref: "sea-forge-agent::AgentConfig::endpoints".into(),
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

/// Whether an intended-operation method depends on external agent delegation.
/// Kept intentionally small for v1; extended as the method catalog grows.
fn operation_requires_delegation(method: &str) -> bool {
    let lowered = method.to_ascii_lowercase();
    lowered.contains("delegate")
        || lowered.contains("agent_run")
        || lowered.contains("agent_task")
        || lowered.contains("agent.")
}
