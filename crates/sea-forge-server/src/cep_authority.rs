//! `authority_request` verb body (migration Stage 6): load the cell's policy and
//! registered worlds, evaluate the CEP request as the verified caller, and
//! return the CEP `authority_decision` envelope. Fail-closed at every step.

use crate::config::ServerConfig;
use sea_forge_authority::cep::CepAuthorityService;
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::Actor;
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry, MAX_AGGREGATE_BYTES};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

fn refusal(class: &str, message: impl std::fmt::Display) -> Value {
    json!({"error": message.to_string(), "error_class": class, "no_side_effect": true})
}

/// Most distinct world sets kept; a world is immutable by digest, so entries never go stale.
const CACHE_LIMIT: usize = 8;

/// The registry and every configured `(name, world_ref)`, in configuration order.
type Loaded = (Arc<WorldRegistry>, Vec<(String, String)>);
type RegistryCache = Mutex<HashMap<String, Loaded>>;

/// Largest prior-identity document read. A `DomainModelIdentity` is a few hundred bytes.
const MAX_PRIOR_IDENTITY_BYTES: u64 = 64 * 1024;

fn cache() -> &'static RegistryCache {
    static CACHE: OnceLock<RegistryCache> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// Build (or reuse) the registry for the configured worlds. The cache key covers the
/// configuration *and* every file's bytes, so an edited file can never reuse a stale
/// registry: changed content is a different key and mints a different `world_ref`.
///
/// Prior identities are registered after every current world, so a prior that
/// equals a current identity is refused rather than silently changing its role.
/// Their bytes are part of the key like source bytes are.
fn load_worlds(config: &ServerConfig, root: &Path) -> Result<Loaded, String> {
    let mut key = Sha256::new();
    key.update(serde_json::to_vec(&config.cep_authority.worlds).map_err(|e| e.to_string())?);
    let mut sets = Vec::new();
    let mut priors: Vec<(String, String, String)> = Vec::new();
    for world in &config.cep_authority.worlds {
        sea_forge_core::path::validate_relative_path(&world.base).map_err(|e| e.to_string())?;
        sea_forge_core::path::validate_relative_path(&world.entry).map_err(|e| e.to_string())?;
        let base = root.join(&world.base);
        let mut files = Vec::new();
        let mut total = 0usize;
        for relative in &world.files {
            sea_forge_core::path::validate_relative_path(relative).map_err(|e| e.to_string())?;
            let content = std::fs::read_to_string(base.join(relative))
                .map_err(|e| format!("world {} file {relative}: {e}", world.name))?;
            total += content.len();
            if total > MAX_AGGREGATE_BYTES {
                return Err(format!(
                    "world {} exceeds {MAX_AGGREGATE_BYTES} bytes",
                    world.name
                ));
            }
            key.update(Sha256::digest(content.as_bytes()));
            files.push(SourceFile {
                uri: relative.clone(),
                sha256: hex::encode(Sha256::digest(content.as_bytes())),
                content,
            });
        }
        for relative in &world.prior_identities {
            let content = read_prior_identity(root, relative)?;
            key.update(Sha256::digest(content.as_bytes()));
            priors.push((world.name.clone(), relative.clone(), content));
        }
        sets.push((world.name.clone(), world.entry.clone(), files));
    }
    let key = hex::encode(key.finalize());
    if let Some(hit) = cache()
        .lock()
        .map_err(|_| "world cache poisoned".to_string())?
        .get(&key)
    {
        return Ok((Arc::clone(&hit.0), hit.1.clone()));
    }
    let mut registry = WorldRegistry::new();
    let mut loaded = Vec::new();
    for (name, entry, files) in sets {
        let world_ref = registry
            .register_source_set(
                &name,
                &SeaSourceSet {
                    entry_uri: entry,
                    files,
                },
            )
            .map_err(|e| format!("world {name}: {e}"))?;
        loaded.push((name, world_ref));
    }
    for (name, relative, content) in priors {
        registry
            .register_prior_identity_json(&name, &content)
            .map_err(|e| format!("world {name} prior identity {relative}: {e}"))?;
    }
    // Logged only once every world and prior identity has registered, since any failure discards the registry. The
    // configured name and the ref a caller must present differ; logging both lets an "unknown world" refusal be
    // matched to what was actually loaded.
    for (name, world_ref) in &loaded {
        tracing::info!(world = %name, world_ref = %world_ref, "cep_authority world loaded");
    }
    let registry = Arc::new(registry);
    let baseline = loaded
        .iter()
        .map(|(name, world_ref)| (name.clone(), world_ref.to_string()))
        .collect::<Vec<_>>();
    let mut guard = cache()
        .lock()
        .map_err(|_| "world cache poisoned".to_string())?;
    if guard.len() >= CACHE_LIMIT {
        guard.clear();
    }
    guard.insert(key, (Arc::clone(&registry), baseline.clone()));
    Ok((registry, baseline))
}

