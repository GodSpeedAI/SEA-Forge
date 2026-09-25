//! Durable audit record for delegated (`on_behalf_of`) requests (plan T02,
//! decision D-2).
//!
//! The kernel records the *effective actor* — the end user — on every trace and
//! ledger entry a delegated request produces, exactly as it would for a direct
//! request: `TraceEvent.actor_id` is the end user, the approver comparison and
//! separation of duty see the end user, and `writer_identity_ref` follows the
//! acting principal. That is deliberate and must not change; the kernel's job
//! is to record who acted, and the end user acted.
//!
//! What the kernel cannot know is *which gateway spoke for that end user*. The
//! connection's uid belongs to the gateway, and a record naming only the end
//! user cannot tell "the operator did this" from "the gateway did this as the
//! operator" — precisely the two stories D-2 exists to keep apart. So the
//! server records the pair itself, once per admitted delegated request, in its
//! own ledger under the cell root.
//!
//! Fail-closed: if this record cannot be written, the delegated request is
//! refused *before* it runs. An unrecorded delegation is exactly the
//! unattributable side effect D-2 requires the server to refuse — and the
//! refusal leaves nothing behind either, because nothing has run yet.

use crate::identity::ResolvedActor;
use sea_forge_ledger::types::LedgerStream;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The ledger id, under `<root>/ledgers/`.
pub const DELEGATION_AUDIT_LEDGER: &str = "delegation-audit";

/// The record kind written for one admitted delegated request.
pub const DELEGATED_REQUEST_RECORD_KIND: &str = "delegated_request";

/// The ledger writer for these records. Not an actor id: the ledger's writer
/// field names the process that wrote the entry, and the two principals are
/// named in the payload where they belong.
const WRITER: &str = "sea-forge-server";

/// One admitted delegated request, both principals named.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegatedRequestRecord {
    /// The wire verb (`case_add_item`, `approval_decide`, …).
    pub verb: String,
    /// The client's correlation id, when it sent one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The principal that spoke: the configured gateway actor.
    pub gateway_actor_id: String,
    /// The gateway's uid, as `SO_PEERCRED` reported it on this connection.
    pub gateway_uid: u32,
    /// The end user every other record from this request names.
    pub effective_actor_id: String,
    /// The wire spelling of the role claimed for that end user.
    pub effective_role: String,
}

/// The `verb` a raw NDJSON request line carries, or `unknown` when it has
/// none. Read off the line rather than matched per variant, for the reason
/// `ActorClaim` is: a per-variant match would silently skip any verb added
/// later.
pub fn verb_of(raw_line: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw_line)
        .ok()
        .and_then(|value| {
            value
                .get("verb")
                .and_then(|verb| verb.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unknown".into())
}

/// Record one admitted delegated request. `Ok(())` means the record is durable.
///
/// Takes a *delegated* actor: the caller checks
/// [`ResolvedActor::is_delegated`] first, and a non-delegated actor has no
/// gateway to name.
///
/// The write happens on a blocking thread: the ledger appends and fsyncs, and
/// the caller is an async worker.
pub async fn record_delegated_request(
    root: &Path,
    record: DelegatedRequestRecord,
) -> Result<(), String> {
    let root = root.to_path_buf();
    tokio::task::spawn_blocking(move || write(&root, &record))
        .await
        .map_err(|error| format!("delegation audit task failed: {error}"))?
}

impl DelegatedRequestRecord {
    /// The record for `actor` acting through the gateway on `raw_line`.
    ///
    /// `None` when the actor is not delegated — there is no gateway to name.
    pub fn of(raw_line: &str, request_id: Option<&str>, actor: &ResolvedActor) -> Option<Self> {
        let provenance = actor.delegation()?;
        Some(Self {
            verb: verb_of(raw_line),
            request_id: request_id.map(str::to_owned),
            gateway_actor_id: provenance.gateway_actor_id().into(),
            gateway_uid: provenance.gateway_uid(),
            effective_actor_id: actor.actor_id().into(),
            effective_role: crate::identity::role_wire_name(actor.role()),
        })
    }
}

fn write(root: &Path, record: &DelegatedRequestRecord) -> Result<(), String> {
    let ledger =
        LedgerStream::open(root, DELEGATION_AUDIT_LEDGER, WRITER).map_err(|e| e.to_string())?;
    let payload = serde_json::to_value(record).map_err(|e| e.to_string())?;
    // Subject refs name both principals, so a reader can find every delegated
    // request either of them was involved in without scanning the file.
    let subject_refs = vec![
        format!("actor:{}", record.effective_actor_id),
        format!("gateway:{}", record.gateway_actor_id),
    ];
    ledger
        .append(DELEGATED_REQUEST_RECORD_KIND, subject_refs, payload, vec![])
        .map_err(|e| e.to_string())?;
    Ok(())
}
