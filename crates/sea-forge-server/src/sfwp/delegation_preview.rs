//! `delegation.preview` — the SFWP **inspect** projection of the job contract a
//! delegation would be governed by (Task 10, ADR-003 additive; epic 9.4).
//!
//! # The gap this closes
//!
//! The kernel has spoken `delegate` since M12, and `asset.list` (Task 9) made
//! endpoints enumerable with real standing. What was still missing is the step
//! epic 9.4 actually describes: *before* committing, see the complete job
//! contract — which model, which caps, which timeout, which retention, which
//! authority — and whether this endpoint can lawfully be used at all. Without
//! it the only way to learn what a delegation would do is to run one, which is
//! precisely the side effect the operator was trying to decide about.
//!
//! # What this method is not
//!
//! It is **not** an authority decision. `delegate` evaluates
//! [`sea_forge_authority::PolicyAuthorityEngine`] against a committed intent,
//! plan, and criteria, and writes the resulting decision to the ledger. This
//! method commits nothing, so it has nothing to evaluate *against* and no
//! ledger entry to record a verdict in. It reports the authority action that
//! *will be* submitted ([`JobContractPreview::required_authority`]) and stops
//! there. A preview verdict would be an authority claim with no record behind
//! it — exactly the unrecorded grant the kernel refuses (authority before side
//! effects; a projection must never become truth).
//!
//! For the same reason [`DelegationPreviewResult::eligible`] means only "no
//! precondition known at preview time is unmet". It is never a prediction that
//! `delegate` will be allowed.
//!
//! # Why the preconditions are not re-implemented here
//!
//! A preview's entire value is that it agrees with execution about what is
//! runnable. So the four request-shape rules live in
//! [`crate::delegation::check_preconditions`] and both callers use them: the
//! command refuses on the first unmet rule, this method lists all of them.
//! Endpoint standing and the blocking conditions that come from *evidence*
//! (an invalid descriptor, a last probe that settled rejected) are read from
//! [`crate::sfwp::assets`] for the same reason — the catalog and the preview
//! must never disagree about whether an endpoint is usable.
//!
//! # Why value provenance is carried
//!
//! `summarized` chosen for this endpoint and `summarized` because nobody ever
//! chose anything are different facts, and an operator signing off on a
//! delegation needs to tell them apart. So resolved values travel with a
//! [`ValueSource`] rather than as bare strings. Where the source genuinely
//! cannot be recovered, the narrower claim is made rather than a guessed one —
//! see [`ValueSource::CellDefault`].

use std::path::Path;

use schemars::JsonSchema;
use sea_forge_agent::{AgentConfig, AgentEndpointConfig, TranscriptRetentionMode};
use sea_forge_core::types::AuthorityAction;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::delegation::{action_for_delegation, check_preconditions};
use crate::sfwp::assets;

/// `delegation.preview` parameters.
///
/// Deliberately the same inputs `Request::Delegate` accepts and no others. A
/// preview that took a knob the command cannot take (a per-request retention
/// override, say) would describe a delegation nobody can actually run.
///
/// `max_turns` defaults to `0` rather than to some runnable number: an unset
/// turn cap is not a runnable delegation, and the preview says so through the
/// same rule execution uses instead of inventing a cap on the operator's
/// behalf.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct DelegationPreviewParams {
    pub endpoint: String,
    #[serde(default)]
    pub instruction: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_turns: u32,
    #[serde(default)]
    pub token_budget: Option<u64>,
}

/// Where a resolved value came from.
///
/// Closed on purpose. Each variant is a distinct, recoverable fact about who
/// decided the value; a variant that could not be told apart from another would
/// be provenance theatre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueSource {
    /// The caller asked for this value explicitly in the preview request.
    Requested,
    /// The endpoint descriptor in `server.yaml` declares it.
    EndpointDefault,
    /// The cell-wide `[agent]` configuration supplied it.
    ///
    /// This does *not* separate "the cell chose this" from "the cell left it
    /// alone and the kernel's fallback applied", because `AgentConfig`'s
    /// retention field is not an `Option` — after parsing, an explicit
    /// `summarized` and an absent one are the same value. Reporting a
    /// `built_in` here would be inventing a distinction the record does not
    /// hold.
    CellDefault,
    /// Nothing declared it at any level and the kernel's own fallback applies.
    /// Used only where the config type preserves the absence (`default_model`
    /// is an `Option`, so this is real for the model).
    BuiltIn,
}