/// Read one prior-identity document under the cell root, size-bounded. Fails closed on a bad path or an
/// unreadable or oversized file; the document is parsed and digest-checked by the registry.
fn read_prior_identity(root: &Path, relative: &str) -> Result<String, String> {
    sea_forge_core::path::validate_relative_path(relative).map_err(|e| e.to_string())?;
    let path = root.join(relative);
    let len = std::fs::metadata(&path)
        .map_err(|e| format!("prior identity {relative}: {e}"))?
        .len();
    if len > MAX_PRIOR_IDENTITY_BYTES {
        return Err(format!(
            "prior identity {relative} exceeds {MAX_PRIOR_IDENTITY_BYTES} bytes"
        ));
    }
    std::fs::read_to_string(&path).map_err(|e| format!("prior identity {relative}: {e}"))
}

fn criteria(config: &ServerConfig) -> sea_forge_authority::cep::SettlementCriteria {
    sea_forge_authority::cep::SettlementCriteria {
        min_supporting: config.cep_authority.settlement.min_supporting,
        min_reliability: config.cep_authority.settlement.min_reliability.clone(),
    }
}

fn ttl(config: &ServerConfig) -> chrono::Duration {
    chrono::Duration::hours(config.cep_authority.approval_ttl_hours.clamp(1, 720) as i64)
}

/// Load policy and worlds and run `f` against a fresh service (a fresh engine per request, so an
/// escalation's in-memory opaque constraint never leaks between requests).
fn with_service(
    config: &ServerConfig,
    root: &Path,
    f: impl FnOnce(&CepAuthorityService<'_>) -> Value,
) -> Value {
    with_service_baseline(config, root, false, |service, _| f(service))
}

/// The baseline world: the pinned world until the first promotion. Explicit
/// configuration only (`cep_authority.pinned_baseline`), or the single
/// configured world. With several worlds and no named baseline there is none
/// (`Ok(None)`): the first configured world is never assumed.
fn baseline_of(
    config: &ServerConfig,
    loaded: &[(String, String)],
) -> Result<Option<String>, String> {
    match &config.cep_authority.pinned_baseline {
        Some(name) => loaded
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, r)| Some(r.clone()))
            .ok_or_else(|| format!("pinned_baseline {name:?} names no configured world")),
        None if loaded.len() == 1 => Ok(Some(loaded[0].1.clone())),
        None => Ok(None),
    }
}

/// [`with_service`], also giving the baseline world (see [`baseline_of`]).
/// `need_baseline`: the verb moves the pin, so an ambiguous baseline is refused.
fn with_service_baseline(
    config: &ServerConfig,
    root: &Path,
    need_baseline: bool,
    f: impl FnOnce(&CepAuthorityService<'_>, Option<&str>) -> Value,
) -> Value {
    if !config.cep_authority.enabled {
        return refusal(
            "cep_authority_disabled",
            "the CEP authority loop is not enabled for this cell",
        );
    }
    let policy_path =
        match crate::agent_probe::resolve_policy_path(root, &config.cep_authority.policy) {
            Ok(p) => p,
            Err(e) => return refusal("cep_authority_config", e),
        };
    let bundle = match AuthorityPolicyBundle::load(&policy_path) {
        Ok(b) => b,
        Err(e) => return refusal("cep_authority_config", e),
    };
    let engine = match PolicyAuthorityEngine::new(bundle.clone()) {
        Ok(e) => e,
        Err(e) => return refusal("cep_authority_config", e),
    };
    let (worlds, loaded) = match load_worlds(config, root) {
        Ok(w) => w,
        Err(e) => return refusal("cep_world_config", e),
    };
    let baseline = match baseline_of(config, &loaded) {
        Ok(b) => b,
        Err(e) => return refusal("cep_world_config", e),
    };
    if need_baseline && baseline.is_none() {
        return refusal(
            "cep_pin_baseline_ambiguous",
            "several worlds are configured and cep_authority.pinned_baseline names none: \
             the pinned world is not guessed, so the pin cannot be moved",
        );
    }
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: worlds.as_ref(),
        root,
        approval_ttl: ttl(config),
        criteria: criteria(config),
    };
    f(&service, baseline.as_deref())
}

