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

use crate::config::GatewayConfig;
use crate::Request;
use schemars::JsonSchema;
use sea_forge_core::types::ActorRole;
use serde::{Deserialize, Serialize};

/// The wire spelling of a role (`operator`, `R-SO`, …).
///
/// The views below report roles as these strings rather than as `ActorRole`.
/// `ActorRole` lives in `sea-forge-core`, a kernel crate, and deriving
/// `JsonSchema` on it would pull `schemars` across the kernel boundary to
/// describe a value whose wire form is already just this string. Public so the
/// delegation audit record spells roles the same way the wire does.
pub fn role_wire_name(role: &ActorRole) -> String {
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
        Self::block(raw_line, "actor")
    }

    /// Pull the `on_behalf_of` object out of a raw NDJSON request line, if
    /// present (T02, D-2).
    ///
    /// Same shape and same failure mode as [`Self::parse`], deliberately: a
    /// garbled `on_behalf_of` reads as *absent*, so the request falls back to
    /// the direct path and is attributed to the actor block it does carry —
    /// never to the end user a mangled block half-names. The gateway's own
    /// standing polices that path, so a mangled block cannot become a
    /// privilege: it can only lose the delegation.
    ///
    /// A *well-formed* `on_behalf_of` on a connection that is not the
    /// configured gateway is refused by [`IdentityBindings::resolve_delegated`]
    /// — presence alone authorizes nothing.
    pub fn parse_on_behalf_of(raw_line: &str) -> Option<Self> {
        Self::block(raw_line, "on_behalf_of")
    }

    fn block(raw_line: &str, key: &str) -> Option<Self> {
        serde_json::from_str::<serde_json::Value>(raw_line)
            .ok()?
            .get(key)
            .and_then(|block| serde_json::from_value(block.clone()).ok())
    }
}

/// An actor claim that has been checked against the peer's uid.
///
/// Only constructible through [`IdentityBindings::resolve`], so a value of this
/// type is evidence the check ran — a resolved actor cannot be fabricated by
/// assembling the struct at a call site.
///
/// A delegated resolution additionally carries the gateway principal that
/// spoke for the end user (T02, D-2): the effective actor is always the end
/// user, and the gateway attribution rides alongside for the audit trail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedActor {
    actor_id: String,
    role: ActorRole,
    uid: u32,
    delegation: Option<DelegationProvenance>,
}

/// Which gateway spoke for this end-user actor, on a delegated request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DelegationProvenance {
    gateway_actor_id: String,
    gateway_uid: u32,
}

impl ResolvedActor {
    pub fn actor_id(&self) -> &str {
        &self.actor_id
    }
    pub fn role(&self) -> &ActorRole {
        &self.role
    }
    /// The uid the kernel reported for this connection's peer.
    ///
    /// On a delegated resolution this is the *gateway's* uid — the connection
    /// is the gateway's, not the end user's. [`Self::actor_id`] and
    /// [`Self::role`] are the end user's; [`Self::delegation`] names the
    /// gateway principal that spoke for them.
    pub fn uid(&self) -> u32 {
        self.uid
    }
    /// The gateway principal that presented `on_behalf_of`, if this request
    /// was delegated. `None` on direct (non-delegated) resolutions.
    pub fn delegation(&self) -> Option<&DelegationProvenance> {
        self.delegation.as_ref()
    }
    /// Whether this request was delegated by the gateway principal.
    pub fn is_delegated(&self) -> bool {
        self.delegation.is_some()
    }
}

