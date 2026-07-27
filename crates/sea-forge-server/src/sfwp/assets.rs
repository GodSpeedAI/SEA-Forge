//! `asset.list` — the SFWP **inspect** projection over the operating assets
//! this cell actually holds (Task 9, ADR-003 additive; epic journey 4).
//!
//! # What an "asset" is here
//!
//! Three families the kernel already records, and nothing else:
//!
//! | family | source of truth |
//! |---|---|
//! | plan templates | `<root>/templates/*.yaml` (the same files `case.entry_options` lists) |
//! | agent endpoints | `server.yaml`'s `agent.endpoints` + the evidence that has accrued against each |
//! | extensions | `<root>/extensions/registry.json` (`sea_forge_extension::ExtensionRegistry`) |
//!
//! Nothing is invented. A cell with no templates, no endpoints, and no
//! extensions gets an empty list — an honest answer, never a fabricated
//! built-in catalog (`unknown ≠ unavailable`).
//!
//! # Why `standing` is a kind-native string, not one shared ladder
//!
//! The obvious simplification is a single availability enum every asset is
//! squeezed into. It is wrong. The kernel already owns three *different*
//! standing vocabularies with three different meanings, and each is evidence-
//! backed on its own terms:
//!
//! - endpoints: [`sea_forge_agent::EndpointStatus`] — `declared`, `probed`,
//!   `demonstrated`. `AgentEndpointConfig::validate` explicitly *refuses* a
//!   status asserted in configuration ("status is evidence-derived"), so this
//!   ladder may only ever be climbed by reading committed records.
//! - extensions: [`sea_forge_extension::ExtensionStatus`] — `active`,
//!   `disabled`, `quarantined`, `superseded`.
//! - templates: a template is materialized on disk or it is not.
//!
//! Collapsing them would make a `quarantined` extension and an unprobed
//! endpoint render as the same "not available" — erasing the distinction
//! between *refused* and *not yet proven*, which is the whole point of epic
//! 4.8. So the row carries its own kind's word verbatim and the UI maps each
//! word separately.
//!
//! # Why availability and blocking are separate fields
//!
//! `standing` answers "what has been proven about this asset?"; `blocking_reason`
//! answers "will this cell offer it for work right now?". They are not the same
//! question: a `declared` endpoint is unproven but perfectly legitimate to probe,
//! while an endpoint whose *last probe was rejected* is equally `probed` and must
//! not be selected for delegation. One field cannot say both without one of the
//! two meanings being lost.

use std::collections::BTreeMap;
use std::path::Path;

use schemars::JsonSchema;
use sea_forge_agent::AgentConfig;
use sea_forge_core::types::{CasePlan, Operation, SettlementEvent, SettlementStatus};
use sea_forge_extension::{ExtensionRegistry, ExtensionStatus, TrustLevel};
use serde::{Deserialize, Serialize};

use crate::sfwp::run_views::{read_json, run_dirs};

/// Which family an asset belongs to. Closed on purpose: a fourth family means a
/// fourth real source of truth in the kernel, not a free-text label.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    PlanTemplate,
    AgentEndpoint,
    Extension,
}

/// One asset, projected. Every field is read from a committed record or from
/// configuration; none is inferred from another.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AssetRow {
    /// Stable, kind-prefixed address (`template:repair@1.0.0`,
    /// `agent_endpoint:local`, `extension:acme.linter@2.1.0`).
    pub asset_id: String,
    pub kind: AssetKind,
    pub name: String,
    /// `None` where the kind has no version concept (agent endpoints are
    /// identified by their descriptor digest, not by a version).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// This asset kind's own standing word, verbatim. See the module docs for
    /// why this is deliberately not one cross-kind ladder.
    pub standing: String,
    /// The digest that identifies exactly these bytes/this configuration.
    /// `None` when the source could not be read well enough to compute one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_digest: Option<String>,
    /// The committed records that put `standing` where it is. Empty at the
    /// floor of a ladder, where nothing has been proven yet — an empty list is
    /// the honest answer, not a missing one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    /// Why this cell will not offer the asset for work (epic 4.8). `None` means
    /// nothing blocks it — which is *not* a claim that it has been proven.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_reason: Option<String>,
}

/// `asset.list` result body.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct AssetListResult {
    pub assets: Vec<AssetRow>,
    /// Asset *sources* that exist but could not be read — an unparseable
    /// template, an unreadable registry. Reported rather than silently dropped:
    /// `agent_probe::list` and `case.entry_options` both filter these away, so a
    /// broken file reads as "no such asset" there. Here it reads as an
    /// integrity signal, which is what it is.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unreadable: Vec<String>,
}

// --- standing vocabularies, one per kind (never mixed) ---

/// Templates: materialized on disk is the only fact a template has.
const TEMPLATE_MATERIALIZED: &str = "materialized";

/// Endpoints, mirroring `sea_forge_agent::EndpointStatus` exactly.
const ENDPOINT_DECLARED: &str = "declared";
const ENDPOINT_PROBED: &str = "probed";
const ENDPOINT_DEMONSTRATED: &str = "demonstrated";