/// RealityTrace's `regression-matrix.v1`, vendored by digest in
/// `contracts/rt/DIGESTS` (a gate test fails when the copy differs).
pub const REGRESSION_MATRIX_SCHEMA: &str =
    include_str!("../contracts/rt/regression-matrix.v1.json");

fn validate_matrix(matrix: &Value) -> Result<(), String> {
    static VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
    let validator = VALIDATOR.get_or_init(|| {
        let schema: Value =
            serde_json::from_str(REGRESSION_MATRIX_SCHEMA).expect("vendored schema is JSON");
        jsonschema::validator_for(&schema).expect("vendored schema compiles")
    });
    let errors: Vec<String> = validator
        .iter_errors(matrix)
        .take(5)
        .map(|e| format!("{}: {e}", e.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "the matrix is not a regression-matrix.v1 document: {}",
            errors.join("; ")
        ))
    }
}

fn pin_refusal(r: &sea_forge_authority::cep::PinRefusal) -> Value {
    json!({"error": r.to_string(), "error_class": "cep_promotion_refused", "code": r.code})
}

/// Promote the proposal's candidate world through the existing gated transition (`authority_promote`
/// verb). `proposal` and `matrix` are RealityTrace's `revision-proposal.v1` and `regression-matrix.v1`;
/// `envelope` is the CEP `authority_request` that names the approval.
pub fn promote(
    config: &ServerConfig,
    root: &Path,
    proposal: &Value,
    matrix: &Value,
    envelope: &Value,
    caller: &Actor,
) -> Value {
    // Schema first: RealityTrace's own contract, vendored by digest.
    if let Err(message) = validate_matrix(matrix) {
        return json!({"error": message, "error_class": "cep_promotion_refused", "code": "MATRIX_INVALID"});
    }
    with_service_baseline(config, root, true, |service, baseline| {
        let request = sea_forge_authority::cep::PromoteRequest {
            proposal: proposal.clone(),
            matrix: matrix.clone(),
            envelope: envelope.clone(),
        };
        match service.promote(&request, caller, baseline) {
            Ok(pin) => json!({"ok": true, "pin": pin}),
            Err(r) => pin_refusal(&r),
        }
    })
}

/// Re-pin to a world the cell was pinned to before, through the same gated transition (`authority_repin`).
pub fn repin(
    config: &ServerConfig,
    root: &Path,
    to_world_ref: &str,
    reason: &str,
    envelope: &Value,
    caller: &Actor,
) -> Value {
    with_service_baseline(config, root, true, |service, baseline| {
        let request = sea_forge_authority::cep::RepinRequest {
            to_world_ref: to_world_ref.to_string(),
            reason: reason.to_string(),
            envelope: envelope.clone(),
        };
        match service.repin(&request, caller, baseline) {
            Ok(pin) => json!({"ok": true, "pin": pin}),
            Err(r) => pin_refusal(&r),
        }
    })
}

/// The current pin, the history and the retained refused attempts (`authority_pin` verb; read-only).
pub fn pin(config: &ServerConfig, root: &Path) -> Value {
    with_service_baseline(config, root, false, |service, baseline| {
        match (service.pin_state(baseline), service.pin_attempts()) {
            (Ok(state), Ok(attempts)) => {
                let mut v = state.to_json();
                v["ok"] = json!(true);
                v["attempts"] = json!(attempts);
                v
            }
            (Err(e), _) | (_, Err(e)) => refusal(e.class(), e),
        }
    })
}

/// The world currently pinned to the cell, for the freeze check (WP-21,
/// `WORLD_NOT_PINNED`). `None`: the CEP authority loop is not enabled, so no pin
/// source exists and the check is not made. When it is enabled and the pin
/// cannot be read the answer is a value no world reference equals, so a freeze
/// fails closed rather than passing unchecked.
pub fn pinned_world(config: &ServerConfig, root: &Path) -> Option<String> {
    if !config.cep_authority.enabled {
        return None;
    }
    let state = pin(config, root);
    Some(
        state["current"]
            .as_str()
            .map_or_else(|| "pin-unavailable".to_string(), String::from),
    )
}

