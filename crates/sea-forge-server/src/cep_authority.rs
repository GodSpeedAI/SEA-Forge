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

type RegistryCache = Mutex<HashMap<String, Arc<WorldRegistry>>>;

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
fn load_worlds(config: &ServerConfig, root: &Path) -> Result<Arc<WorldRegistry>, String> {
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
        return Ok(Arc::clone(hit));
    }
    let mut registry = WorldRegistry::new();
    for (name, entry, files) in sets {
        registry
            .register_source_set(
                &name,
                &SeaSourceSet {
                    entry_uri: entry,
                    files,
                },
            )
            .map_err(|e| format!("world {name}: {e}"))?;
    }
    for (name, relative, content) in priors {
        registry
            .register_prior_identity_json(&name, &content)
            .map_err(|e| format!("world {name} prior identity {relative}: {e}"))?;
    }
    let registry = Arc::new(registry);
    let mut guard = cache()
        .lock()
        .map_err(|_| "world cache poisoned".to_string())?;
    if guard.len() >= CACHE_LIMIT {
        guard.clear();
    }
    guard.insert(key, Arc::clone(&registry));
    Ok(registry)
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
    let worlds = match load_worlds(config, root) {
        Ok(w) => w,
        Err(e) => return refusal("cep_world_config", e),
    };
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: worlds.as_ref(),
        root,
        approval_ttl: ttl(config),
        criteria: criteria(config),
    };
    f(&service)
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