/// Build the `asset.list` view. Infallible in the sense every inspect method
/// here is: an unreadable source becomes a row in `unreadable`, never a
/// propagated transport error.
pub fn list(root: &Path, agent: &AgentConfig) -> AssetListResult {
    let mut result = AssetListResult::default();
    collect_templates(root, &mut result);
    collect_endpoints(root, agent, &mut result);
    collect_extensions(root, &mut result);
    result.assets.sort_by(|a, b| a.asset_id.cmp(&b.asset_id));
    result.unreadable.sort();
    result
}

fn collect_templates(root: &Path, out: &mut AssetListResult) {
    let dir = root.join("templates");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        // No templates directory is not an error: this cell has materialized
        // no templates. Distinct from a directory that exists and cannot be
        // listed, which `read_dir` would also surface — but that case is rare
        // enough that conflating it costs an operator nothing they can act on.
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("<unnamed>")
            .to_string();
        let digest = std::fs::read(&path).ok().map(|bytes| {
            use sha2::{Digest, Sha256};
            format!("sha256:{:x}", Sha256::digest(&bytes))
        });
        match sea_forge_planner::templates::load(&path) {
            Ok(template) => {
                let asset_ref = format!("{}@{}", template.name, template.version);
                out.assets.push(AssetRow {
                    asset_id: format!("template:{asset_ref}"),
                    kind: AssetKind::PlanTemplate,
                    name: template.name,
                    version: Some(template.version),
                    standing: TEMPLATE_MATERIALIZED.into(),
                    identity_digest: digest,
                    evidence_refs: Vec::new(),
                    blocking_reason: None,
                });
            }
            Err(error) => {
                // A template file that will not load still exists, and an
                // operator wondering why it is not offered deserves the reason
                // rather than its disappearance.
                out.unreadable.push(format!("template:{stem}"));
                out.assets.push(AssetRow {
                    asset_id: format!("template:{stem}"),
                    kind: AssetKind::PlanTemplate,
                    name: stem,
                    version: None,
                    standing: TEMPLATE_MATERIALIZED.into(),
                    identity_digest: digest,
                    evidence_refs: Vec::new(),
                    blocking_reason: Some(format!("template file could not be loaded: {error}")),
                });
            }
        }
    }
}

/// The most recent probe settlement per endpoint, from the run records the
/// probe path writes (`runs/<run>/plan.json` carries `Operation::AgentProbe`,
/// `runs/<run>/settlement.json` carries the verdict).
struct ProbeEvidence {
    run_id: String,
    settled_at: String,
    status: SettlementStatus,
    basis: Vec<String>,
}

fn latest_probe_per_endpoint(root: &Path) -> BTreeMap<String, ProbeEvidence> {
    let mut latest: BTreeMap<String, ProbeEvidence> = BTreeMap::new();
    for (run_id, dir) in run_dirs(root) {
        let Some(plan) = read_json::<CasePlan>(&dir.join("plan.json")) else {
            continue;
        };
        let endpoints: Vec<String> = plan
            .items
            .iter()
            .flat_map(|item| &item.operations)
            .filter_map(|op| match op {
                Operation::AgentProbe { endpoint_ref, .. } => Some(endpoint_ref.clone()),
                _ => None,
            })
            .collect();
        if endpoints.is_empty() {
            continue;
        }
        let Some(settlement) = read_json::<SettlementEvent>(&dir.join("settlement.json")) else {
            continue;
        };
        for endpoint_ref in endpoints {
            // `settled_at` is the kernel's own RFC3339 UTC stamp, so a string
            // compare orders it; `run_id` breaks ties deterministically rather
            // than letting directory iteration order decide which probe counts.
            let candidate_key = (settlement.settled_at.as_str(), run_id.as_str());
            let replace = latest.get(&endpoint_ref).is_none_or(|held| {
                (held.settled_at.as_str(), held.run_id.as_str()) < candidate_key
            });
            if replace {
                latest.insert(
                    endpoint_ref,
                    ProbeEvidence {
                        run_id: run_id.clone(),
                        settled_at: settlement.settled_at.clone(),
                        status: settlement.status.clone(),
                        basis: settlement.basis.clone(),
                    },
                );
            }
        }
    }
    latest
}