impl DelegationProvenance {
    pub fn gateway_actor_id(&self) -> &str {
        &self.gateway_actor_id
    }
    pub fn gateway_uid(&self) -> u32 {
        self.gateway_uid
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
    /// An `on_behalf_of` block the gateway rules refuse (T02, D-2).
    ///
    /// Deliberately distinct from `NotBound`: `identity_not_bound` means "not
    /// your actor", while this means "not the gateway, not allowlisted, role
    /// widened, or delegation unconfigured". Operators triaging a refusal
    /// must be able to tell the two apart.
    DelegationRefused { reason: String },
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
            Self::DelegationRefused { .. } => "identity_delegation_refused",
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
            Self::DelegationRefused { reason } => format!("delegated identity refused: {reason}"),
        }
    }

    /// The only lawful repair for this refusal, kept beside the error-class
    /// vocabulary so inspect and command surfaces cannot drift into separate
    /// remediation stories.
    pub fn next_lawful_action(&self) -> &'static str {
        match self {
            Self::Missing => "Select a server-advertised actor and retry the protected operation",
            Self::UnknownPeer => "Reconnect through a transport that exposes peer credentials",
            Self::Unconfigured => "Configure an identity binding for this operating-system user",
            Self::NotBound { .. } => "Use an actor bound to this operating-system user",
            Self::RoleNotHeld { .. } => "Select a role held by the bound actor",
            Self::EntityMismatch { .. } => "Attribute the operation to the verified actor",
            Self::SelfApproval { .. } => "Ask an independently bound approver to decide",
            Self::DelegationRefused { .. } => {
                "Send on_behalf_of only from the gateway uid for an allowlisted actor and a held role"
            }
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
            "next_lawful_action": self.next_lawful_action(),
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
                no_side_effect: Some(true),
                next_lawful_action: Some(refusal.next_lawful_action().into()),
            })
        };

        let Some(peer) = peer else {
            return IdentityView {
                uid: None,
                configured: !self.is_empty(),
                available: Vec::new(),
                effective_actor: None,
                refusal: refusal(IdentityRefusal::UnknownPeer),
            };
        };

        if self.is_empty() {
            return IdentityView {
                uid: Some(peer.uid),
                configured: false,
                available: Vec::new(),
                effective_actor: None,
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
            no_side_effect: Some(true),
            next_lawful_action: Some("Use an actor bound to this operating-system user".into()),
        });

        IdentityView {
            uid: Some(peer.uid),
            configured: true,
            available,
            effective_actor: None,
            refusal,
        }
    }

    /// [`Self::describe`], plus the effective actor a *delegated* inspect
    /// presents (T02, D-2): `identity.get` must be able to report the actor a
    /// delegated request will be attributed to, not only what the connection
    /// may claim directly.
    ///
    /// A delegation that does not resolve replaces the refusal with its own
    /// reason — the same vocabulary a refused protected verb uses — so a
    /// client can show why delegation will fail before attempting it. The
    /// connection's own `available` set is reported either way: the gateway is
    /// still entitled to act as itself directly.
    pub fn describe_delegated(
        &self,
        claim: Option<&ActorClaim>,
        on_behalf_of: Option<&ActorClaim>,
        gateway: Option<&GatewayConfig>,
        peer: Option<PeerIdentity>,
    ) -> IdentityView {
        let mut view = self.describe(peer);
        if on_behalf_of.is_none() {
            return view;
        }
        match self.resolve_delegated(claim, on_behalf_of, gateway, peer) {
            Ok(actor) => view.effective_actor = EffectiveActorView::of(&actor),
            Err(refusal) => {
                view.refusal = Some(RefusalView {
                    error_class: refusal.error_class().into(),
                    message: refusal.message(),
                    no_side_effect: Some(true),
                    next_lawful_action: Some(refusal.next_lawful_action().into()),
                })
            }
        }
        view
    }

    /// Check a claim against the peer's uid and this cell's bindings.
    ///
    /// The direct path: the actor a connection claims must be bound to that
    /// connection's own uid. Delegation is the other path,
    /// [`Self::resolve_delegated`].
    pub fn resolve(
        &self,
        claim: Option<&ActorClaim>,
        peer: Option<PeerIdentity>,
    ) -> Result<ResolvedActor, IdentityRefusal> {
        self.resolve_delegated(claim, None, None, peer)
    }

    /// [`Self::resolve`], plus the gateway delegation path (T02, D-2).
    ///
    /// `on_behalf_of` is honoured only when *every* one of these holds:
    ///
    /// 1. this cell configures a gateway principal at all (`gateway`);
    /// 2. the connection's uid is that principal's uid, read from
    ///    `SO_PEERCRED` — a client cannot assert it;
    /// 3. the request's own `actor` block names the configured gateway actor
    ///    and holds the role it claims, so the gateway is governed like any
    ///    other principal rather than exempted from the gate;
    /// 4. the target actor is in the gateway-delegable allowlist and is not
    ///    the gateway itself;
    /// 5. the target actor's own binding declares the role claimed for it —
    ///    delegation may narrow a role, never widen one.
    ///
    /// Everything else is an [`IdentityRefusal::DelegationRefused`]: a distinct
    /// error class, because "you are not the gateway", "that actor is not
    /// delegable" and "that role is not theirs" are different operator stories
    /// from `identity_not_bound`. A refusal leaves nothing behind — the audit
    /// record of a delegated request is written only after this returns `Ok`
    /// (`record_delegation_audit` in the server).
    ///
    /// The target's roles are read from *its* binding, at whichever uid that
    /// binding names. A delegated request has no connection of the end user's
    /// own, so the binding is consulted for the standing it declares; its uid
    /// answers only the separate question of which connection may act as that
    /// actor directly.
    pub fn resolve_delegated(
        &self,
        claim: Option<&ActorClaim>,
        on_behalf_of: Option<&ActorClaim>,
        gateway: Option<&GatewayConfig>,
        peer: Option<PeerIdentity>,
    ) -> Result<ResolvedActor, IdentityRefusal> {
        let Some(target) = on_behalf_of else {
            return self.resolve_direct(claim, peer);
        };
        let claim = claim.ok_or(IdentityRefusal::Missing)?;
        let peer = peer.ok_or(IdentityRefusal::UnknownPeer)?;
        if self.is_empty() {
            return Err(IdentityRefusal::Unconfigured);
        }
        let Some(gateway) = gateway else {
            return Err(delegation_refused(format!(
                "this cell configures no `gateway` principal, so no connection may send \
                 `on_behalf_of` (uid {} did)",
                peer.uid
            )));
        };
        if peer.uid != gateway.uid {
            return Err(delegation_refused(format!(
                "uid {} is not the configured gateway uid {}, so it may not send `on_behalf_of`",
                peer.uid, gateway.uid
            )));
        }
        if claim.actor_id != gateway.actor {
            return Err(delegation_refused(format!(
                "`on_behalf_of` may only be sent by the gateway principal `{}`; this request \
                 claims to be `{}`",
                gateway.actor, claim.actor_id
            )));
        }
        // The gateway is a governed principal too: its own claim must name the
        // configured gateway actor at the configured uid, holding the role it
        // claims, before anything it delegates is considered at all.
        let Some(gateway_binding) = self
            .bindings
            .iter()
            .find(|binding| binding.uid == gateway.uid && binding.actor_id == gateway.actor)
        else {
            return Err(delegation_refused(format!(
                "the configured gateway principal `{}` has no binding for uid {}",
                gateway.actor, gateway.uid
            )));
        };
        if !gateway_binding.roles.contains(&claim.role) {
            return Err(IdentityRefusal::RoleNotHeld {
                actor_id: claim.actor_id.clone(),
                role: claim.role.clone(),
            });
        }
        if target.actor_id == gateway.actor {
            return Err(delegation_refused(format!(
                "the gateway principal `{}` may not act as itself through `on_behalf_of`",
                gateway.actor
            )));
        }
        if !gateway
            .delegable_actors
            .iter()
            .any(|actor| actor == &target.actor_id)
        {
            return Err(delegation_refused(format!(
                "actor `{}` is not in this cell's gateway-delegable allowlist",
                target.actor_id
            )));
        }
        // The end user's roles, collected from every binding that names them.
        // No binding means no established standing: refused, never assumed.
        let held: Vec<&ActorRole> = self
            .bindings
            .iter()
            .filter(|binding| binding.actor_id == target.actor_id)
            .flat_map(|binding| binding.roles.iter())
            .collect();
        if held.is_empty() {
            return Err(delegation_refused(format!(
                "actor `{}` has no binding in this cell, so its standing cannot be established",
                target.actor_id
            )));
        }
        if !held.contains(&&target.role) {
            return Err(delegation_refused(format!(
                "role {:?} exceeds the roles bound to `{}`",
                target.role, target.actor_id
            )));
        }
        Ok(ResolvedActor {
            actor_id: target.actor_id.clone(),
            role: target.role.clone(),
            uid: peer.uid,
            delegation: Some(DelegationProvenance {
                gateway_actor_id: gateway.actor.clone(),
                gateway_uid: gateway.uid,
            }),
        })
    }

    /// The direct resolution: `claim` against the connection's own uid.
    fn resolve_direct(
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
            delegation: None,
        })
    }
}

