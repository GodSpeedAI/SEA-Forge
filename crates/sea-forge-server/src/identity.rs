//! Governed actor identity for SFWP requests (SF-005, decision U-07).
//!
//! Before this module the server attributed every protected action to whatever
//! `entity` string the request carried, with `ActorRole::Operator` hardcoded,
//! and the desktop client supplied a fabricated constant
//! (`router.tsx` `mockGuardContext`). Separation of duty could not be
//! expressed, let alone enforced.
//!
//! U-07 settled the contract in four parts, and each one shows up here:
//!
//! 1. **Identity binds per request.** The actor block travels on the request,
//!    not on a session, so it lands in the ledger entry the request produces
//!    and survives a reconnect without a handshake. [`ActorClaim`].
//! 2. **The server derives the caller's OS identity from the socket.** The uid
//!    comes from `SO_PEERCRED`, which the kernel fills in — a client cannot
//!    assert it. The claim must map to that uid or the request is refused.
//!    [`PeerIdentity`], [`IdentityBindings::resolve`].
//! 3. **The approver is compared against the ledger**, not against the
//!    connection. That comparison lives at the approval call site; this module
//!    supplies the resolved actor it compares.
//! 4. **Optional on inspect, required on protected.** [`is_protected`] is one
//!    exhaustive match over the request enum, so a new verb cannot be added
//!    without classifying it — the compiler refuses.
//!
//! Trusting the client's assertion because the socket is `0600` was rejected
//! deliberately: every process running as the owner can open that socket, so
//! believing the assertion would move the fabricated identity one layer down
//! rather than remove it.

use crate::Request;
use schemars::JsonSchema;
use sea_forge_core::types::ActorRole;
use serde::{Deserialize, Serialize};

/// The wire spelling of a role (`operator`, `R-SO`, …).
///
/// The view below reports roles as these strings rather than as `ActorRole`.
/// `ActorRole` lives in `sea-forge-core`, a kernel crate, and deriving
/// `JsonSchema` on it would pull `schemars` across the kernel boundary to
/// describe a value whose wire form is already just this string.
fn role_wire_name(role: &ActorRole) -> String {
    serde_json::to_value(role)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{role:?}"))
}

/// The OS identity of the process on the other end of the connection, as
/// reported by the kernel.
///
/// `None` means the credential could not be read. That is treated as *no
/// identity*, never as a wildcard: a connection whose peer cannot be
/// identified may still use inspect verbs and is refused every protected one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeerIdentity {
    pub uid: u32,
}

impl PeerIdentity {
    /// Read `SO_PEERCRED` off an accepted Unix stream.
    pub fn of(stream: &tokio::net::UnixStream) -> Option<Self> {
        stream.peer_cred().ok().map(|cred| Self { uid: cred.uid() })
    }
}

/// The effective uid of *this* process.
///
/// Used to configure a cell for the user who will run it, and by tests that
/// need a binding matching the uid their own connection will present.
///
/// ponytail: derived from the ownership of a file this process creates,
/// because `getuid` would mean taking a direct `libc` dependency and
/// dependency changes are ask-first here. Swap for `libc::geteuid()` if that
/// is ever authorized — it is the same answer, without the syscall round trip.
pub fn current_uid() -> Option<u32> {
    use std::os::unix::fs::MetadataExt;
    let probe = std::env::temp_dir().join(format!("sea-forge-uid-probe-{}", std::process::id()));
    let uid = std::fs::File::create(&probe)
        .ok()
        .and_then(|file| file.metadata().ok())
        .map(|meta| meta.uid());
    let _ = std::fs::remove_file(&probe);
    uid
}

/// The actor block a client attaches to a protected request.
///
/// Deliberately *not* a field on every `Request` variant. Parsing it off the
/// raw line keeps one shape for ~15 protected verbs, and means a verb added
/// later cannot forget to carry it — `is_protected` is what decides, and that
/// match is exhaustive.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ActorClaim {
    pub actor_id: String,
    pub role: ActorRole,
}

impl ActorClaim {
    /// Pull the `actor` object out of a raw NDJSON request line, if present.
    ///
    /// A malformed `actor` is `None` rather than an error here, so it reaches
    /// the protected-verb check and is refused there with the same typed
    /// denial as an absent one. An inspect verb with a garbled actor block is
    /// simply an inspect verb.
    pub fn parse(raw_line: &str) -> Option<Self> {
        serde_json::from_str::<serde_json::Value>(raw_line)
            .ok()?
            .get("actor")
            .and_then(|actor| serde_json::from_value(actor.clone()).ok())
    }
}