/// One resolved value together with who decided it.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResolvedValue {
    pub value: String,
    pub source: ValueSource,
}

/// The job contract a delegation would run under.
///
/// Every field is derived from the endpoint descriptor and the request; nothing
/// here is a claim about what *did* happen, and nothing is stored.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct JobContractPreview {
    /// `openai_compatible` | `anthropic` | `acp`, taken from the authority
    /// action rather than re-matched, so it cannot drift from what is submitted.
    pub provider_kind: String,
    /// `descriptor_config_sha256` — the identity of the endpoint configuration
    /// this contract was built from.
    pub endpoint_digest: String,
    pub model: ResolvedValue,
    pub transcript_retention: ResolvedValue,
    /// Hash of the instruction packet. The instruction text itself is never
    /// echoed back: the caller already has it, and a projection that repeated
    /// it would become a second place the packet lives.
    pub instruction_sha256: String,
    pub instruction_bytes: u64,
    pub max_request_bytes: u64,
    pub max_response_bytes: u64,
    pub timeout_secs: u64,
    pub max_turns: u32,
    /// Absent means no token cap — the delegation is bounded by `max_turns`
    /// alone. Absence is reported as absence, not as a zero budget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<u64>,
    /// The authority action kind `delegate` will submit for evaluation
    /// (`agent_task`). Naming the action is not obtaining it: no decision
    /// exists until the command runs.
    pub required_authority: String,
    /// Identity of this exact contract. If the endpoint descriptor or any
    /// request value changes between preview and `delegate`, this digest
    /// changes — so an operator can tell whether what they inspected is what
    /// would run.
    pub contract_digest: String,
}

/// `delegation.preview` result body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DelegationPreviewResult {
    pub endpoint: String,
    /// True only when no precondition known at preview time is unmet *and* a
    /// contract could be built. Never a prediction that `delegate` will be
    /// allowed — see the module docs.
    pub eligible: bool,
    /// The endpoint's standing word from `asset.list`, verbatim. Absent when
    /// the endpoint is not configured in this cell: an unconfigured endpoint
    /// has no standing, and minting a word for it would put a fourth rung on a
    /// three-rung ladder the kernel owns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standing: Option<String>,
    /// Every unmet condition, not just the first. An operator repairing a
    /// request wants the whole list rather than one round trip per problem.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_reasons: Vec<String>,
    /// The evidence behind the standing, as `asset.list` reports it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    /// Absent when the endpoint is unknown or its configuration will not
    /// snapshot. There is then no contract to show, and assembling one from
    /// values that did not resolve would be the projection inventing truth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract: Option<JobContractPreview>,
}

