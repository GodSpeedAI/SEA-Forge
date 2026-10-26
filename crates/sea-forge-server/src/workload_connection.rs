//! Authentication by possession of an authority-issued private connection.
//!
//! The trusted launcher owns the issuer channel. Workloads receive only their
//! private endpoint, never the issuer or another workload's grant. Deployment
//! must prevent descriptor theft; this module does not isolate same-UID hosts.
use crate::identity::{ActorClaim, IdentityBindings, IdentityRefusal, PeerIdentity, ResolvedActor};
use sea_forge_authority::ActionGrant;
use sea_forge_core::types::{Actor, AuthorityAction};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;
use crate::Transport;

fn request_digest(request: &Value) -> String {
    hex::encode(Sha256::digest(request.to_string().as_bytes()))
}

/// The exact action the trusted issuer must authorize and commit before
/// creating a private connection. This grants authentication for one request;
/// the requested effect still needs its own ordinary authority decision.
pub fn authentication_action(principal: &str, actor: &Actor, request: &Value) -> AuthorityAction {
    AuthorityAction::Reserved {
        resource_type: "identity_minting".into(),
        resource_id: principal.into(),
        parameters: json!({"principal":principal,"actor":actor,"request_sha256":request_digest(request)}),
    }
}

/// A connection received from the trusted launcher's in-memory control plane.
/// There is deliberately no JSON constructor or public-socket equivalent.
pub struct AuthenticatedConnection {
    pub(crate) stream: Transport,
    pub(crate) identity: WorkloadIdentity,
}

pub(crate) struct WorkloadIdentity {
    principal: String,
    actor: Actor,
    request_sha256: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    authentication_action_sha256: String,
    committed_decision: sea_forge_ledger::types::CommittedRecordRef,
    used: bool,
}

impl AuthenticatedConnection {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        stream: Transport,
        grant: ActionGrant,
        principal: &str,
        actor: Actor,
        request: &Value,
        run_id: &str,
        item_id: &str,
        root: &Path,
    ) -> Result<Self, sea_forge_core::errors::ForgeError> {
        use sea_forge_core::errors::ForgeError;
        if principal.trim().is_empty() || actor.actor_id.trim().is_empty() {
            return Err(ForgeError::Input(
                "workload principal and actor are required".into(),
            ));
        }
        let expires_at = grant.expires_at();
        let authentication_action_sha256 = grant.action_hash().to_owned();
        let committed_decision = grant.committed_decision().clone();
        if chrono::Utc::now() >= expires_at {
            return Err(ForgeError::Input(
                "workload authentication grant expired".into(),
            ));
        }
        let action = authentication_action(principal, &actor, request);
        grant.authorize(&action, run_id, item_id, root)?;
        Ok(Self {
            stream,
            identity: WorkloadIdentity {
                principal: principal.into(),
                actor,
                request_sha256: request_digest(request),
                expires_at,
                authentication_action_sha256,
                committed_decision,
                used: false,
            },
        })
    }
}

impl WorkloadIdentity {
    pub(crate) fn proof(&self) -> AuthenticatedRequest {
        AuthenticatedRequest {
            principal: self.principal.clone(),
            actor: self.actor.clone(),
            request_sha256: self.request_sha256.clone(),
            authentication_action_sha256: self.authentication_action_sha256.clone(),
            committed_decision: self.committed_decision.clone(),
            expires_at: self.expires_at,
        }
    }
    pub(crate) fn resolve(
        &mut self,
        raw: &str,
        bindings: &IdentityBindings,
        peer: Option<PeerIdentity>,
    ) -> Result<ResolvedActor, IdentityRefusal> {
        let claim = ActorClaim::parse(raw).ok_or(IdentityRefusal::Missing)?;
        // Preserve ordinary unbound/role refusal before checking the channel.
        let actor = bindings.resolve_authenticated_claim(&claim, peer)?;
        if claim.actor_id != self.actor.actor_id || claim.role != self.actor.role {
            return Err(IdentityRefusal::AuthenticationRefused {
                reason: "private connection belongs to a different actor".into(),
            });
        }
        if ActorClaim::parse_on_behalf_of(raw).is_some() {
            return Err(IdentityRefusal::AuthenticationRefused {
                reason: "workload delegation requires a separate bounded grant".into(),
            });
        }
        if self.used || chrono::Utc::now() >= self.expires_at {
            return Err(IdentityRefusal::AuthenticationRefused {
                reason: "private connection grant consumed or expired".into(),
            });
        }
        let request = crate::mcp_protocol::decode_json(raw.as_bytes()).map_err(|_| {
            IdentityRefusal::AuthenticationRefused {
                reason: "ambiguous request".into(),
            }
        })?;
        if request_digest(&request) != self.request_sha256 {
            return Err(IdentityRefusal::AuthenticationRefused {
                reason: "request is outside the authenticated grant".into(),
            });
        }
        self.used = true;
        tracing::debug!(
            principal = self.principal,
            actor_id = self.actor.actor_id,
            request_sha256 = self.request_sha256,
            "private workload request authenticated"
        );
        Ok(actor)
    }

    pub(crate) fn expires_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.expires_at
    }
}

#[derive(serde::Serialize)]
pub(crate) struct AuthenticatedRequest {
    principal: String,
    actor: Actor,
    request_sha256: String,
    authentication_action_sha256: String,
    committed_decision: sea_forge_ledger::types::CommittedRecordRef,
    expires_at: chrono::DateTime<chrono::Utc>,
}

pub(crate) fn record_authenticated_request(
    root: &Path,
    request_id: Option<&str>,
    proof: &AuthenticatedRequest,
) -> Result<(), sea_forge_core::errors::ForgeError> {
    // One short append per admission; serialize independent stream handles so
    // concurrent connections cannot race their hash-chain predecessor.
    static APPEND: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = APPEND.lock().map_err(|_| {
        sea_forge_core::errors::ForgeError::Internal(
            "workload authentication ledger poisoned".into(),
        )
    })?;
    let ledger =
        sea_forge_ledger::LedgerStream::open(root, "workload-authentication", "sea-forge")?;
    ledger.commit_typed(
        "workload_request_authenticated",
        request_id.map(str::to_owned).into_iter().collect(),
        proof,
        vec![proof.committed_decision.entry_ulid().to_owned()],
    )?;
    Ok(())
}
