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
use std::path::Path;

fn refusal(class: &str, message: impl std::fmt::Display) -> Value {
    json!({"error": message.to_string(), "error_class": class, "no_side_effect": true})
}

fn load_worlds(config: &ServerConfig, root: &Path) -> Result<WorldRegistry, String> {
    let mut registry = WorldRegistry::new();
    for world in &config.cep_authority.worlds {
        let mut files = Vec::new();
        let mut total = 0usize;
        sea_forge_core::path::validate_relative_path(&world.base).map_err(|e| e.to_string())?;
        sea_forge_core::path::validate_relative_path(&world.entry).map_err(|e| e.to_string())?;
        let base = root.join(&world.base);
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
            files.push(SourceFile {
                uri: relative.clone(),
                sha256: format!("{:x}", Sha256::digest(content.as_bytes())),
                content,
            });
        }
        registry
            .register_source_set(
                &world.name,
                &SeaSourceSet {
                    entry_uri: world.entry.clone(),
                    files,
                },
            )
            .map_err(|e| format!("world {}: {e}", world.name))?;
    }
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