/// An actor claim that has been checked against the peer's uid.
///
/// Only constructible through [`IdentityBindings::resolve`], so a value of this
/// type is evidence the check ran — a resolved actor cannot be fabricated by
/// assembling the struct at a call site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedActor {
    actor_id: String,
    role: ActorRole,
    uid: u32,
}

impl ResolvedActor {
    pub fn actor_id(&self) -> &str {
        &self.actor_id
    }
    pub fn role(&self) -> &ActorRole {
        &self.role
    }
    pub fn uid(&self) -> u32 {
        self.uid
    }
}

/// Why a protected request was refused.
///
/// Each carries the operator-facing `error_class` the SFWP response uses, so
/// the vocabulary is defined once rather than spelled at each call site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityRefusal {
    /// The verb is protected and the request carried no usable actor block.
    Missing,
    /// The peer credential could not be read, so nothing can be verified.
    UnknownPeer,
    /// The cell configures no identity bindings at all.
    Unconfigured,
    /// The claimed actor is not bound to this uid.
    NotBound { actor_id: String, uid: u32 },
    /// The actor is bound to this uid but not in the role it claimed.
    RoleNotHeld { actor_id: String, role: ActorRole },
    /// The request attributes its work to a different actor than it verified.
    ///
    /// `entity` is what reaches the authority engine and lands in the ledger as
    /// the acting principal; `actor` is what the uid check verified. If those
    /// disagree the cell would verify one identity and record another — which
    /// would defeat separation of duty by letting a submitter file work under a
    /// name they are not, then approve it under the name they are.
    EntityMismatch { actor_id: String, entity: String },
    /// The actor resolving this approval is the one whose work it gates.
    ///
    /// U-07 answer 3: the approver is identified independently and compared
    /// against the submitter *recorded in the ledger* — not against the
    /// connection, which would let a reconnect launder a self-approval.
    SelfApproval {
        actor_id: String,
        approval_id: String,
    },
}

impl IdentityRefusal {
    pub fn error_class(&self) -> &'static str {
        match self {
            Self::Missing => "identity_required",
            Self::UnknownPeer => "identity_unverifiable",
            Self::Unconfigured => "identity_unconfigured",
            Self::NotBound { .. } => "identity_not_bound",
            Self::RoleNotHeld { .. } => "identity_role_not_held",
            Self::EntityMismatch { .. } => "identity_entity_mismatch",
            Self::SelfApproval { .. } => "separation_of_duty",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Missing => "this verb causes a side effect and requires an `actor` block \
                 {\"actor_id\":…,\"role\":…}; inspect verbs do not"
                .into(),
            Self::UnknownPeer => "the peer credential for this connection could not be read, \
                 so no actor claim on it can be verified"
                .into(),
            Self::Unconfigured => "this cell configures no identity bindings; add \
                 `identity.bindings` to server.yaml mapping a uid to an actor \
                 before running protected verbs"
                .into(),
            Self::NotBound { actor_id, uid } => {
                format!("actor `{actor_id}` is not bound to uid {uid} in this cell")
            }
            Self::RoleNotHeld { actor_id, role } => {
                format!("actor `{actor_id}` does not hold the role {role:?} it claimed")
            }
            Self::EntityMismatch { actor_id, entity } => format!(
                "this request verified actor `{actor_id}` but attributes its work to \
                 `{entity}`; the two must name the same actor"
            ),
            Self::SelfApproval {
                actor_id,
                approval_id,
            } => format!(
                "actor `{actor_id}` requested the work approval `{approval_id}` gates and \
                 cannot resolve it; approval requires a different actor"
            ),
        }
    }

    /// The SFWP error body. `no_side_effect` is stated rather than implied:
    /// SF-005 requires a refusal to leave nothing behind, and an operator
    /// reading the response should not have to infer that from the absence of
    /// other fields.
    pub fn response(&self) -> serde_json::Value {
        serde_json::json!({
            "error": self.message(),
            "error_class": self.error_class(),
            "no_side_effect": true,
        })
    }
}

/// One uid, and the actor and roles it may claim.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IdentityBinding {
    pub uid: u32,
    pub actor_id: String,
    /// Every role this actor may claim. A binding with an empty list can hold
    /// no role and therefore authorizes nothing — which is a usable way to
    /// suspend an actor without deleting the record of who they are.
    #[serde(default)]
    pub roles: Vec<ActorRole>,
}

