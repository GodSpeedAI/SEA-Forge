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

/// Byte length of the authority ledger's entries file. The ledger is append-only, so a longer file
/// means a durable row was appended, whatever the call then returned.
fn ledger_bytes(root: &Path) -> Option<u64> {
    let stream = sea_forge_ledger::LedgerStream::open(
        root,
        sea_forge_authority::cep::LEDGER_ID,
        "sea-forge-cep-authority",
    )
    .ok()?;
    match std::fs::metadata(stream.entries_path()) {
        Ok(meta) => Some(meta.len()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Some(0),
        Err(_) => None,
    }
}

/// A receipt refusal: a stable code and a fixed public sentence. Stored payload text, caller-chosen ids
/// and upstream diagnostics never appear in it. `no_side_effect` is true only when no row was appended.
pub(crate) fn receipt_refusal(error_class: &str, message: &str, appended: bool) -> Value {
    json!({
        "error": message,
        "error_class": error_class,
        "reason_code": error_class,
        "no_side_effect": !appended,
    })
}

/// The wire form of a verified receipt: exactly the allowlisted keys.
fn receipt_json(r: &sea_forge_authority::cep::SettlementReceipt) -> Value {
    json!({
        "representation": "verified_projection",
        "operation_id": r.operation_id,
        "subject_refs": r.subject_refs,
        "world_ref": r.world_ref,
        "authority_decision_ref": r.authority_decision_ref,
        "issuer": r.issuer,
        "status": r.status,
        "final": r.final_status,
        "ledger_id": r.ledger_id,
        "ledger_entry": r.ledger_entry,
        "payload_hash": r.payload_hash,
        "envelope_id": r.envelope_id,
    })
}

/// The first-call minimal settlement receipt (`authority_settle_receipt` verb).
pub fn settle_receipt(
    config: &ServerConfig,
    root: &Path,
    operation_id: &str,
    caller: &Actor,
) -> Value {
    let before = ledger_bytes(root);
    let response = with_service(config, root, |service| {
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
        if let Err(e) = service.require_settlement_authority(operation_id, caller) {
            let (class, message) = receipt_error(&e);
            return receipt_refusal(class, message, false);
        }
        let outcome = match service.asserts_obligation(operation_id) {
            Ok(true) => match &source {
                Some(source) => service.settle_obligation_for_receipt(operation_id, caller, source),
                None => Err(sea_forge_authority::cep::CepAuthorityError::Evidence(
                    String::new(),
                )),
            },
            Ok(false) => service.settle_for_receipt(operation_id, caller, required),
            Err(e) => Err(e),
        };
        match outcome {
            Ok(receipt) => json!({"ok": true, "receipt": receipt_json(&receipt)}),
            Err(e) => {
                let (class, message) = receipt_error(&e);
                receipt_refusal(class, message, false)
            }
        }
    });
    if response.get("error").is_some() {
        // Truthful by construction: the flag follows the ledger, not the call's own bookkeeping.
        let appended = match (before, ledger_bytes(root)) {
            (Some(b), Some(a)) => a != b,
            _ => true,
        };
        let mut response = response;
        response["no_side_effect"] = json!(!appended);
        return response;
    }
    response
}

/// Stable class and fixed public sentence for a receipt refusal. Never the error's own Display text:
/// that can carry stored payload text, caller-chosen ids or upstream diagnostics.
fn receipt_error(e: &sea_forge_authority::cep::CepAuthorityError) -> (&'static str, &'static str) {
    use sea_forge_authority::cep::{CepAuthorityError as E, ReceiptRefusal};
    match e {
        E::Receipt(r) => (r.code(), r.message()),
        // The ledger itself refused to append to, or read from, a damaged history.
        E::Internal(m) if m.contains("ledger_integrity_error") => (
            ReceiptRefusal::IntegrityFailed.code(),
            ReceiptRefusal::IntegrityFailed.message(),
        ),
        E::SettlementPending(_) => (e.class(), "settlement is not yet decidable"),
        E::Evidence(_) | E::Approval(_) => {
            (e.class(), "the settlement receipt request was refused")
        }
        _ => (e.class(), "the settlement receipt could not be issued"),
    }
}

/// Settle an operation from its committed evidence (`authority_settle` verb).
pub fn settle(config: &ServerConfig, root: &Path, operation_id: &str, caller: &Actor) -> Value {
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
        // Settlement authority first, so an unauthorized caller gets the uniform refusal whatever the
        // operation or the RealityTrace configuration looks like.
        if let Err(e) = service.require_settlement_authority(operation_id, caller) {
            return refusal(e.class(), e);
        }
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
            Ok(done) => json!({
                "ok": true,
                "operation_id": operation_id,
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

/// Whether `caller` may still be handed a stored outcome of `method`. Only a successful settlement or
/// closure outcome carries a settlement packet; anything else is not gated. Executes nothing: a refusal here
/// leaves the stored outcome and every ledger untouched. `operation_id` comes from the stored outcome (or the
/// replayed request); its absence on a gated outcome is refused, never assumed.
pub fn authorize_stored_outcome(
    config: &ServerConfig,
    root: &Path,
    method: &str,
    outcome: &Value,
    operation_id: Option<&str>,
    caller: &Actor,
) -> Result<(), Value> {
    if !matches!(method, "authority.settle" | "authority.closure")
        || outcome.get("ok") != Some(&Value::Bool(true))
    {
        return Ok(());
    }
    let Some(operation_id) = operation_id.or_else(|| outcome["operation_id"].as_str()) else {
        return Err(refusal(
            "cep_evidence_refused",
            "the stored settlement outcome does not name its operation, so settlement authority cannot be re-checked",
        ));
    };
    let verdict = with_service(config, root, |service| {
        match service.authorize_settlement_replay(operation_id, caller) {
            Ok(()) => json!({"ok": true}),
            Err(e) => refusal(e.class(), e),
        }
    });
    if verdict.get("error").is_some() {
        Err(verdict)
    } else {
        Ok(())
    }
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

    // Receipt behaviour is tested against the real ledger in `receipt_tests` below.

    #[test]
    fn receipt_errors_never_echo_stored_or_upstream_text() {
        use sea_forge_authority::cep::{CepAuthorityError as E, ReceiptRefusal};
        for error in [
            E::Evidence("stored-modality-marker upstream-divergence-marker".into()),
            E::Approval("stored-modality-marker".into()),
            E::Internal("stored-modality-marker".into()),
            E::Malformed("stored-modality-marker"),
            E::World("stored-modality-marker".into()),
        ] {
            let (class, message) = receipt_error(&error);
            let wire = receipt_refusal(class, message, false).to_string();
            assert!(!wire.contains("marker"), "{wire}");
        }
        let (class, _) = receipt_error(&E::Internal("ledger_integrity_error: chain".into()));
        assert_eq!(class, ReceiptRefusal::IntegrityFailed.code());
        for r in [
            ReceiptRefusal::Replay,
            ReceiptRefusal::NonFinal,
            ReceiptRefusal::IntegrityFailed,
            ReceiptRefusal::ReferenceMismatch,
        ] {
            assert!(r.code().starts_with("receipt_"));
            assert!(!r.message().is_empty());
        }
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