/// An `on_behalf_of` the gateway rules refuse (T02, D-2).
///
/// Built as a typed variant rather than spelled at each of the six refusal
/// sites, so the class and the message stay one vocabulary.
fn delegation_refused(reason: String) -> IdentityRefusal {
    IdentityRefusal::DelegationRefused { reason }
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

/// The actor a delegated request is attributed to (`identity.get`, T02).
///
/// Reported only when the connection presented both an `actor` block and an
/// `on_behalf_of` block that the gateway rules accept. A direct connection
/// learns its own claimable actors from `available` instead.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct EffectiveActorView {
    /// The end-user actor every record from this request will name.
    pub actor_id: String,
    /// The wire spelling of the role that actor holds and was claimed for it.
    pub role: String,
    /// The gateway principal that spoke for this actor, and its uid.
    pub delegated_by: DelegatedByView,
}

/// The gateway principal behind a delegated request.
///
/// Reported on the gateway's own connection, so it reveals nothing the caller
/// did not already present — and nothing about any other uid's bindings.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct DelegatedByView {
    pub actor_id: String,
    pub uid: u32,
}

impl EffectiveActorView {
    /// The view of a *delegated* resolution, or `None` for a direct one.
    pub fn of(actor: &ResolvedActor) -> Option<Self> {
        let provenance = actor.delegation()?;
        Some(Self {
            actor_id: actor.actor_id().into(),
            role: role_wire_name(actor.role()),
            delegated_by: DelegatedByView {
                actor_id: provenance.gateway_actor_id().into(),
                uid: provenance.gateway_uid(),
            },
        })
    }
}