/// The cell's uid → actor mapping, from `server.yaml`.
///
/// Absent means *unconfigured*, and unconfigured refuses every protected verb.
/// There is deliberately no "derive an actor from the uid when nothing is
/// configured" fallback: that would be a fail-open that let any local process
/// act as a governed operator, which is the defect SF-005 exists to remove.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IdentityBindings {
    #[serde(default)]
    pub bindings: Vec<IdentityBinding>,
}

impl IdentityBindings {
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Bind `actor_id`, as an operator, to the user running this process.
    ///
    /// This is the shape a single-operator local cell wants, and the one a
    /// test needs so its own connection resolves. It is a convenience for
    /// building the configuration, not a bypass: the resulting bindings go
    /// through [`Self::resolve`] like any other, and a connection from a
    /// different uid still fails to match.
    ///
    /// Yields no bindings — and therefore authorizes nothing — if the uid
    /// cannot be determined.
    pub fn local_operator(actor_id: &str) -> Self {
        Self {
            bindings: current_uid()
                .map(|uid| IdentityBinding {
                    uid,
                    actor_id: actor_id.into(),
                    roles: vec![ActorRole::Operator],
                })
                .into_iter()
                .collect(),
        }
    }

    /// What this connection may claim (`identity.get`).
    ///
    /// Scoped to `peer`'s own uid: a caller learns which actors *they* may act
    /// as and nothing about anyone else's bindings. The refusal reasons are the
    /// same ones [`Self::resolve`] would produce, so a client can show why
    /// protected work will fail before attempting it rather than discovering it
    /// from a denied side effect.
    pub fn describe(&self, peer: Option<PeerIdentity>) -> IdentityView {
        let refusal = |refusal: IdentityRefusal| {
            Some(RefusalView {
                error_class: refusal.error_class().into(),
                message: refusal.message(),
            })
        };

        let Some(peer) = peer else {
            return IdentityView {
                uid: None,
                configured: !self.is_empty(),
                available: Vec::new(),
                refusal: refusal(IdentityRefusal::UnknownPeer),
            };
        };

        if self.is_empty() {
            return IdentityView {
                uid: Some(peer.uid),
                configured: false,
                available: Vec::new(),
                refusal: refusal(IdentityRefusal::Unconfigured),
            };
        }

        let available: Vec<AvailableActor> = self
            .bindings
            .iter()
            .filter(|binding| binding.uid == peer.uid)
            .map(|binding| AvailableActor {
                actor_id: binding.actor_id.clone(),
                roles: binding.roles.iter().map(role_wire_name).collect(),
            })
            .collect();

        // A uid the cell has bindings for, but none of them this one. Reported
        // as `not_bound` against the uid itself rather than against an actor
        // the caller never named — there is no claim here to echo back.
        let refusal = available.is_empty().then(|| RefusalView {
            error_class: "identity_not_bound".into(),
            message: format!("no actor is bound to uid {} in this cell", peer.uid),
        });

        IdentityView {
            uid: Some(peer.uid),
            configured: true,
            available,
            refusal,
        }
    }

    /// Check a claim against the peer's uid and this cell's bindings.
    pub fn resolve(
        &self,
        claim: Option<&ActorClaim>,
        peer: Option<PeerIdentity>,
    ) -> Result<ResolvedActor, IdentityRefusal> {
        let claim = claim.ok_or(IdentityRefusal::Missing)?;
        let peer = peer.ok_or(IdentityRefusal::UnknownPeer)?;
        if self.is_empty() {
            return Err(IdentityRefusal::Unconfigured);
        }
        let bound = self
            .bindings
            .iter()
            .find(|binding| binding.uid == peer.uid && binding.actor_id == claim.actor_id)
            .ok_or_else(|| IdentityRefusal::NotBound {
                actor_id: claim.actor_id.clone(),
                uid: peer.uid,
            })?;
        if !bound.roles.contains(&claim.role) {
            return Err(IdentityRefusal::RoleNotHeld {
                actor_id: claim.actor_id.clone(),
                role: claim.role.clone(),
            });
        }
        Ok(ResolvedActor {
            actor_id: bound.actor_id.clone(),
            role: claim.role.clone(),
            uid: peer.uid,
        })
    }
}