/// Resolve an escalation as the verified `resolver` (`authority_approval` verb).
pub fn resolve(
    config: &ServerConfig,
    root: &Path,
    approval_id: &str,
    approve: bool,
    resolver: &Actor,
) -> Value {
    with_service(config, root, |service| {
        match service.resolve_approval(approval_id, approve, resolver) {
            Ok(done) => done,
            Err(e) => refusal(e.class(), e),
        }
    })
}

/// Every approvable escalation and where it stands (`authority_approvals` verb; read-only).
pub fn list(config: &ServerConfig, root: &Path) -> Value {
    with_service(config, root, |service| match service.list_approvals() {
        Ok(all) => {
            json!({"ok": true, "approvals": all.iter().map(|a| a.to_json()).collect::<Vec<_>>()})
        }
        Err(e) => refusal(e.class(), e),
    })
}

/// Every allowed world transition (`authority_transitions` verb; read-only). Derived from the ledger.
pub fn transitions(config: &ServerConfig, root: &Path) -> Value {
    with_service(config, root, |service| match service.transitions() {
        Ok(all) => {
            json!({"ok": true, "transitions": all.iter().map(|t| t.to_json()).collect::<Vec<_>>()})
        }
        Err(e) => refusal(e.class(), e),
    })
}

pub fn respond(config: &ServerConfig, root: &Path, envelope: &Value, caller: &Actor) -> Value {
    with_service(config, root, |service| {
        match service.decide(envelope, caller) {
            Ok(out) => json!({
                "ok": true,
                "envelope": out.envelope,
                "decision_id": out.decision.decision_id,
                "ledger_entry": out.committed.entry_ulid(),
            }),
            Err(e) => refusal(e.class(), e),
        }
    })
}

/// Accept a RealityTrace-derived `evidence_packet` for an allowed operation (`authority_evidence` verb).
pub fn submit_evidence(
    config: &ServerConfig,
    root: &Path,
    packet: &Value,
    caller: &Actor,
) -> Value {
    with_service(config, root, |service| {
        let source = config.cep_authority.realitytrace.clone().map(|rt| {
            (
                rt.require_corroboration,
                crate::realitytrace_source::RealityTraceSource::new(rt),
            )
        });
        let corroboration = match &source {
            Some((required, source)) => sea_forge_authority::cep::Corroboration::Source {
                source,
                required: *required,
            },
            None => sea_forge_authority::cep::Corroboration::None,
        };
        match service.submit_evidence_with(packet, caller, &corroboration) {
            Ok(ack) => json!({
                "ok": true,
                "operation_id": ack.operation_id,
                "envelope_id": ack.envelope_id,
                "items": ack.items,
                "corroboration": ack.corroborated_by.unwrap_or("none"),
                "ledger_entry": ack.committed.entry_ulid(),
            }),
            Err(e) => refusal(e.class(), e),
        }
    })
}

