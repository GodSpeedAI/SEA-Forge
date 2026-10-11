# T02 independent confirmation — delegated actor identity for the gateway principal

- **Verdict: APPROVE** (prereg claim verified; one medium ADR-accuracy finding to
  correct as follow-up — it does not falsify the claim).
- Plan: `casework-live-wiring-production` T02 (proof P2, confirmation=independent).
- Prereg: `.agents/evidence/casework-live-wiring/prereg/T02.yaml`.
- Under review: commit `85545a6` on `casework/live-wiring`.
- Critic: session-3 independent critic (did not build T02). All commands and
  attacks below were executed personally by the critic on 2026-09-24/25; logs and
  drivers under `.agents/evidence/casework-live-wiring/T02/critic/`.
- Prereg claim: "on_behalf_of is honoured only from the gateway principal and
  never widens roles beyond the target actor's bindings."
- Prereg falsifier: "Any accepted request whose effective actor is not in the
  allowlist, or whose role exceeds that actor's binding." — **not found**; every
  candidate path was attacked and refused (findings 2–4).
- Prereg confirmation_condition: "An independent agent's negative tests pass on a
  fresh cell." — **satisfied** (finding 2).

## 1. Gates (all re-run by the critic with true exit capture)

| # | Command | Exit | Log |
|---|---------|------|-----|
| 1 | `cargo test -p sea-forge-server identity` | 0 | `critic/gate-1-identity.log` (22 identity unit tests in lib; 3 in conformance_identity) |
| 2 | `cargo test -p sea-forge-server --test sfwp_delegated_identity` | 0 | `critic/gate-2-sfwp-delegated-identity.log` — **9/9 pass** |
| 3 | `cargo test -p sea-forge-server --test '*delegat*'` | 0 | `critic/gate-3-delegat-glob.log` — 14+12+9 = 35 pass |
| 4 | `cargo test -p sea-forge-cli -p sea-forge-case-runner -p sea-forge-server` | 0 | `critic/gate-4-three-crate.log` — 58 test binaries, 0 failures |
| 5 | `cargo fmt --all -- --check` | 0 | `critic/gate-5-fmt.log` (empty) |
| 6 | `just no-async-kernel` | 0 | `critic/gate-6-no-async-kernel.log` |
| 7 | `just workbench-contracts-gate` | 0 | `critic/gate-7-workbench-contracts.log` (regen byte-stable) |
| 8 | `cargo test -p sea-forge-planner` (prereg global gate) | 0 | `critic/gate-8-planner.log` |

## 2. Negative tests on FRESH cells (prereg confirmation_condition) — real server binary, real Unix socket

Driver: `critic/attack_driver.py`; transcript: `critic/runtime-attacks.log`. The
driver boots the actual `target/debug/sea-forge-server` as a separate process on
three fresh temp cells (seeds mirror `tests/sfwp_delegated_identity.rs`:
gateway at uid 1000, end users bound at uid 4242 — not the process uid, per the
prereg confound mitigation). Every refusal was captured verbatim AND the cell's
`ledgers/**` + `approvals.jsonl` were sha256-hashed (sorted relpath+content)
before/after to prove zero writes.

- **(a) on_behalf_of from a uid bound as operator** (cell: this uid bound as
  `operator_local`; gateway configured at uid 4242): refused
  `identity_delegation_refused` — "uid 1000 is not the configured gateway uid
  4242, so it may not send `on_behalf_of`", `no_side_effect: true`; digest
  unchanged, no case, no delegation-audit ledger. **PASS**
- **(b) gateway section omitted entirely**: refused
  `identity_delegation_refused` — "this cell configures no `gateway` principal…",
  digest unchanged; the same connection's direct claim still commits a case
  (refusing delegation does not disable bound actors). **PASS**
- **(c) bound-but-not-allowlisted actor** (`operator_c`, bound with a role,
  absent from `delegable_actors`): refused — "actor `operator_c` is not in this
  cell's gateway-delegable allowlist"; digest unchanged. **PASS**
