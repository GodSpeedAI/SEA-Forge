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

fn cache() -> &'static RegistryCache {
    static CACHE: OnceLock<RegistryCache> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// Build (or reuse) the registry for the configured worlds. The cache key covers the
/// configuration *and* every file's bytes, so an edited file can never reuse a stale
/// registry: changed content is a different key and mints a different `world_ref`.
fn load_worlds(config: &ServerConfig, root: &Path) -> Result<Arc<WorldRegistry>, String> {
    let mut key = Sha256::new();
    key.update(serde_json::to_vec(&config.cep_authority.worlds).map_err(|e| e.to_string())?);
    let mut sets = Vec::new();
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
                sha256: format!("{:x}", Sha256::digest(content.as_bytes())),
                content,
            });
        }
        sets.push((world.name.clone(), world.entry.clone(), files));
    }
    let key = format!("{:x}", key.finalize());
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

pub fn respond(config: &ServerConfig, root: &Path, envelope: &Value, caller: &Actor) -> Value {
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
        worlds: &worlds,
        root,
    };
    match service.decide(envelope, caller) {
        Ok(out) => json!({
            "ok": true,
            "envelope": out.envelope,
            "decision_id": out.decision.decision_id,
            "ledger_entry": out.committed.entry_ulid(),
        }),
        Err(e) => refusal(e.class(), e),
    }
}