/// The actor who requested the work an approval gates, per the ledger.
///
/// Walks `approval_request` → its `decision_id` → the `authority_decision` that
/// escalated, and reports that decision's bound principal. The ledger is the
/// only acceptable source here: the submitter may have disconnected, restarted,
/// or reconnected since, so anything derived from a live connection would be a
/// different question with a coincidentally similar answer.
///
/// `Ok(None)` means the chain could not be completed — an unknown approval, a
/// case with no ledger, or a decision that is not recorded. The caller must
/// treat that as *not proven distinct* rather than as permission, and it does.
pub fn approval_submitter(
    root: &std::path::Path,
    case_id: &str,
    approval_id: &str,
) -> Result<Option<String>, sea_forge_core::errors::ForgeError> {
    let ledger_dir = root.join("ledgers").join(format!("case-{case_id}"));
    if !ledger_dir.exists() {
        return Ok(None);
    }
    let entries =
        sea_forge_ledger::LedgerStream::open(root, format!("case-{case_id}"), "sea-forge-server")?
            .read_entries()?;

    let field = |value: &serde_json::Value, key: &str| {
        value.get(key).and_then(|v| v.as_str()).map(str::to_owned)
    };

    let Some(decision_id) = entries
        .iter()
        .filter(|entry| entry.record_kind == "approval_request")
        .find(|entry| field(&entry.payload, "approval_id").as_deref() == Some(approval_id))
        .and_then(|entry| field(&entry.payload, "decision_id"))
    else {
        return Ok(None);
    };

    Ok(entries
        .iter()
        .filter(|entry| entry.record_kind == "authority_decision")
        .find(|entry| field(&entry.payload, "decision_id").as_deref() == Some(&decision_id))
        .and_then(|entry| {
            entry
                .payload
                .get("identity_binding")
                .and_then(|binding| field(binding, "principal"))
        }))
}

/// One actor this connection is entitled to claim, and the roles it holds.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct AvailableActor {
    pub actor_id: String,
    /// Wire spellings, in `server.yaml` order. Empty means a suspended actor:
    /// the cell still records who they are, but they can claim nothing.
    pub roles: Vec<String>,
}

/// Why nothing can be claimed on this connection, in the same vocabulary a
/// refused protected verb uses — so a client never has to map one set of
/// reasons onto another.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct RefusalView {
    pub error_class: String,
    pub message: String,
}

/// What this cell believes about the identity on the other end of this
/// connection (`identity.get`).
///
/// An inspect verb: it reports only what the kernel already knows about a
/// connection the caller already holds, and reveals nothing about other uids'
/// bindings. It exists so the client can *display* a resolved identity and
/// choose which bound actor to act as, instead of fabricating one — which is
/// precisely what `router.tsx`'s `mockGuardContext` did before SF-005.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct IdentityView {
    /// The uid the kernel reported for this connection, when readable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uid: Option<u32>,
    /// Whether this cell configures any bindings at all. False means every
    /// protected verb will be refused regardless of what is claimed.
    pub configured: bool,
    /// Every actor this connection may claim. Empty when nothing resolves.
    pub available: Vec<AvailableActor>,
    /// Present exactly when `available` is empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<RefusalView>,
}