- **(d) role the target lacks** (`security_officer` allowlisted but holds only
  R-SO; gateway claims `operator` for it): refused — "role Operator exceeds the
  roles bound to `security_officer`"; digest unchanged. **PASS**
- extra: unknown actor `ghost` → refused at the same allowlist check. **PASS**
- extra: garbled `on_behalf_of` (a bare string) parses as absent
  (`ActorClaim::block`, identity.rs:124-133) and the request falls to the direct
  path, where the entity-consistency gate refused it
  (`identity_entity_mismatch`, "verified actor `gateway` but attributes its work
  to `operator_a`"). Judged **fail-closed**: a malformed block can only LOSE the
  delegation — the direct path still requires the connection's own binding and a
  held role, so it can never become a privilege. **PASS**

## 3. Positive dual-principal proof (runtime, same driver)

Two end users on two sockets through ONE gateway principal: A submits, proposes
a discretionary item (`proposed_by: "operator_a"`) and executes it (escalation
opens approval `apr_0001`); B resolves it (`approval_decide` →
`"resolved_by=operator_b"`). Durable reads:

- `ledgers/delegation-audit/entries.jsonl`: 4 records, one per admitted
  delegated request, each naming BOTH principals —
  `{gateway_actor_id: "gateway", gateway_uid: 1000, effective_actor_id:
  "operator_a"|"operator_b", effective_role: "operator", verb, request_id}`.
- `approval_resolution` ledger payload: `resolved_by: "operator_b"`,
  `status: "approved"`; escalation `action_request.actor.actor_id` =
  `operator_a` — the SoD-comparable fields name END USERS only.
- Gateway appears in NO `actor_id`/`principal`/`resolved_by` field in the case
  ledger or `approvals.jsonl` (string-scanned).
- `identity_get` with `on_behalf_of` returns
  `effective_actor: {actor_id: "operator_a", role: "operator", delegated_by:
  {actor_id: "gateway", uid: 1000}}` — the additive inspect extension works on
  the wire.

## 4. Falsifier hunt (code + runtime)

- `IdentityBindings::resolve_delegated` (identity.rs:512-611): six ordered
  fail-closed checks — gateway configured; peer uid == gateway uid; claim names
  the gateway actor; gateway binding exists at (uid, actor); gateway holds its
  claimed role; target != gateway; target in `delegable_actors`; target has a
  binding; target's binding holds the claimed role. No wildcard, no fallback to
  a wider standing. `Ok` is reachable only past all checks.
- **Replay/dedupe ordering**: the identity gate runs in `dispatch_bounded`
  (lib.rs:1626-1699) BEFORE `dispatch_admitted`'s dedupe (lib.rs:1487-1563), so
  a replayed request_id from a non-gateway uid is refused at the gate before the
  recorded outcome can be served. Runtime probe (`critic/replay_audit_probe.py`,
  log `critic/replay-unwritable-probe.log`): identical replay returns the
  recorded outcome with exactly ONE audit record; same-id+different-actor is
  refused (`identity_entity_mismatch`, gate-first); same-id+different-payload is
  refused (`request_id_reused`). No re-attribution path exists.
- **Unrecordable delegation**: the audit write sits after dedupe and before any
  handler (lib.rs:1565-1596). Runtime-proven fail-closed: with the
  delegation-audit entries file made unwritable, the delegated request was
  REFUSED `delegation_audit_unwritable`, `no_side_effect: true`, no case
  created, cell digest unchanged. (First probe run also showed that if the audit
  *succeeds* and a later mid-handler ledger write fails, the failure surfaces on
  the generic pre-existing commit error path — not the delegation gate; see
  finding 9.)
- **identity.get / inspect verbs**: read-only; `on_behalf_of` is consulted only
  by IdentityGet (to report, never to authorize); all other inspect verbs ignore
  it. No widening surface.
- **Supervisor**: `gateway.delegable_actors` may not contain the supervisor
  actor — refused at config load (config.rs:327-331), unit-tested in
  `load_rejects_invalid_gateway_settings` (config.rs:770-778). The supervisor
  path (T04) attributes its own actor and never crosses the delegation gate.
- **Config fail-closed**: absent `gateway:` section → `GatewayConfig::None` →
  all delegation refused (unit test `an_absent_gateway_section_parses_to_none_fail_closed`,
  config.rs:615); load also refuses a gateway uid with no binding, an
  allowlisted actor with no/empty binding, duplicates, self-delegation, and
  invalid identifiers (config.rs:292-365).

## 5. Wire additivity (ADR-003)

- `pub enum Request` and `pub enum Response` are **byte-identical** between
  `85545a6^` and `85545a6` (critic diff, 322 lines each side, empty diff); the
  commit touches no file under `crates/sea-forge-server/src/sfwp/`.
- `on_behalf_of`/`actor` are parsed off the RAW NDJSON line
  (`ActorClaim::parse_on_behalf_of`, identity.rs:124-133; call sites
  lib.rs:1615-1632) — no Request variant was reshaped.
- Workbench regen is purely additive: `IdentityView.schema.json` /
  `IdentityView.ts` / validator add optional `effective_actor` +
  `EffectiveActorView`/`DelegatedByView`; `required` stays
  `["configured","available"]`. `just workbench-contracts-gate` exits 0 (finding 1).
- `justfile` `casework-cell-init` binds `operator_local`, `security_officer`,
  `lifecycle_custodian`, and `gateway` on the single local uid with
  `delegable_actors: [operator_local, security_officer, lifecycle_custodian]` —
  the allowlist bounds delegation in the shared-uid local topology (documented
  prereg confound; the committed tests mitigate it by binding end users at a
  foreign uid).

## 6. ADR completeness vs the plan's D-2 requirement list

`.agents/reports/casework-live-wiring/adr-identity-delegation.md`:

- Threat model — present ("Threat model": the three cumulative conditions; a
  CAN/CANNOT list including "a lying gateway is out of scope (T07)").
- Revocation — present ("Revocation"), **but one sentence is empirically
  inaccurate** (finding 7).
- Audit fields — present and concrete ("Audit fields": kernel records name the
  end user; the server appends `delegated_request`
  `{verb, request_id?, gateway_actor_id, gateway_uid, effective_actor_id,
  effective_role}` to `<root>/ledgers/delegation-audit`, written after dedupe
  and before handlers; fail-closed `delegation_audit_unwritable` — all verified
  at runtime above).
- identity.rs:14-27 amendment scope — present ("Amends": adds a fifth clause to
  U-07's four; "Nothing in U-07 1-4 is weakened" — verified: the direct path
  `resolve_direct` is unchanged in behavior and still the default).
- Supervisor never delegable — present (Decision: "The supervisor actor is never
  delegable") and enforced at config load.
- D-2's "role 'gateway'" wording — deviated consciously (no kernel
  `ActorRole::Gateway` variant); see finding 8.

## 7. Findings

1. **PASS — gates** (evidence in §1).
2. **PASS — prereg teeth on fresh cells** (evidence in §2).
3. **PASS — falsifier not found** across resolve_delegated, replay/dedupe,
   identity.get, inspect verbs, supervisor, malformed fallback (evidence in §4).
4. **PASS — positive dual-principal + end-user-only SoD** (evidence in §3).
5. **PASS — wire additivity** (evidence in §5).
6. **INFO — correlation marker precedes the audit write.** A request refused as
   `delegation_audit_unwritable` has already bound a pending correlation record
   (dedupe admission precedes the audit, per ADR design); the lib.rs:1573
   comment "nothing has run yet, so that refusal still leaves nothing behind" is
   not literally true of that pending marker (a same-id retry gets "pending",
   not re-execution). No governance write; honest idempotency behavior. Accept.
7. **MEDIUM — ADR revocation immediacy is overstated.** The ADR claims
   "Bindings and allowlist entries are read from the live server config on every
   request … Removing a binding or allowlist entry takes effect on the NEXT
   request" and "no session, token, or cache exists to expire". In reality the
   gate reads the in-memory snapshot (`state.config()`), and the only runtime
   `reload_config()` caller is `commit_plan` (lib.rs:2764; Submit/CaseCommit).
   Critic probe (`critic/revocation_probe.py`, log `critic/revocation-probe.log`):
   after removing `operator_b` from the allowlist, a delegated `identity_get`
   STILL resolved `operator_b` as effective actor; only after one delegated
   `case_commit` forced the reload did the refusal appear. A removed allowlist
   entry therefore keeps working for non-commit traffic until some commit
   happens or the server restarts. This does not falsify the prereg claim (the
   config-as-loaded rules are enforced), but the ADR sentence is false as
   written and the revocation story should be corrected (and/or the gate should
   re-read the file). **Required follow-up before settlement paperwork is
   considered final; does not block this approval of the claim.**
8. **LOW — D-2 wording deviation, documented.** D-2's option text binds the
   gateway uid to "role 'gateway'"; the implementation uses a `gateway:` section
   plus a normal identity binding (role `service` in the live cell), because
   `ActorRole::Gateway` would be a kernel-crate enum change (ask-first). The ADR
   explains this ("Gateway principal: which role") with fail-closed
   consequences. Acceptable, but it is a deviation from the approved decision
   text and the "recorded as an open question" lives only in ADR prose — there
   is no entry in `.agents/OPEN_QUESTIONS.md` or the decision log. Add one.
9. **INFO (out of T02 scope) — untyped mid-handler error shape.** During probe
   iteration, a delegated commit whose CASE-ledger mkdir failed (permissions)
   returned an untyped `{"error": "create ledger directory: Permission denied
   (os error 13)"}` (no `error_class`, no `no_side_effect`) and left a partial
   `cases/<id>/runs/` directory. The delegation audit had already been written
   correctly (dual-principal attribution durable). This is the pre-existing
   generic commit failure path, not the delegation gate; flag for T11
   hardening.
10. **INFO — no `gateway.actor != supervisor.actor` validation.** A cell may
    name the gateway principal "supervisor", conflating attribution streams. No
    authority is gained (standing comes from bindings; the supervisor actor
    still cannot be a delegation TARGET). Cosmetic.

## 8. Deviations from plan T02 / prereg

- Plan step 3 says "Thread the effective actor through every protected verb,
  including the run_cli --actor path." The `run_cli` approval.decide path was
  deleted by T04 (commit 1b73724) in favour of the in-process case-ops path, so
  T02 threads via `handle_request_as`/`case_dispatch` instead — consistent with
  T04's approved plan; no run_cli bypass remains.
- D-2 "role 'gateway'" → binding-based gateway standing (finding 8).
- Prereg confound mitigation honored: the committed suite binds end users at a
  foreign uid (4242/4243) and documents the uid it runs as
  (sfwp_delegated_identity.rs:28-33, 44-58).
- Everything else matches the plan steps (ADR, fail-closed schema default,
  threading, integration tests, gates).

## 9. Evidence index (critic-produced)

- `critic/attack_driver.py`, `critic/runtime-attacks.log` — fresh-cell attacks + positive.
- `critic/revocation_probe.py`, `critic/revocation-probe.log` — finding 7.
- `critic/replay_audit_probe.py`, `critic/replay-unwritable-probe.log` — replay/dedupe + unwritable audit.
- `critic/gate-{1..8}-*.log` — gates with true exit codes.

## 10. Conclusion

The prereg claim is TRUE under attack: `on_behalf_of` is honoured only from the
configured gateway principal, only for allowlisted actors, only for roles the
target's own binding grants, with both principals durably recorded before any
handler runs and every refusal leaving the cell byte-identical. The prereg's
confirmation condition (independent negative tests on a fresh cell) is
satisfied. **T02 is approved for settlement**, conditional on the ADR revocation
correction (finding 7) and the OPEN_QUESTIONS entry (finding 8) landing as
follow-ups.