fn collect_endpoints(root: &Path, agent: &AgentConfig, out: &mut AssetListResult) {
    if agent.endpoints.is_empty() {
        return;
    }
    let probes = latest_probe_per_endpoint(root);
    // An endpoint registered as an immutable runtime adapter was probed under
    // an allowed authority decision — the registration happens only after the
    // grant (see `crate::agent_probe::register_endpoint`). It is therefore
    // evidence of a probe even when the run records have been pruned.
    let registered = ExtensionRegistry::load(root)
        .map(|registry| {
            registry
                .extensions
                .into_iter()
                .filter_map(|entry| {
                    entry
                        .extension_id
                        .strip_prefix("agent_endpoint_")
                        .map(|id| (id.to_string(), entry.authority_ref))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    for endpoint in &agent.endpoints {
        let snapshot = endpoint.snapshot();
        let identity_digest = snapshot
            .as_ref()
            .ok()
            .map(|s| s.descriptor_config_sha256.clone());
        let mut evidence_refs = Vec::new();
        let mut standing = ENDPOINT_DECLARED;
        // Configuration that will not snapshot is the one blocking condition
        // that outranks probe evidence: nothing can lawfully use the endpoint
        // until it is fixed, whatever a past probe demonstrated.
        let mut blocking_reason = snapshot
            .as_ref()
            .err()
            .map(|message| format!("endpoint configuration is invalid: {message}"));

        if let Some(authority_ref) = registered.get(&endpoint.id) {
            standing = ENDPOINT_PROBED;
            if let Some(reference) = authority_ref {
                evidence_refs.push(reference.clone());
            }
        }
        if let Some(probe) = probes.get(&endpoint.id) {
            evidence_refs.push(format!("run:{}", probe.run_id));
            match probe.status {
                SettlementStatus::Accepted => standing = ENDPOINT_DEMONSTRATED,
                _ => {
                    // A rejected probe still *happened*, so the ladder rung is
                    // `probed`; the rejection is what blocks selection. Reading
                    // this as `declared` would erase the attempt, and reading it
                    // as `demonstrated` would promote a failure.
                    standing = ENDPOINT_PROBED;
                    blocking_reason.get_or_insert_with(|| {
                        format!(
                            "last probe settled {}: {}",
                            enum_word(&probe.status),
                            if probe.basis.is_empty() {
                                "no basis recorded".to_string()
                            } else {
                                probe.basis.join(", ")
                            }
                        )
                    });
                }
            }
        }

        out.assets.push(AssetRow {
            asset_id: format!("agent_endpoint:{}", endpoint.id),
            kind: AssetKind::AgentEndpoint,
            name: endpoint.id.clone(),
            version: None,
            standing: standing.into(),
            identity_digest,
            evidence_refs,
            blocking_reason,
        });
    }
}

fn collect_extensions(root: &Path, out: &mut AssetListResult) {
    let Ok(registry) = ExtensionRegistry::load(root) else {
        out.unreadable.push("extension_registry".into());
        return;
    };
    for entry in registry.extensions {
        // Agent endpoints are registered here as runtime adapters too, but they
        // are projected above with their probe evidence attached. Listing them
        // twice would give the same endpoint two standings on one screen.
        if entry.extension_id.starts_with("agent_endpoint_") {
            continue;
        }
        let quarantined_trust = entry.trust_level == TrustLevel::Quarantined;
        let blocking_reason = match (&entry.status, quarantined_trust) {
            (_, true) => Some("extension trust level is quarantined".to_string()),
            (ExtensionStatus::Active, false) => None,
            (status, false) => Some(format!("extension status is {}", enum_word(status))),
        };
        out.assets.push(AssetRow {
            asset_id: format!("extension:{}@{}", entry.extension_id, entry.version),
            kind: AssetKind::Extension,
            name: entry.extension_id,
            version: Some(entry.version),
            standing: enum_word(&entry.status),
            identity_digest: Some(entry.descriptor_sha256),
            evidence_refs: entry.authority_ref.into_iter().collect(),
            blocking_reason,
        });
    }
}

/// A serde-renamed enum's wire word, for carrying a kernel vocabulary through
/// verbatim instead of re-spelling it here (where it could drift).
fn enum_word<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The invariant the whole contract rests on: `standing` and
    /// `blocking_reason` answer different questions, so a row may be at the top
    /// of its ladder and still blocked, or at the bottom and not blocked.
    /// Nothing in the projection may collapse them.
    #[test]
    fn standing_and_blocking_are_independent() {
        let row = AssetRow {
            asset_id: "agent_endpoint:x".into(),
            kind: AssetKind::AgentEndpoint,
            name: "x".into(),
            version: None,
            standing: ENDPOINT_DECLARED.into(),
            identity_digest: None,
            evidence_refs: Vec::new(),
            blocking_reason: None,
        };
        // A declared endpoint is unproven but not blocked.
        assert_eq!(row.standing, "declared");
        assert!(row.blocking_reason.is_none());
    }

    #[test]
    fn endpoint_standing_words_match_the_kernels_own_vocabulary() {
        // If `EndpointStatus` is ever renamed, these must move with it — the
        // whole point of the ladder is that it is the kernel's, not ours.
        use sea_forge_agent::EndpointStatus;
        assert_eq!(enum_word(&EndpointStatus::Declared), ENDPOINT_DECLARED);
        assert_eq!(enum_word(&EndpointStatus::Probed), ENDPOINT_PROBED);
        assert_eq!(
            enum_word(&EndpointStatus::Demonstrated),
            ENDPOINT_DEMONSTRATED
        );
    }

    #[test]
    fn empty_cell_lists_nothing_rather_than_a_builtin_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let result = list(dir.path(), &AgentConfig::default());
        assert!(result.assets.is_empty(), "{:?}", result.assets);
        assert!(result.unreadable.is_empty());
    }
}