/// Whether this verb causes a side effect and therefore requires an actor.
///
/// The match is exhaustive with no wildcard arm on purpose. Adding a verb to
/// [`Request`] without deciding whether it is governed will not compile, which
/// is the only way this classification stays true as the protocol grows — a
/// `_ => false` arm would silently admit every future verb as unprotected.
pub fn is_protected(request: &Request) -> bool {
    match request {
        // Side effects: these run work, commit records, or resolve approvals.
        Request::Submit { .. }
        | Request::Approve { .. }
        | Request::Reject { .. }
        | Request::Delegate { .. }
        | Request::CancelDelegation { .. }
        | Request::Ask { .. }
        | Request::AgentProbe { .. }
        | Request::CaseCommit { .. }
        | Request::ApprovalDecide { .. } => true,

        // Read-only projections and protocol chatter. An old client that never
        // learned about the actor block keeps working against all of these.
        Request::Status { .. }
        | Request::AgentList
        | Request::SystemHello { .. }
        | Request::SystemDescribe
        | Request::SystemGetSchema { .. }
        | Request::RequestGetStatus { .. }
        | Request::EventsSubscribe { .. }
        | Request::EventsUnsubscribe
        | Request::EventsGetRange { .. }
        | Request::ReadinessGet { .. }
        // Reports only what the kernel already knows about the caller's own
        // connection. Requiring an actor here would make it impossible to
        // discover which actor to claim.
        | Request::IdentityGet
        | Request::CaseEntryOptions
        | Request::CasePreflight { .. }
        | Request::CaseList
        | Request::CaseGetOverview { .. }
        | Request::CaseGetHorizon { .. }
        | Request::ApprovalList { .. }
        | Request::RunList { .. }
        | Request::RunGet { .. }
        | Request::AssetList
        | Request::DelegationPreview { .. }
        | Request::DelegationList => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bindings() -> IdentityBindings {
        IdentityBindings {
            bindings: vec![
                IdentityBinding {
                    uid: 1000,
                    actor_id: "operator_local".into(),
                    roles: vec![ActorRole::Operator],
                },
                IdentityBinding {
                    uid: 1001,
                    actor_id: "reviewer_local".into(),
                    roles: vec![ActorRole::Operator],
                },
            ],
        }
    }

    fn claim(actor_id: &str, role: ActorRole) -> ActorClaim {
        ActorClaim {
            actor_id: actor_id.into(),
            role,
        }
    }

    #[test]
    fn a_bound_actor_on_its_own_uid_resolves() {
        let resolved = bindings()
            .resolve(
                Some(&claim("operator_local", ActorRole::Operator)),
                Some(PeerIdentity { uid: 1000 }),
            )
            .expect("bound actor must resolve");
        assert_eq!(resolved.actor_id(), "operator_local");
        assert_eq!(resolved.uid(), 1000);
    }

    /// The whole point of reading the credential from the kernel: a caller
    /// cannot become someone else by saying so.
    #[test]
    fn a_claim_for_another_uids_actor_is_refused() {
        let refusal = bindings()
            .resolve(
                Some(&claim("reviewer_local", ActorRole::Operator)),
                Some(PeerIdentity { uid: 1000 }),
            )
            .expect_err("uid 1000 must not be able to act as reviewer_local");
        assert_eq!(refusal.error_class(), "identity_not_bound");
    }

    #[test]
    fn a_role_the_actor_does_not_hold_is_refused() {
        let refusal = bindings()
            .resolve(
                Some(&claim("operator_local", ActorRole::SecurityOfficer)),
                Some(PeerIdentity { uid: 1000 }),
            )
            .expect_err("operator_local holds only operator");
        assert_eq!(refusal.error_class(), "identity_role_not_held");
    }

    #[test]
    fn a_missing_claim_is_refused_before_anything_else() {
        let refusal = bindings()
            .resolve(None, Some(PeerIdentity { uid: 1000 }))
            .expect_err("no claim, no protected work");
        assert_eq!(refusal.error_class(), "identity_required");
    }

    /// An unreadable peer credential must not behave like a wildcard.
    #[test]
    fn an_unverifiable_peer_is_refused_even_with_a_valid_claim() {
        let refusal = bindings()
            .resolve(Some(&claim("operator_local", ActorRole::Operator)), None)
            .expect_err("nothing to check the claim against");
        assert_eq!(refusal.error_class(), "identity_unverifiable");
    }

    /// The fail-closed default. A cell that configures nothing authorizes
    /// nothing, rather than deriving an actor from whoever happens to connect.
    #[test]
    fn an_unconfigured_cell_refuses_every_protected_verb() {
        let refusal = IdentityBindings::default()
            .resolve(
                Some(&claim("operator_local", ActorRole::Operator)),
                Some(PeerIdentity { uid: 1000 }),
            )
            .expect_err("an unconfigured cell must not authorize");
        assert_eq!(refusal.error_class(), "identity_unconfigured");
    }

    #[test]
    fn an_actor_block_is_read_off_the_raw_line() {
        let parsed = ActorClaim::parse(
            r#"{"verb":"submit","actor":{"actor_id":"operator_local","role":"operator"}}"#,
        );
        assert_eq!(
            parsed,
            Some(claim("operator_local", ActorRole::Operator)),
            "the actor block must parse off the wire exactly as documented"
        );
    }

    /// A garbled actor block must not read as a valid one, and must not crash
    /// the parse of the request around it.
    #[test]
    fn a_malformed_actor_block_reads_as_absent() {
        assert_eq!(
            ActorClaim::parse(r#"{"verb":"submit","actor":"nope"}"#),
            None
        );
        assert_eq!(
            ActorClaim::parse(r#"{"verb":"submit","actor":{"actor_id":"x"}}"#),
            None,
            "a claim without a role is not a claim"
        );
        assert_eq!(ActorClaim::parse("not json at all"), None);
    }

    #[test]
    fn a_binding_with_no_roles_authorizes_nothing() {
        let suspended = IdentityBindings {
            bindings: vec![IdentityBinding {
                uid: 1000,
                actor_id: "operator_local".into(),
                roles: vec![],
            }],
        };
        let refusal = suspended
            .resolve(
                Some(&claim("operator_local", ActorRole::Operator)),
                Some(PeerIdentity { uid: 1000 }),
            )
            .expect_err("a suspended actor holds no role");
        assert_eq!(refusal.error_class(), "identity_role_not_held");
    }
}