/// Return only fields read back from the exact committed ledger record. The hash is an
/// opaque record reference, not an independent proof of the selected fields.
fn project_receipt(
    root: &Path,
    operation_id: &str,
    outcome: &sea_forge_authority::cep::SettlementOutcome,
) -> Option<Value> {
    // Replay references are currently reconstructed from an unauthenticated snapshot.
    // A nonfinal commit can also return an indistinguishable prior winner.
    if outcome.replayed || !outcome.final_status {
        return None;
    }
    let stream = sea_forge_ledger::LedgerStream::open(
        root,
        outcome.committed.ledger_id(),
        "sea-forge-cep-authority",
    )
    .ok()?;
    let entry = stream
        .read_entries()
        .ok()?
        .into_iter()
        .find(|e| e.entry_ulid == outcome.committed.entry_ulid())?;
    if outcome.committed.ledger_id() != sea_forge_authority::cep::LEDGER_ID
        || sea_forge_ledger::types::payload_hash(&entry.payload).ok()? != entry.payload_hash
        || sea_forge_ledger::types::entry_hash(&entry).ok()? != entry.entry_hash
        || entry.committed_ref() != outcome.committed
        || entry.record_kind != "settlement_packet"
        || entry.subject_refs != ["cognate", operation_id]
    {
        return None;
    }
    let packet = &entry.payload;
    let text = |pointer: &str| {
        packet
            .pointer(pointer)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control))
    };
    let op = text("/extensions/godspeed.settlement_packet/operation_id")?;
    let world = text("/scope/world_ref")?;
    let decision = text("/extensions/godspeed.settlement_packet/authority_decision_ref")?;
    let issuer = text("/created_by")?;
    let envelope_id = text("/envelope_id")?;
    let status = text("/extensions/godspeed.settlement_packet/overall_status")?;
    let finality = packet
        .pointer("/extensions/godspeed.settlement_packet/final")?
        .as_bool()?;
    if op != operation_id
        || issuer != "sea-forge-authority"
        || status != outcome.status
        || finality != outcome.final_status
        || (status != "unsettled" && !finality)
        || !matches!(status, "settled" | "unsettled" | "rejected")
        || packet.get("envelope_kind").and_then(Value::as_str) != Some("settlement_packet")
        || packet
            .pointer("/extensions/cep.profile/profile_id")
            .and_then(Value::as_str)
            != Some("godspeed.settlement_packet")
        || packet
            .pointer("/extensions/cep.profile/profile_version")
            .and_then(Value::as_str)
            != Some("1.0.0")
    {
        return None;
    }
    Some(
        json!({"representation": "verified_projection", "operation_id": op,
        "subject_refs": entry.subject_refs, "world_ref": world, "authority_decision_ref": decision,
        "issuer": issuer, "status": status, "final": finality, "ledger_id": entry.ledger_id,
        "ledger_entry": entry.entry_ulid, "payload_hash": entry.payload_hash, "envelope_id": envelope_id}),
    )
}

fn receipt_response(
    root: &Path,
    operation_id: &str,
    outcome: &sea_forge_authority::cep::SettlementOutcome,
) -> Value {
    match project_receipt(root, operation_id, outcome) {
        Some(receipt) => json!({"ok": true, "replayed": outcome.replayed, "receipt": receipt}),
        None => refusal(
            "invalid_settlement",
            "committed settlement cannot be projected",
        ),
    }
}

/// Settle an operation from its committed evidence (`authority_settle` verb).
pub fn settle(
    config: &ServerConfig,
    root: &Path,
    operation_id: &str,
    caller: &Actor,
    receipt_only: bool,
) -> Value {
    with_service(config, root, |service| {
        let required = config
            .cep_authority
            .realitytrace
            .as_ref()
            .filter(|rt| rt.require_corroboration)
            .map(|_| "realitytrace");
        let source = config
            .cep_authority
            .realitytrace
            .clone()
            .map(crate::realitytrace_source::RealityTraceSource::new);
        // An operation whose allow decision asserted a modality is settled from RealityTrace's ledger.
        let outcome = match service.asserts_obligation(operation_id) {
            Ok(true) => match &source {
                Some(source) => service.settle_obligation(operation_id, caller, source),
                None => Err(sea_forge_authority::cep::CepAuthorityError::Evidence(
                    "obligation settlement needs RealityTrace configured (cep_authority.realitytrace)".into(),
                )),
            },
            Ok(false) => service.settle_with(operation_id, caller, required),
            Err(e) => Err(e),
        };
        match outcome {
            Ok(done) if receipt_only => receipt_response(root, operation_id, &done),
            Ok(done) => json!({
                "ok": true,
                "status": done.status,
                "final": done.final_status,
                "replayed": done.replayed,
                "envelope": done.envelope,
                "ledger_entry": done.committed.entry_ulid(),
            }),
            Err(e) => refusal(e.class(), e),
        }
    })
}

