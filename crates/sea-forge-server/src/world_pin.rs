//! World pinning for the legacy convergence chain (CEP-0008, migration Stage 9).
//!
//! The E4 -> E5A -> E5B -> E6 library chain carries the `world_ref` of the
//! originating work request. This module is the one place that chain checks
//! it: syntax (via DomainForge's own `WorldRef` parser) and equality. It does
//! NOT verify the digest — that needs a registry and the carried identity, and
//! is done by `WorldRegistry` on the CEP authority path. Holding a pinned
//! string here means "well formed and consistent", not "verified".

use sea_forge_domainforge::WorldRef;

/// Syntax-only gate. An alias (`world:<name>`) or label is refused.
pub(crate) fn verify(candidate: &str) -> Result<(), String> {
    candidate
        .parse::<WorldRef>()
        .map(|_| ())
        .map_err(|e| format!("{candidate:?}: {e}"))
}

/// Fail unless `got` is exactly `expected`; absent is a mismatch, not a pass.
pub(crate) fn require_same(expected: &str, got: Option<&str>) -> Result<(), String> {
    match got {
        Some(g) if g == expected => Ok(()),
        Some(g) => Err(format!("expected {expected}, got {g}")),
        None => Err(format!("expected {expected}, got <missing>")),
    }
}

/// A payload's `world_ref`, required and syntax-checked.
pub(crate) fn from_payload(
    payload: &serde_json::Map<String, serde_json::Value>,
) -> Result<String, String> {
    match payload.get("world_ref").and_then(serde_json::Value::as_str) {
        Some(w) => verify(w).map(|()| w.to_string()),
        None => Err("world_ref is missing".to_string()),
    }
}
