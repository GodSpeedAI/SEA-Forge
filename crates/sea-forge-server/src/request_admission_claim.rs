//! Source-owned, durable request admission claims.
//!
//! The ledger claim precedes the correlation projection. A crash between the
//! two writes leaves an orphan, which callers must treat as ambiguous.
use sea_forge_core::errors::ForgeError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AdmissionOwner {
    actor_id: String,
    principal: String,
    gateway_actor_id: Option<String>,
}

impl AdmissionOwner {
    pub(crate) fn from_actor(actor: &crate::identity::ResolvedActor) -> Self {
        Self {
            actor_id: actor.actor_id().to_owned(),
            principal: actor.principal_label(),
            gateway_actor_id: actor.delegation().map(|d| d.gateway_actor_id().to_owned()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AdmissionClaim {
    pub request_id: String,
    pub payload_hash: String,
    pub owner: AdmissionOwner,
}

pub(crate) fn ledger_id(request_id: &str) -> String {
    format!(
        "request-admission-{}",
        hex::encode(Sha256::digest(request_id.as_bytes()))
    )
}

/// No directory means no claim. An existing but incomplete or invalid ledger
/// is ambiguous, never evidence that a request can execute again.
pub(crate) fn lookup(root: &Path, request_id: &str) -> Result<Option<AdmissionClaim>, ForgeError> {
    let id = ledger_id(request_id);
    let dir = root.join("ledgers").join(&id);
    match std::fs::symlink_metadata(&dir) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(ForgeError::io("inspect request admission", e)),
        Ok(meta) if !meta.file_type().is_dir() => {
            return Err(ForgeError::Internal(
                "invalid request admission ledger".into(),
            ))
        }
        Ok(_) => {}
    }
    // The same locked, chain-verified, symlink-refusing single read a receipt uses.
    let ledger = sea_forge_ledger::LedgerStream::open(root, id, "sea-forge-server")?;
    let entries = ledger.verified_snapshot_locked()?;
    if entries.len() != 1 || entries[0].record_kind != "request_admitted" {
        return Err(ForgeError::Internal(
            "ambiguous request admission ledger".into(),
        ));
    }
    let claim: AdmissionClaim = serde_json::from_value(entries[0].payload.clone())?;
    if claim.request_id != request_id || claim.payload_hash.is_empty() {
        return Err(ForgeError::Internal(
            "invalid request admission claim".into(),
        ));
    }
    Ok(Some(claim))
}

/// Called only under the correlation admission lock after confirming there is
/// neither a correlation record nor an existing claim.
pub(crate) fn record(root: &Path, claim: &AdmissionClaim) -> Result<(), ForgeError> {
    let ledger = sea_forge_ledger::LedgerStream::open(
        root,
        ledger_id(&claim.request_id),
        "sea-forge-server",
    )?;
    ledger.commit_typed(
        "request_admitted",
        vec![claim.request_id.clone()],
        claim,
        vec![],
    )?;
    Ok(())
}

/// (cell root, actor, request) triples already recorded as denied by this
/// process. The ledger remains the source of truth; this only stops repeated
/// identical polls from appending without bound. Cleared at the cap, and empty
/// after a restart (one more record then is acceptable).
fn denied_audit_polls(
) -> &'static std::sync::Mutex<std::collections::HashSet<(String, String, String)>> {
    static SET: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashSet<(String, String, String)>>,
    > = std::sync::OnceLock::new();
    SET.get_or_init(Default::default)
}

const DENIED_AUDIT_POLL_CAP: usize = 4096;

/// Blocking: loads policy and may append to the audit ledger. Call from a
/// blocking thread.
pub(crate) fn authorize_audit(
    config: &crate::ServerConfig,
    actor: &crate::identity::ResolvedActor,
    request_id: &str,
) -> Result<bool, ForgeError> {
    use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
    use sea_forge_core::types::{Actor, AuthorityAction, Verdict};

    let policy =
        crate::agent_probe::resolve_policy_path(&config.root, &config.cep_authority.policy)?;
    let bundle = AuthorityPolicyBundle::load(&policy)?;
    let engine = PolicyAuthorityEngine::new(bundle.clone())?;
    let actor = Actor {
        actor_id: actor.actor_id().into(),
        role: actor.role().clone(),
    };
    let action = AuthorityAction::Reserved {
        resource_type: "request_status_audit".into(),
        resource_id: request_id.into(),
        parameters: serde_json::json!({"request_id": request_id}),
    };
    let run_id = "request-status-audit";
    let item_id = "request_status_audit";
    let decision = engine.evaluate(AuthorityEvaluation {
        actor: &actor,
        binding: bundle.resolve_identity(&actor.actor_id, actor.role.clone()),
        run_id,
        case_id: "request-status-audit",
        plan_item_id: item_id,
        sequence: 1,
        action: &action,
        workspace_root: &config.root,
        evidence_refs: vec![],
        artifacts_root: None,
        timeout_secs: None,
        env_keys: Default::default(),
        opa_candidate: None,
        environment: None,
    })?;
    let poll_key = (
        config.root.to_string_lossy().into_owned(),
        actor.actor_id.clone(),
        request_id.to_owned(),
    );
    if decision.verdict != Verdict::Allow
        && denied_audit_polls()
            .lock()
            .is_ok_and(|seen| seen.contains(&poll_key))
    {
        return Ok(false);
    }
    let ledger = sea_forge_ledger::LedgerStream::open(
        &config.root,
        "request-status-audit",
        "sea-forge-server",
    )?;
    let committed = ledger.commit_typed(
        "authority_decision",
        vec![request_id.into()],
        &decision,
        vec![],
    )?;
    if decision.verdict != Verdict::Allow {
        if let Ok(mut seen) = denied_audit_polls().lock() {
            if seen.len() >= DENIED_AUDIT_POLL_CAP {
                seen.clear();
            }
            seen.insert(poll_key);
        }
        return Ok(false);
    }
    engine
        .grant(&decision, &committed, &action, None)?
        .authorize(&action, run_id, item_id, &config.root)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim() -> AdmissionClaim {
        AdmissionClaim {
            request_id: "req-one".into(),
            payload_hash: "payload-hash".into(),
            owner: AdmissionOwner {
                actor_id: "alice".into(),
                principal: "uid 1".into(),
                gateway_actor_id: None,
            },
        }
    }

    #[test]
    fn claim_is_durable_and_exactly_bound() {
        let root = tempfile::tempdir().unwrap();
        let claim = claim();
        assert_eq!(lookup(root.path(), &claim.request_id).unwrap(), None);
        record(root.path(), &claim).unwrap();
        assert_eq!(lookup(root.path(), &claim.request_id).unwrap(), Some(claim));
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_claim_ledger_file_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let claim = claim();
        record(root.path(), &claim).unwrap();
        let dir = root
            .path()
            .join("ledgers")
            .join(ledger_id(&claim.request_id));
        let entries = dir.join("entries.jsonl");
        let copy = root.path().join("elsewhere.jsonl");
        std::fs::copy(&entries, &copy).unwrap();
        std::fs::remove_file(&entries).unwrap();
        std::os::unix::fs::symlink(&copy, &entries).unwrap();
        assert!(lookup(root.path(), &claim.request_id).is_err());
    }

    #[test]
    fn a_claim_ledger_without_its_lock_file_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let claim = claim();
        record(root.path(), &claim).unwrap();
        let dir = root
            .path()
            .join("ledgers")
            .join(ledger_id(&claim.request_id));
        std::fs::remove_file(dir.join("stream.lock")).unwrap();
        assert!(lookup(root.path(), &claim.request_id).is_err());
    }

    #[test]
    fn incomplete_claim_ledger_is_ambiguous() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("ledgers").join(ledger_id("req-one"))).unwrap();
        assert!(lookup(root.path(), "req-one").is_err());
    }
}