/// Record the closure of an obligation's window (`authority_closure` verb). SEA-Forge reads RealityTrace's
/// ledger head itself and stamps the closure with its own clock.
pub fn closure(
    config: &ServerConfig,
    root: &Path,
    operation_id: &str,
    discrimination_observation_id: &str,
    verification_observation_ids: &[String],
    caller: &Actor,
) -> Value {
    with_service(config, root, |service| {
        let Some(rt) = config.cep_authority.realitytrace.clone() else {
            return json!({
                "error": "obligation closure needs RealityTrace configured (cep_authority.realitytrace)",
                "error_class": "cep_evidence_refused",
            });
        };
        let source = crate::realitytrace_source::RealityTraceSource::new(rt);
        match service.record_closure(
            operation_id,
            discrimination_observation_id,
            verification_observation_ids,
            caller,
            &source,
        ) {
            Ok(ack) => json!({
                "ok": true,
                "operation_id": operation_id,
                "b": ack.b,
                "c": ack.c,
                "committed_at": ack.committed_at,
                "on_time": ack.on_time,
                "replayed": ack.replayed,
                "ledger_entry": ack.entry_ulid,
            }),
            Err(e) => refusal(e.class(), e),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_forge_authority::cep::CepAuthorityError;

    fn fixture_packet() -> Value {
        json!({
            "envelope_id": "env-settlement_packet-op-1-test", "envelope_kind": "settlement_packet",
            "created_by": "sea-forge-authority", "scope": {"world_ref": format!("world:demo@sha256:{}", "a".repeat(64))},
            "questions": [{"question_form": "private-question-marker"}],
            "evidence": [{"description": "private-evidence-marker"}],
            "extensions": {
                "cep.profile": {"profile_id": "godspeed.settlement_packet", "profile_version": "1.0.0"},
                "godspeed.settlement_packet": {"operation_id": "op-1", "authority_decision_ref": "env-authority_decision-op-1", "overall_status": "unsettled", "final": true}
            }
        })
    }

    fn project_fixture(packet: Value, subjects: Vec<String>) -> Option<Value> {
        let root = tempfile::tempdir().unwrap();
        let ledger =
            sea_forge_ledger::LedgerStream::open(root.path(), "cognate-authority", "test").unwrap();
        let committed = ledger
            .commit_typed_new("settlement_packet", "test", subjects, &packet, vec![])
            .unwrap();
        let outcome = sea_forge_authority::cep::SettlementOutcome {
            status: "unsettled",
            final_status: true,
            envelope: packet,
            committed,
            replayed: false,
        };
        project_receipt(root.path(), "op-1", &outcome)
    }

    #[test]
    fn receipt_projection_preserves_final_unsettled_without_copying_evidence() {
        let receipt =
            project_fixture(fixture_packet(), vec!["cognate".into(), "op-1".into()]).unwrap();
        assert_eq!(receipt["status"], "unsettled");
        assert_eq!(receipt["final"], true);
        assert!(!receipt.to_string().contains("private-question-marker"));
        assert!(!receipt.to_string().contains("private-evidence-marker"));
    }

    #[test]
    fn receipt_projection_refuses_missing_and_mismatched_source_references() {
        let good_subjects = || vec!["cognate".into(), "op-1".into()];
        let mut missing_decision = fixture_packet();
        missing_decision["extensions"]["godspeed.settlement_packet"]
            .as_object_mut()
            .unwrap()
            .remove("authority_decision_ref");
        assert!(project_fixture(missing_decision, good_subjects()).is_none());
        let mut wrong_operation = fixture_packet();
        wrong_operation["extensions"]["godspeed.settlement_packet"]["operation_id"] =
            json!("op-other");
        assert!(project_fixture(wrong_operation, good_subjects()).is_none());
        let mut wrong_issuer = fixture_packet();
        wrong_issuer["created_by"] = json!("other");
        assert!(project_fixture(wrong_issuer, good_subjects()).is_none());
        assert!(
            project_fixture(fixture_packet(), vec!["cognate".into(), "op-other".into()]).is_none()
        );
    }

    #[test]
    fn receipt_projection_preserves_replay_and_nonfinal_outcomes() {
        let root = tempfile::tempdir().unwrap();
        let ledger =
            sea_forge_ledger::LedgerStream::open(root.path(), "cognate-authority", "test").unwrap();
        let packet = fixture_packet();
        let committed = ledger
            .commit_typed_new(
                "settlement_packet",
                "test",
                vec!["cognate".into(), "op-1".into()],
                &packet,
                vec![],
            )
            .unwrap();
        let replay = sea_forge_authority::cep::SettlementOutcome {
            status: "unsettled",
            final_status: true,
            envelope: packet.clone(),
            committed: committed.clone(),
            replayed: true,
        };
        let replay_receipt = project_receipt(root.path(), "op-1", &replay).unwrap();
        assert_eq!(replay_receipt["ledger_entry"], committed.entry_ulid());
        assert_eq!(replay_receipt["final"], true);
        let mut packet = packet;
        packet["extensions"]["godspeed.settlement_packet"]["final"] = json!(false);
        let nonfinal_committed = ledger
            .commit_typed_new(
                "settlement_packet",
                "nonfinal",
                vec!["cognate".into(), "op-1".into()],
                &packet,
                vec![],
            )
            .unwrap();
        let nonfinal = sea_forge_authority::cep::SettlementOutcome {
            status: "unsettled",
            final_status: false,
            envelope: packet,
            committed: nonfinal_committed.clone(),
            replayed: false,
        };
        let nonfinal_receipt = project_receipt(root.path(), "op-1", &nonfinal).unwrap();
        assert_eq!(
            nonfinal_receipt["ledger_entry"],
            nonfinal_committed.entry_ulid()
        );
        assert_eq!(nonfinal_receipt["final"], false);
    }

    #[test]
    fn receipt_projection_refuses_corruption_before_the_committed_entry() {
        let root = tempfile::tempdir().unwrap();
        let ledger =
            sea_forge_ledger::LedgerStream::open(root.path(), "cognate-authority", "test").unwrap();
        ledger
            .commit_typed_new(
                "test_record",
                "prior",
                vec![],
                &json!({"value": "prior-marker"}),
                vec![],
            )
            .unwrap();
        let packet = fixture_packet();
        let committed = ledger
            .commit_typed_new(
                "settlement_packet",
                "test",
                vec!["cognate".into(), "op-1".into()],
                &packet,
                vec![],
            )
            .unwrap();
        let outcome = sea_forge_authority::cep::SettlementOutcome {
            status: "unsettled",
            final_status: true,
            envelope: packet,
            committed,
            replayed: true,
        };
        let path = ledger.entries_path();
        let original = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, original.replace("prior-marker", "alter-marker")).unwrap();
        assert!(project_receipt(root.path(), "op-1", &outcome).is_none());
    }

    #[test]
    fn receipt_projection_refuses_a_tampered_committed_payload() {
        let root = tempfile::tempdir().unwrap();
        let ledger =
            sea_forge_ledger::LedgerStream::open(root.path(), "cognate-authority", "test").unwrap();
        let packet = fixture_packet();
        let committed = ledger
            .commit_typed_new(
                "settlement_packet",
                "test",
                vec!["cognate".into(), "op-1".into()],
                &packet,
                vec![],
            )
            .unwrap();
        let outcome = sea_forge_authority::cep::SettlementOutcome {
            status: "unsettled",
            final_status: true,
            envelope: packet,
            committed,
            replayed: false,
        };
        let path = ledger.entries_path();
        let original = std::fs::read_to_string(&path).unwrap();
        std::fs::write(
            &path,
            original.replace("env-authority_decision-op-1", "env-authority_decision-op-2"),
        )
        .unwrap();
        assert!(project_receipt(root.path(), "op-1", &outcome).is_none());
    }

    #[test]
    fn failed_projection_never_claims_no_side_effect_after_settlement_commit() {
        let root = tempfile::tempdir().unwrap();
        let ledger =
            sea_forge_ledger::LedgerStream::open(root.path(), "cognate-authority", "test").unwrap();
        let packet = fixture_packet();
        let committed = ledger
            .commit_typed_new(
                "settlement_packet",
                "test",
                vec!["cognate".into(), "other-operation".into()],
                &packet,
                vec![],
            )
            .unwrap();
        let outcome = sea_forge_authority::cep::SettlementOutcome {
            status: "unsettled",
            final_status: true,
            envelope: packet,
            committed,
            replayed: false,
        };
        let response = receipt_response(root.path(), "op-1", &outcome);
        assert_eq!(response["error_class"], "invalid_settlement");
        assert_ne!(response["no_side_effect"], true);
        assert_eq!(ledger.read_entries().unwrap().len(), 1);
    }

    #[test]
    fn settlement_pending_is_a_no_side_effect_retry_class_not_an_evidence_refusal() {
        let pending = refusal(
            CepAuthorityError::SettlementPending("the window is still open".into()).class(),
            CepAuthorityError::SettlementPending("the window is still open".into()),
        );
        assert_eq!(
            pending["error_class"], "cep_settlement_pending",
            "{pending}"
        );
        assert_eq!(pending["no_side_effect"], true, "{pending}");
        assert!(pending["error"]
            .as_str()
            .unwrap()
            .contains("settlement pending"));

        let refused = refusal(
            CepAuthorityError::Evidence("no evidence".into()).class(),
            CepAuthorityError::Evidence("no evidence".into()),
        );
        assert_eq!(refused["error_class"], "cep_evidence_refused", "{refused}");
    }
}