/// Project the job contract for one prospective delegation. Creates no case,
/// run, ledger entry, or authority decision.
pub fn preview(
    root: &Path,
    agent: &AgentConfig,
    params: DelegationPreviewParams,
) -> DelegationPreviewResult {
    // The catalog is the one place endpoint standing is derived, so read it
    // rather than re-deriving: a preview that disagreed with `/assets` about
    // whether an endpoint is usable would make one of the two views a liar.
    let row = assets::list(root, agent)
        .assets
        .into_iter()
        .find(|row| row.kind == assets::AssetKind::AgentEndpoint && row.name == params.endpoint);
    let standing = row.as_ref().map(|row| row.standing.clone());
    let evidence_refs = row
        .as_ref()
        .map(|row| row.evidence_refs.clone())
        .unwrap_or_default();
    let mut blocking_reasons: Vec<String> = row
        .and_then(|row| row.blocking_reason)
        .into_iter()
        .collect();

    let unresolved = |blocking_reasons: Vec<String>| DelegationPreviewResult {
        endpoint: params.endpoint.clone(),
        eligible: false,
        standing: standing.clone(),
        blocking_reasons,
        evidence_refs: evidence_refs.clone(),
        contract: None,
    };

    let Some(endpoint) = agent.endpoint(&params.endpoint) else {
        blocking_reasons.push(format!(
            "agent endpoint '{}' is not configured in this cell",
            params.endpoint
        ));
        return unresolved(blocking_reasons);
    };
    // A descriptor that will not snapshot has already been reported by
    // `assets::list` as this endpoint's blocking reason — it snapshots the same
    // bytes — so there is nothing to add here, and nothing to build a contract
    // from either: the values a contract is made of are exactly the ones that
    // failed to resolve.
    let Ok(snapshot) = endpoint.snapshot() else {
        return unresolved(blocking_reasons);
    };

    let model = match check_preconditions(
        &snapshot,
        &params.instruction,
        params.model.as_deref(),
        params.max_turns,
    ) {
        Ok(model) => model,
        Err(unmet) => {
            blocking_reasons.extend(unmet);
            // The contract is still worth showing: the endpoint resolved, and
            // an operator fixing an over-long instruction needs to see the cap
            // they are up against. `snapshot.model` stands in for a model that
            // did not resolve — it is the value execution would have used.
            snapshot.model.clone()
        }
    };

    let instruction_sha256 = format!("sha256:{:x}", Sha256::digest(params.instruction.as_bytes()));
    let action = action_for_delegation(
        &snapshot,
        &params.endpoint,
        &model,
        &instruction_sha256,
        params.max_turns,
        params.token_budget,
    );
    // Digest the struct, not a re-parsed `Value`: field order is declaration
    // order and therefore stable, whereas a `Value` round trip depends on
    // whether `serde_json` was built with `preserve_order`.
    let action_bytes = serde_json::to_vec(&action).unwrap_or_default();
    let contract_digest = format!("sha256:{:x}", Sha256::digest(&action_bytes));

    let contract = JobContractPreview {
        provider_kind: action_field(&action, "provider_kind"),
        endpoint_digest: snapshot.descriptor_config_sha256.clone(),
        model: resolve_model(params.model.as_deref(), endpoint, &snapshot.model),
        transcript_retention: resolve_retention(endpoint, agent),
        instruction_sha256,
        instruction_bytes: params.instruction.len() as u64,
        max_request_bytes: snapshot.max_request_bytes as u64,
        max_response_bytes: snapshot.max_response_bytes as u64,
        timeout_secs: snapshot.timeout.as_secs(),
        max_turns: params.max_turns,
        token_budget: params.token_budget,
        required_authority: action_field(&action, "kind"),
        contract_digest,
    };

    DelegationPreviewResult {
        endpoint: params.endpoint,
        eligible: blocking_reasons.is_empty(),
        standing,
        blocking_reasons,
        evidence_refs,
        contract: Some(contract),
    }
}

/// Read one string field off the serialized authority action.
///
/// Going through serde rather than re-matching on `ProviderKind` keeps the
/// preview's words identical to the ones the authority engine will see; a
/// hand-written match would be a second spelling free to drift.
fn action_field(action: &AuthorityAction, field: &str) -> String {
    serde_json::to_value(action)
        .ok()
        .and_then(|value| {
            value
                .get(field)
                .and_then(|found| found.as_str().map(str::to_owned))
        })
        .unwrap_or_default()
}

fn resolve_model(
    requested: Option<&str>,
    endpoint: &AgentEndpointConfig,
    snapshot_model: &str,
) -> ResolvedValue {
    let source = if requested.is_some_and(|model| !model.is_empty()) {
        ValueSource::Requested
    } else if endpoint.default_model.is_some() {
        ValueSource::EndpointDefault
    } else {
        // `EndpointSnapshot` substitutes the literal `"default"` when nothing
        // declares a model. That is the kernel's own fallback, and
        // `default_model` being an `Option` is what makes it distinguishable.
        ValueSource::BuiltIn
    };
    let value = requested
        .filter(|model| !model.is_empty())
        .unwrap_or(snapshot_model)
        .to_owned();
    ResolvedValue { value, source }
}

