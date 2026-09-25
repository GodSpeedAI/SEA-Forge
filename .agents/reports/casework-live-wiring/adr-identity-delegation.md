# ADR: Delegated actor identity for the gateway principal (T02, D-2)

- **Status:** accepted 2026-09-24 (implements operator decision D-2,
  approved 2026-09-23; see decision-log.yaml).
- **Plan:** casework-live-wiring-production T02 (proof P2).
- **Prereg:** .agents/evidence/casework-live-wiring/prereg/T02.yaml.
- **Amends:** the identity contract at identity.rs:14-27 (U-07, four parts:
  per-request actor block; socket-uid derivation; ledger-compared approver;
  optional-on-inspect/required-on-protected). This ADR adds a fifth,
  narrowly-scoped clause below. Nothing in U-07 1-4 is weakened.

## Decision

The server binds the gateway uid to a gateway principal in server.yaml.
Only that principal may send `on_behalf_of`, and only for actors in a
gateway-delegable allowlist. Every ledger/trace record from a delegated
request carries BOTH principals. SoD compares end-user actors, never the
gateway. identity.get reports the effective actor. Absent gateway section
refuses all on_behalf_of (fail-closed). The supervisor actor is never
delegable; supervisor passes attribute to the supervisor actor only.

Wire: `on_behalf_of` is an optional sibling of `actor` on protected
requests. Parsed off the raw NDJSON line; no Request variant reshaped
(ADR-003). Go/bun gates unaffected.

Refusal class: `identity_delegation_refused` (new, distinct from
`identity_not_bound`). No ledger write on refusal.

## Threat model

Socket-uid-only identity answers "which OS user opened this socket". A
multi-user gateway collapses N browser users into ONE OS uid. Honouring
the gateway's bare `actor` claim would let it assert any actor; ignoring
per-user attribution would collapse SoD (B could approve A's proposal).

`on_behalf_of` is honoured only when ALL three hold:

1. Peer uid == gateway uid binding (SO_PEERCRED; assertion proves
   nothing). Operator-bound or unbound uids sending it are refused.
2. Target actor_id is in the delegable allowlist. The gateway names no
   arbitrary actors; unknown targets are refused.
3. Target role is a subset of the target actor's binding roles (narrow
   allowed, widen refused).

CAN: select the effective actor for attribution, authority evaluation, SoD
and `identity.get` — one request, gateway only, allowlisted actor, held
roles. CANNOT: authenticate the browser user (T07 owns that; a lying
gateway is out of scope); widen roles; reach the supervisor actor; bypass
entity consistency (an `entity` that does not name the end user is still
`identity_entity_mismatch`); bypass the allowlist or revocation; emit a
single-principal record. `identity.get` on a gateway connection reports the
gateway's own claimable set, never the allowlist.

## Gateway principal: which role

D-2 says the server "binds the gateway uid to role 'gateway' in
server.yaml". `ActorRole` (sea-forge-core/src/types.rs:299) has no `Gateway`
variant and is a kernel-crate enum that serializes into ledgers, so adding
one is a kernel change outside D-2's scope (and ask-first under the root
AGENTS.md). The delegation standing of a connection therefore comes from
the `gateway:` section (uid + actor), not from a role name: the gateway is
bound in `identity.bindings` like any other actor and its own claim must
name that actor and hold a role the binding grants (in the live cell,
`service`). Consequences, all fail-closed: a gateway uid with no binding
cannot delegate; an unbound `gateway:` uid is refused at config load; the
gateway's own claim is verified before anything it delegates is considered.
Whether to add `ActorRole::Gateway` later is an operator decision, recorded
as an open question — nothing in this ADR depends on the answer.

## Where the end user's roles come from

Rule 3 reads the target's roles from the *target's own* binding(s) in
`identity.bindings`, at whatever uid those bindings name. A delegated
request has no connection of the end user's own, so the binding is
consulted for the standing it declares; its uid answers only the separate
question of which connection may act as that actor directly. An allowlisted
actor with no binding (or with no role) is refused, and the config loader
refuses such a cell at startup rather than discovering it at the first
delegated request.

## Revocation

Bindings and allowlist entries are read from the live server config on
every request (the gate resolves against state.config() per request).
Removing a binding or allowlist entry takes effect on the NEXT request:
no session, token, or cache exists to expire. In-flight requests keep the
ResolvedActor they verified; revocation cannot rewrite a verified actor
mid-flight nor retract committed ledger records (still honestly
attributed to both principals at decision time). Removing the whole
gateway section disables all delegation from the next request. Changing
the gateway uid re-binds the principal; the old uid is refused next.

## Audit fields

Every ledger/trace record from a delegated request carries BOTH
principals: writer_identity_ref and TraceEvent.actor_id name the END-USER
actor (SoD, approval_submitter, single-principal readers unchanged); the
gateway principal (gateway actor_id + uid) rides alongside in the
request's ledger/trace payload so an auditor answers "which gateway spoke
for this end user" without connection logs. identity.get reports the
effective actor. Refusals write nothing (no_side_effect: true).
SoD compares end-user actors only; the gateway uid never enters a SoD
comparison.

Where that pair is durable, concretely (T02 implementation): the kernel
records only the end user, as it must. The server therefore appends one
record per *admitted* delegated protected request to its own ledger,
`<root>/ledgers/delegation-audit`, record kind `delegated_request`, fields
`{verb, request_id?, gateway_actor_id, gateway_uid, effective_actor_id,
effective_role}` — written after the request passes the identity gate and
the dedupe admission, and before any handler runs, so a replayed or
refused request adds nothing. Fail-closed: if that record cannot be
written the request is refused (`delegation_audit_unwritable`,
`no_side_effect: true`) rather than run with its gateway attribution
missing.

## Local topology note

In the live single-machine stack the gateway process and the browser users
share one OS uid, so one uid legitimately holds the gateway binding *and*
the end-user bindings. That is not a weakening of rule 1: the rule is
about which uid may *speak* for others, and it is enforced against the
gateway section's uid, not against the end users'. On a multi-machine
deployment the gateway has its own uid and the end users have none.