/// Why nothing can be claimed on this connection, in the same vocabulary a
/// refused protected verb uses — so a client never has to map one set of
/// reasons onto another.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct RefusalView {
    pub error_class: String,
    pub message: String,
    /// Inspect clients can state the effect of a prospective refusal without
    /// attempting a protected command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_side_effect: Option<bool>,
    /// The server-owned repair path for this error class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_lawful_action: Option<String>,
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
    /// The actor a *delegated* request would be attributed to, when this
    /// request carried both an `actor` and an `on_behalf_of` block the gateway
    /// rules accept (T02, D-2). Absent on a direct connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_actor: Option<EffectiveActorView>,
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
        | Request::AuthorityRequest { .. }
        | Request::AuthorityApproval { .. }
        | Request::AuthorityEvidence { .. }
        | Request::AuthoritySettle { .. }
        | Request::CaseCommit { .. }
        | Request::ApprovalDecide { .. }
        | Request::CaseAddItem { .. }
        | Request::CaseReopen { .. }
        | Request::CaseTerminate { .. }
        | Request::CaseAdvance { .. }
        | Request::ItemExecute { .. }
        | Request::HumanTaskComplete { .. } => true,

        // Read-only projections and protocol chatter. An old client that never
        // learned about the actor block keeps working against all of these.
        Request::Status { .. }
        | Request::AgentList
        | Request::AuthorityApprovals
        | Request::AuthorityTransitions
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
        | Request::DelegationList
        // Content-addressed read over already-committed evidence records.
        | Request::ArtifactGet { .. } => false,
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

    // -----------------------------------------------------------------------
    // Delegated identity (T02, D-2)
    // -----------------------------------------------------------------------

    /// A cell with a gateway principal at uid 2000, two delegable end users
    /// bound at uids no connection presents, and a security officer bound but
    /// *not* delegable.
    fn gateway_cell() -> (IdentityBindings, GatewayConfig) {
        let bindings = IdentityBindings {
            bindings: vec![
                IdentityBinding {
                    uid: 2000,
                    actor_id: "gateway".into(),
                    roles: vec![ActorRole::Service],
                },
                IdentityBinding {
                    uid: 4242,
                    actor_id: "operator_a".into(),
                    roles: vec![ActorRole::Operator],
                },
                IdentityBinding {
                    uid: 4243,
                    actor_id: "operator_b".into(),
                    roles: vec![ActorRole::Operator],
                },
                IdentityBinding {
                    uid: 4244,
                    actor_id: "security_officer".into(),
                    roles: vec![ActorRole::SecurityOfficer],
                },
            ],
        };
        let gateway = GatewayConfig {
            uid: 2000,
            actor: "gateway".into(),
            delegable_actors: vec!["operator_a".into(), "operator_b".into()],
        };
        (bindings, gateway)
    }

    fn gateway_claim() -> ActorClaim {
        claim("gateway", ActorRole::Service)
    }

    #[test]
    fn the_gateway_may_act_for_an_allowlisted_end_user() {
        let (bindings, gateway) = gateway_cell();
        let resolved = bindings
            .resolve_delegated(
                Some(&gateway_claim()),
                Some(&claim("operator_a", ActorRole::Operator)),
                Some(&gateway),
                Some(PeerIdentity { uid: 2000 }),
            )
            .expect("an allowlisted end user must resolve");

        // The effective actor is the end user; the gateway rides alongside.
        assert_eq!(resolved.actor_id(), "operator_a");
        assert_eq!(resolved.role(), &ActorRole::Operator);
        assert_eq!(resolved.uid(), 2000, "the connection is the gateway's");
        let provenance = resolved.delegation().expect("delegated");
        assert_eq!(provenance.gateway_actor_id(), "gateway");
        assert_eq!(provenance.gateway_uid(), 2000);
        assert!(resolved.is_delegated());
    }

    /// The end user's binding is consulted for standing, not for the socket it
    /// owns: this connection is uid 2000 and the binding says 4242.
    #[test]
    fn the_end_users_binding_uid_is_not_the_connection_uid() {
        let (bindings, gateway) = gateway_cell();
        let resolved = bindings
            .resolve_delegated(
                Some(&gateway_claim()),
                Some(&claim("operator_b", ActorRole::Operator)),
                Some(&gateway),
                Some(PeerIdentity { uid: 2000 }),
            )
            .expect("the end user's binding uid is not the connection's");
        assert_eq!(resolved.actor_id(), "operator_b");
    }

    #[test]
    fn a_non_gateway_uid_sending_on_behalf_of_is_refused() {
        let (bindings, gateway) = gateway_cell();
        let refusal = bindings
            .resolve_delegated(
                Some(&claim("operator_a", ActorRole::Operator)),
                Some(&claim("operator_b", ActorRole::Operator)),
                Some(&gateway),
                Some(PeerIdentity { uid: 4242 }),
            )
            .expect_err("uid 4242 is not the gateway");
        assert_eq!(refusal.error_class(), "identity_delegation_refused");
        assert_eq!(refusal.response()["no_side_effect"], true);
    }

    #[test]
    fn an_unconfigured_gateway_refuses_every_delegation() {
        let (bindings, _gateway) = gateway_cell();
        let refusal = bindings
            .resolve_delegated(
                Some(&gateway_claim()),
                Some(&claim("operator_a", ActorRole::Operator)),
                None,
                Some(PeerIdentity { uid: 2000 }),
            )
            .expect_err("a cell with no gateway section must refuse all delegation");
        assert_eq!(refusal.error_class(), "identity_delegation_refused");
        assert!(
            refusal
                .message()
                .contains("configures no `gateway` principal"),
            "{}",
            refusal.message()
        );
    }

    #[test]
    fn an_actor_outside_the_allowlist_is_refused() {
        let (bindings, gateway) = gateway_cell();
        let refusal = bindings
            .resolve_delegated(
                Some(&gateway_claim()),
                // Bound, holds the role, but the operator never allowlisted an
                // actor for the gateway to speak for it.
                Some(&claim("security_officer", ActorRole::SecurityOfficer)),
                Some(&gateway),
                Some(PeerIdentity { uid: 2000 }),
            )
            .expect_err("not delegable");
        assert_eq!(refusal.error_class(), "identity_delegation_refused");
        assert!(
            refusal.message().contains("allowlist"),
            "{}",
            refusal.message()
        );
    }

    /// Delegation may narrow a role, never widen one.
    #[test]
    fn a_role_the_end_user_does_not_hold_is_refused() {
        let (bindings, gateway) = gateway_cell();
        let refusal = bindings
            .resolve_delegated(
                Some(&gateway_claim()),
                Some(&claim("operator_a", ActorRole::SecurityOfficer)),
                Some(&gateway),
                Some(PeerIdentity { uid: 2000 }),
            )
            .expect_err("operator_a holds only operator");
        assert_eq!(refusal.error_class(), "identity_delegation_refused");
        assert!(
            refusal.message().contains("exceeds the roles bound to"),
            "{}",
            refusal.message()
        );
    }

    #[test]
    fn an_allowlisted_actor_with_no_binding_is_refused() {
        let (bindings, mut gateway) = gateway_cell();
        gateway.delegable_actors.push("ghost".into());
        let refusal = bindings
            .resolve_delegated(
                Some(&gateway_claim()),
                Some(&claim("ghost", ActorRole::Operator)),
                Some(&gateway),
                Some(PeerIdentity { uid: 2000 }),
            )
            .expect_err("nothing binds `ghost`, so nothing establishes its standing");
        assert_eq!(refusal.error_class(), "identity_delegation_refused");
        assert!(
            refusal.message().contains("no binding"),
            "{}",
            refusal.message()
        );
    }

    /// The gateway's own standing is checked too: it must claim the actor the
    /// cell bound it as, and hold the role it claims.
    #[test]
    fn the_gateway_claim_must_be_the_configured_gateway_actor() {
        let (bindings, gateway) = gateway_cell();
        let refusal = bindings
            .resolve_delegated(
                // uid 2000's *other* claimable actor, acting as the gateway.
                Some(&claim("operator_a", ActorRole::Operator)),
                Some(&claim("operator_b", ActorRole::Operator)),
                Some(&gateway),
                Some(PeerIdentity { uid: 2000 }),
            )
            .expect_err("only the gateway principal may delegate");
        assert_eq!(refusal.error_class(), "identity_delegation_refused");
    }

    #[test]
    fn a_gateway_role_it_does_not_hold_is_refused() {
        let (bindings, gateway) = gateway_cell();
        let refusal = bindings
            .resolve_delegated(
                Some(&claim("gateway", ActorRole::Operator)),
                Some(&claim("operator_a", ActorRole::Operator)),
                Some(&gateway),
                Some(PeerIdentity { uid: 2000 }),
            )
            .expect_err("the gateway holds service, not operator");
        assert_eq!(refusal.error_class(), "identity_role_not_held");
    }

    #[test]
    fn the_gateway_may_not_act_as_itself() {
        let (bindings, gateway) = gateway_cell();
        let refusal = bindings
            .resolve_delegated(
                Some(&gateway_claim()),
                Some(&claim("gateway", ActorRole::Service)),
                Some(&gateway),
                Some(PeerIdentity { uid: 2000 }),
            )
            .expect_err("delegating to itself is not delegation");
        assert_eq!(refusal.error_class(), "identity_delegation_refused");
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

    /// `on_behalf_of` parses off the raw line exactly as documented, and a
    /// garbled block reads as absent — never as a half-believed actor.
    #[test]
    fn an_on_behalf_of_block_is_read_off_the_raw_line() {
        assert_eq!(
            ActorClaim::parse_on_behalf_of(
                r#"{"verb":"case_commit","actor":{"actor_id":"gateway","role":"service"},"on_behalf_of":{"actor_id":"operator_a","role":"operator"}}"#
            ),
            Some(claim("operator_a", ActorRole::Operator))
        );
        assert_eq!(ActorClaim::parse_on_behalf_of(r#"{"verb":"submit"}"#), None);
        assert_eq!(
            ActorClaim::parse_on_behalf_of(r#"{"verb":"submit","on_behalf_of":"operator_a"}"#),
            None
        );
        assert_eq!(
            ActorClaim::parse_on_behalf_of(r#"{"verb":"submit","on_behalf_of":{"actor_id":"x"}}"#),
            None,
            "a delegation without a role is not a delegation"
        );
    }

    /// The direct path is untouched: an absent `on_behalf_of` resolves exactly
    /// as it always did, with no provenance attached.
    #[test]
    fn a_direct_resolution_carries_no_delegation() {
        let (bindings, _gateway) = gateway_cell();
        let resolved = bindings
            .resolve(Some(&gateway_claim()), Some(PeerIdentity { uid: 2000 }))
            .expect("the gateway's own claim resolves directly");
        assert_eq!(resolved.actor_id(), "gateway");
        assert!(!resolved.is_delegated());
        assert!(resolved.delegation().is_none());
    }

    /// The inspect view reports the effective actor, and reports refusals in
    /// the same vocabulary a protected verb uses.
    #[test]
    fn describe_delegated_reports_the_effective_actor_and_refusals() {
        let (bindings, gateway) = gateway_cell();

        let view = bindings.describe_delegated(
            Some(&gateway_claim()),
            Some(&claim("operator_a", ActorRole::Operator)),
            Some(&gateway),
            Some(PeerIdentity { uid: 2000 }),
        );
        let effective = view.effective_actor.expect("reported");
        assert_eq!(effective.actor_id, "operator_a");
        assert_eq!(effective.role, "operator");
        assert_eq!(effective.delegated_by.actor_id, "gateway");
        assert!(view.refusal.is_none());

        let refused = bindings.describe_delegated(
            Some(&gateway_claim()),
            Some(&claim("security_officer", ActorRole::SecurityOfficer)),
            Some(&gateway),
            Some(PeerIdentity { uid: 2000 }),
        );
        assert!(refused.effective_actor.is_none());
        assert_eq!(
            refused.refusal.expect("refusal view").error_class,
            "identity_delegation_refused"
        );

        // Without `on_behalf_of` the view is the plain connection view.
        let plain = bindings.describe_delegated(
            Some(&gateway_claim()),
            None,
            Some(&gateway),
            Some(PeerIdentity { uid: 2000 }),
        );
        assert!(plain.effective_actor.is_none());
        assert_eq!(plain.available.len(), 1);
    }
}