fn resolve_retention(endpoint: &AgentEndpointConfig, agent: &AgentConfig) -> ResolvedValue {
    let source = if endpoint.transcript_retention.is_some() {
        ValueSource::EndpointDefault
    } else {
        ValueSource::CellDefault
    };
    // The same chain `case_dispatch` and `delegate` resolve through. Only the
    // provenance is computed here; the value itself is never independently
    // decided, or the preview could report a mode execution would not use.
    // `None` as the item override cannot fail, so the fallback is unreachable.
    let mode = TranscriptRetentionMode::resolve(None, Some(endpoint), agent).unwrap_or_default();
    ResolvedValue {
        value: retention_word(mode),
        source,
    }
}

/// The wire word for a retention mode, taken from its own serde rename so the
/// preview cannot spell it differently from the plan item `delegate` writes.
fn retention_word(mode: TranscriptRetentionMode) -> String {
    serde_json::to_value(mode)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_forge_agent::ProviderKind;

    fn endpoint(id: &str) -> AgentEndpointConfig {
        AgentEndpointConfig {
            id: id.into(),
            kind: ProviderKind::OpenAiCompatible,
            base_url: Some("https://api.example.invalid/v1".into()),
            argv: vec![],
            env: vec![],
            credential_ref: Some("EXAMPLE_KEY".into()),
            default_model: None,
            allow_loopback_test: false,
            max_request_bytes: 4096,
            max_response_bytes: 4096,
            timeout_secs: 30,
            status: None,
            transcript_retention: None,
        }
    }

    #[test]
    fn retention_words_match_the_modes_the_plan_item_records() {
        assert_eq!(
            retention_word(TranscriptRetentionMode::Summarized),
            "summarized"
        );
        assert_eq!(retention_word(TranscriptRetentionMode::Full), "full");
    }

    /// The provenance distinction the operator is being shown must be real: an
    /// endpoint that declares a model and one that does not resolve to
    /// different sources, not merely different strings.
    #[test]
    fn model_provenance_separates_requested_endpoint_and_built_in() {
        let mut config = endpoint("local");
        let built_in = resolve_model(None, &config, "default");
        assert_eq!(built_in.source, ValueSource::BuiltIn);
        assert_eq!(built_in.value, "default");

        config.default_model = Some("endpoint-model".into());
        let declared = resolve_model(None, &config, "endpoint-model");
        assert_eq!(declared.source, ValueSource::EndpointDefault);
        assert_eq!(declared.value, "endpoint-model");

        let asked = resolve_model(Some("asked-model"), &config, "endpoint-model");
        assert_eq!(asked.source, ValueSource::Requested);
        assert_eq!(asked.value, "asked-model");
    }

    /// An endpoint-level retention override must not be reported as the cell's
    /// choice; that is the exact confusion the source field exists to prevent.
    #[test]
    fn retention_provenance_tracks_the_level_that_declared_it() {
        let agent = AgentConfig {
            transcript_retention: TranscriptRetentionMode::Full,
            ..AgentConfig::default()
        };
        let mut config = endpoint("local");

        let from_cell = resolve_retention(&config, &agent);
        assert_eq!(from_cell.source, ValueSource::CellDefault);
        assert_eq!(from_cell.value, "full");

        config.transcript_retention = Some(TranscriptRetentionMode::Summarized);
        let from_endpoint = resolve_retention(&config, &agent);
        assert_eq!(from_endpoint.source, ValueSource::EndpointDefault);
        assert_eq!(from_endpoint.value, "summarized");
    }

    /// The action kind is what `delegate` submits for authority evaluation. If
    /// this stops reading `agent_task`, the preview is naming the wrong gate.
    #[test]
    fn required_authority_is_read_off_the_action_serde_writes() {
        let snapshot = endpoint("local").snapshot().expect("endpoint snapshots");
        let action = action_for_delegation(&snapshot, "local", "m", "sha256:abc", 3, None);
        assert_eq!(action_field(&action, "kind"), "agent_task");
        assert_eq!(action_field(&action, "provider_kind"), "openai_compatible");
    }
}
