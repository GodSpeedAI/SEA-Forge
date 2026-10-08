# T05 Independent Adversarial Verification — E5A / E5B / I5

Verifier: fresh, independent of the T05 builder work. Attack battery designed
BEFORE opening `.agents/evidence/e2e/T05/t05-report.md` (which was read only at
step E). Freeze SHA256 of `.agents/specs/e2e-preregistration.yml`
before/after: recorded below.

## Step A — Freeze check

- Before: `just e2e-prereg-check` → VERDICT: PASS (exit 0),
  expected == observed == `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`.
- After (re-run after all gates): **PASS** (exit 0), same SHA
  `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`.

## Frozen requirements under attack

- E5A (`AuthorizedInvocation`, sea_forge -> execution_environment, P0):
  required work_request_id, invocation_id, authority_decision_id, operation,
  resource, execution_constraints; execution MUST NOT begin before an
  applicable authority decision permits it; execution environment cannot
  self-assert authority.
- E5B (`ExecutionObservation`, execution_environment -> sea_forge, P0):
  required work_request_id, invocation_id, execution_status, observed_effects;
  correlates to the actually-authorized invocation; late/duplicated/cross-wired
  MUST NOT settle against the wrong invocation.
- I5: a governed side effect cannot legally occur before an applicable
  SEA-Forge authority decision.
- event_ownership: AuthorizedInvocation producer = sea_forge exclusively;
  ExecutionObservation producer = execution_environment exclusively.
- identity_rules ENV-I1..I8 (model-hash identity fail-closed, stable
  correlation, causal parents, idempotent durable writes, conformance != truth).
- falsification_classes applicable here: authority_denied,
  authority_escalated, cross_wired_invocation_id, cross_wired_work_request_id,
  duplicate_delivery, replay_after_restart, late_callback,
  out_of_order_event, stale_authority_grant, malformed_envelope,
  missing/stale/wrong domain model, execution_failure/timeout,
  partial_side_effect, forged_parent_reference.
- T04 verifier debt D1–D3 (t04-verifier-independent-confirmation.md) MUST NOT
  be reproduced in T05 code: checked explicitly below (attack A19).

## Production call-path grounding (read before designing attacks)

- `crates/sea-forge-server/src/governed_execution_boundary.rs`: pure-function
  E5A emitter gated on `verdict == Allow && normalized_disposition == Allow`
  over the real `sea_forge_core::types::AuthorityDecision`;
  `InvocationLedger` admits canonical E5A envelopes (producer/model/payload/
  causality validated from wire shape, generationed slots keyed
  (work_request_id, operation, resource)) and adjudicates E5B observations
  (producer, model binding, full payload validation, own-E5A-event causal
  parent, current-generation currency, work-request correlation,
  self-asserted-authority refusal, once-per-observation-identity dedup).
- Real production I5 enforcement (pre-existing, composed not reimplemented):
  `case_dispatch.rs::execute_sandbox` evaluates the real
  `PolicyAuthorityEngine`, commits the decision, and ONLY on
  `Verdict::Allow` creates workspace/artifacts, mints a move-only
  `ActionGrant` via `engine.grant(&decision, &committed_ref, &action, ...)`
  (grant_exact re-checks verdict==Allow, payload-hash binding to the
  committed record, policy-bundle hash), then calls
  `sea_forge_runtime::execute(grant, ...)` which re-verifies
  `authorize_execution` and executable identity before spawning. Non-Allow:
  no workspace, no grant, `execution = None`.
- Boundary functions are exported from `sea-forge-server/src/lib.rs` and are
  consumed by the two convergence_t05 test files; no other src call sites.
  This mirrors the T03/T04 pure-gate precedent at component boundaries.

## Step B — Planned independent attack battery (written before step E)

Cheapest-counterexample mindset: find ONE input through a real call path that
makes unauthorized or cross-wired execution settle as valid governed
execution.

| ID | Falsifier / class | Attack | Expected if contract holds |
|----|-------------------|--------|----------------------------|
| A1 | F1 / authority_denied | REAL engine Deny decision (empty-rules policy) into `emit_authorized_invocation` | `AuthorityNotGranted`, no envelope exists |
| A2 | F1 / authority_escalated | REAL engine Escalate decision into emit | `AuthorityNotGranted`, no envelope |
| A3 | F1 / stale_authority_grant | `engine.grant()` minting from a committed Deny decision; grant-for-action-A used to execute action B via real `sea_forge_runtime::execute`; raw execute without any grant | grant mint refuses ("does not grant this action"); authorize_execution refuses foreign action; no-grant path uncompilable/unreachable (move-only) |
| A4 | F1 / forged conjunct | serde round-trip of a REAL Escalate decision flipping ONLY `verdict`->Allow (disposition stays Escalate), and separately ONLY `normalized_disposition`->Allow on a Deny decision; then emit | both refused (each Allow conjunct load-bearing) |
| A5 | F2 / late_callback | two generations of one slot; valid observation for superseded gen-1 invocation delivered after gen-2 admission | `LateObservation`, never settles |
| A6 | F2 / out_of_order_event | settle gen-2 observation first, then deliver gen-1 observation | gen-1 refused late |
| A7 | F2 / cross_wired_invocation_id + cross_wired_work_request_id | observation citing never-admitted invocation_id; right invocation_id but another request's work_request_id; observation citing the OTHER invocation's E5A event as causal parent | UnknownInvocation / CrossWiredWorkRequest / CausalityMissing |
| A8 | F3 / self-asserted authority in response | settled-shaped observation carrying fabricated `authority_decision_id` string; nested fabricated `authority_decision.decision_id`; redundant-but-matching claim | fabricated forms refused (`SelfAssertedAuthority`); matching form settles but outcome reports LEDGER-recorded decision id, never the claimed one |
| A9 | fresh: forged-E5A-envelope admission (ledger trust placement) | hand-craft a fully conforming E5A envelope stamped `sea-forge` with a fabricated authority_decision_id and junk causal parent; admit directly; then settle an observation against it | characterize: does admission validate decision provenance beyond envelope shape? weigh vs event_ownership (producer privilege) |
| A10 | duplicate_delivery mutating observed_effects under stable identity | ladder: (a) exact redelivery; (b) same event_id + mutated observed_effects; (c) fresh event_id + settled idempotency_key; (d) fresh event_id + NO idempotency_key + mutated effects | (a)-(c) DuplicateDelivery consequence-free; (d) characterize double-settlement-under-new-envelope-identity against the frozen "MUST NOT settle against the WRONG invocation" wording |
| A11 | replay_after_restart | full cycle in ledger 1; drop; fresh ledger 2; re-admit SAME E5A envelope; redeliver SAME settled observation | characterize durability gap vs frozen clauses (ENV-I6 says durable writes idempotent; ledger claims in-process scope) |
| A12 | status vocabulary smuggling | execution_status = "success", "ok", "Completed" (case), "" on observations incl. one whose observed_effects describe failure | InvalidExecutionStatus for every non-vocabulary token |
| A13 | placeholder/fallback identities (T04-D1 flavor) | declared model hash = sha256("agentic_capability_loop") fallback constant, all-zeros, drifted hash — at emit AND at admit AND at settle | PlaceholderIdentity / DomainDrift everywhere |
| A14 | causality gaps GWR->E5A->E5B + forged_parent_reference | emit with blank/"tbd"/"<missing>" governed_work_request_event_id; admit orphan E5A (no caused_by); observation citing work_request_id instead of its E5A event id as parent | all refused (PlaceholderField / CausalityMissing) |
| A15 | wrong-producer stamps both directions | E5A envelope stamped execution-environment/godspeed-agent/unknown/missing source_agent at admit; ExecutionObservation stamped sea-forge at settle | NotAuthoritative / UnknownAgent, fail closed |
| A16 | malformed_envelope / E5B omission+shape battery | omit each of the four required fields; blank them; observed_effects as scalar; stdout_ref numeric; artifact_refs string | OpaquePayload / InvalidOptionalField, never coerced |
| A17 | E5A frozen-field battery at admission | remove each of the six frozen required payload fields from an emitted envelope; execution_constraints as array/string | OpaquePayload per missing field |
| A18 | idempotency-key confusion | distinct-content observation CLAIMING an already-settled idempotency_key | DuplicateDelivery (conservative fail-closed) |
| A19 | explicit T04 D1/D2/D3 reproduction check | D1: is any projection/ref field present-but-unbound in the T05 family? D2: does admit validate WHICH causal parent (junk `caused_by:` accepted)? D3: is `namespace` checked at admit/settle? | characterize each; none may realize a preregistered falsification outcome |
| A20 | grant_id smuggling | emit with `grant_id = Some("fabricated")`; observe whether admit/settle ever consult grant_id as authority | ignored downstream; settlement authority always ledger-recorded |

## Step C — Empirical execution

Out-of-repo harness `/tmp/opencode/t05-verifier` (cargo bin, path deps on
sea-forge-server / sea-forge-authority / sea-forge-core / sea-forge-ledger /
sea-forge-runtime; REAL policies; REAL `PolicyAuthorityEngine`; REAL ledger
commit; REAL `sea_forge_runtime::execute`). Harness target dir relocated to
`/home/sprime01/.t05-verifier-target` after the /tmp tmpfs filled (harness
artifacts only; no repo build state touched). Results:

| ID | Result |
|----|--------|
| A1 | HOLD — real Deny decision ⇒ `AuthorityNotGranted{Deny}`, no envelope exists |
| A2 | HOLD — real Escalate decision ⇒ `AuthorityNotGranted{Escalate}` |
| A3a | HOLD — `engine.grant()` from a committed Deny decision refused ("authority decision does not grant this action") |
| A3b | HOLD — genuine Allow grant + foreign action through REAL runtime execute ⇒ refused ("authority grant does not match execution context") before any spawn; `execute(grant: ActionGrant, ..)` is move-only so the no-grant path is type-unrepresentable. Extra discovery: a grant can only be minted by the SAME engine instance that issued the decision ("authority decision was not issued by this mediator or was replayed") — anti-replay strengthens I5 |
| A4 | HOLD — serde-flipped verdict=Allow/disposition=Escalate AND verdict=Deny/disposition=Allow both refused; each Allow conjunct load-bearing on real decisions |
| A5/A6 | HOLD — superseded-generation observations refused `LateObservation{1→2}` whether delivered late or out-of-order; current generation still settles |
| A7 | HOLD — unknown invocation_id ⇒ UnknownInvocation; right invocation/wrong work_request_id ⇒ CrossWiredWorkRequest; parent = another invocation's E5A event ⇒ CausalityMissing |
| A8 | HOLD — fabricated top-level `authority_decision_id` and nested fabricated `authority_decision.decision_id` both refused (`SelfAssertedAuthority`, recorded-vs-claimed reported); matching claim settles but outcome carries the LEDGER-recorded id only |
| A9 | FINDING (non-falsifying, see adjudication) — a fully conforming E5A envelope stamped `sea-forge` with fabricated decision id and junk parent ADMITS into the ledger, and a matching observation SETTLES reporting the fabricated id. Admission validates shape+producer stamp+model binding+causality presence, not emission provenance. Requires possession of the sea_forge producer role itself (impersonating the authority plane); frozen falsifiers target execution-side self-assertion and cross-wiring, both of which fail; response-forged authority (the literal F3) has NO effect. Candidate debt: admit from emitted structs / bind admission to emission records |
| A10 | HOLD except variant (d): exact redelivery, same-event-id-mutated-payload, and fresh-event-id+settled-idempotency-key are all consequence-free DuplicateDelivery (dedup fires before validation). Variant (d) fresh-event-id+NO-key+mutated effects settles AGAIN against the SAME correct invocation — see adjudication; candidate debt (content-derived dedup at settle) |
| A11 | FINDING (disclosed by builder; non-falsifying) — in-memory ledger: after simulated restart, re-admission of the same E5A envelope opens gen 1 again and the already-settled observation settles again against the SAME invocation with identical recorded facts. ENV-I6 governs durable envelope writes; this registry claims in-process scope only. Candidate debt for the composing durable transport |
| A12 | HOLD — "success"/"ok"/"Completed"/""/"failed" all refused `InvalidExecutionStatus` even when effects describe failure (vocabulary conformance ≠ truth per ENV-I8) |
| A13 | HOLD — fallback constant sha256("agentic_capability_loop"), all-zeros, and drifted hashes refused at emit AND admit AND settle |
| A14 | HOLD — blank/whitespace/"tbd"/"<missing>"/"unknown" causal parents refused at emit; orphan E5A (no caused_by) refused at admit; observation citing the work-request id instead of its E5A event as parent refused at settle |
| A15 | HOLD — E5A stamped execution-environment/godspeed-agent refused NotAuthoritative; mystery stamp and absent source_agent refused UnknownAgent; E5B stamped sea-forge at settle refused NotAuthoritative |
| A16 | HOLD — each required E5B field omission ⇒ OpaquePayload naming it; observed_effects-as-string ⇒ OpaquePayload; numeric stdout_ref ⇒ InvalidOptionalField; nothing coerced |
| A17 | HOLD — each of the six frozen E5A fields removed ⇒ OpaquePayload at admission |
| A18 | HOLD — distinct content claiming an already-settled idempotency key ⇒ DuplicateDelivery (conservative fail-closed) |
| A19 | D1 NOT reproduced: T05 family carries ONLY bound identity (domain_model_hash required, drift-checked vs local at emit/admit/settle); payload key enumeration shows no unbound ref/projection field. D2-echo: admit demands ≥1 caused_by but does not validate WHICH upstream identity (junk parent accepted); end-to-end impact none — settlement requires exact own-E5A-event citation. D3-echo reproduced as consistency nit: `namespace` unchecked at admit/settle exactly like T04's gate; identity unaffected (model-hash bound) |
| A20 | HOLD — smuggled grant_id is carried verbatim in E5A but never consulted downstream; settlement reports ledger-recorded decision regardless |

## Step D — Gates

```
cargo test -p sea-forge-server --test convergence_t05_authorized_execution \
     --test convergence_t05_execution_observation
  → 10 passed + 10 passed, 0 failed (exit 0)
just e2e-gate T05                    → exit 0 (20/20 across both suites)
just e2e-prereg-check                → PASS before AND after (SHA ef5710… intact)
HEAD unchanged throughout: 006daa2540678eaaf821d873266f79914b10c9af
git status identical to session start; only repo file created by this
verification: this evidence document.
```

## Step E — Comparison with builder report

Builder's falsifier→proof map reproduces exactly where it overlaps my battery
(deny/escalate teeth, both-conjuncts, cross-wire, duplicate ladder variants
a–c = their three duplicate cases, wrong-producer both directions,
placeholder/fallback identity, causality gates, positive real-component
cycle). All claimed results verified green (20/20, gate exit 0).

Divergences / items the builder's report does not disclose (none realizes a
preregistered falsification outcome):

1. **Admission trusts wire shape + producer stamp, not emission provenance**
   (A9): a conforming forged E5A admits and can anchor settlements reporting a
   fabricated decision id. Requires impersonating sea_forge's producer role;
   the same trust placement every settled boundary gate in this architecture
   accepts (T03/T04 precedent).
2. **Re-wrap double settlement** (A10d): an observation envelope with fresh
   event id and NO idempotency key settles again against the same correct
   invocation; first fact stands, nothing rewritten, but dedup is
   observation-identity-based rather than content-derived.
3. **caused_by value unvalidated at admission** (A19 D2-echo).
4. **namespace unchecked at this gate** (A19 D3-echo), consistent with T04.

The builder DOES disclose the restart/durability limitation (report's final
scope note), matching my A11 result, and discloses that the emitter/
adjudicator are library surfaces without a non-test caller yet (T03/T04
pattern), which my call-site survey confirms.

T04 verifier debt D1–D3 reproduction check (explicit):
- **D1**: NOT reproduced — every frozen field is required AND identity is
  bound to the locally resolved model at both edges (A17/A19-D1).
- **D2**: echoed weakly (parent PRESENCE demanded, parent VALUE unvalidated at
  admission) but the end-to-end chain stays closed because settlement requires
  the exact own-E5A-event citation; recorded as debt candidate.
- **D3**: echoed (namespace unchecked here too); identity unaffected;
  recorded as debt candidate.

## Adjudication of findings against the frozen contract

- F1 (denied/escalated produce governed side effect): FALSIFIER FAILS. No
  invocation exists for non-Allow decisions (A1/A2); no grant mints from a
  committed Deny (A3a); the real runtime refuses foreign actions under a
  genuine grant (A3b) and requires a move-only grant structurally; production
  `case_dispatch::execute_sandbox` gates workspace/artifacts creation, grant
  minting, and execution behind `Verdict::Allow` with the commit-bound
  `grant_exact` checks.
- F2 (late observation from A settles B): FALSIFIER FAILS. Superseded
  generations refuse forever (A5/A6); unknown/cross-wired/mis-parented
  observations never settle (A7); settlement always names the actually
  authorized invocation (baseline, A8c).
- F3 (authority forged inside execution response gains effect): FALSIFIER
  FAILS. Both forged shapes refused; matching claims redundant; outcome
  reports only the ledger-recorded decision; the outcome type carries no
  capability.
- I5: HOLDS on the production path (real engine → real commit → move-only
  grant → authorize_execution → executable-identity re-check → sandboxed run)
  and on the canonical cross-component statement (no envelope for non-Allow;
  no settlement without an admitted invocation). A9/A10d/A11 do not realize
  it: they either presuppose ownership of the authoritative producer role or
  settle exclusively against the CORRECT invocation with immutable first
  facts; each is recorded below as debt for the composing layers.

## Verdict

**CONFIRM**

Rationale per strict rule:
1. Frozen requirements survive review: E5A/E5B payloads, producer exclusivity,
   authority-precedes-execution, observation-to-authorized-invocation binding,
   and I5 hold under 20 independent attacks including harsher variants than
   the builder's (foreign-action execution under a genuine grant, serde
   conjunct forgery, forged-E5A admission, four-variant duplicate ladder,
   restart replay).
2. Production-path implementation composes REAL machinery: decisions come
   from the real `PolicyAuthorityEngine` under real policies; grants are real
   move-only `ActionGrant`s bound to committed ledger records; side effects
   run through real `sea_forge_runtime::execute` sandbox spawning; the
   boundary module composes these types rather than stubbing them.
3. Gates green: prereg-check PASS before/after (SHA ef5710…), 20/20 tests,
   e2e-gate T05 exit 0.
4. No attack falsified a frozen requirement; no preregistered falsification
   outcome was realized.
5. No known counterexample remains: the four disclosed-or-new strictness/
   trust findings (A9 trust placement, A10d re-wrap, A19 D2/D3 echoes, plus
   the builder-disclosed restart durability note) cannot realize unauthorized
   or cross-wired settlement at this boundary and are recorded as debt
   candidates for `.agents/OBSERVED_DEBT.md`: (a) bind ledger admission to
   emission records or accept emitted structs; (b) content-derived dedup for
   observations; (c) validate causal-parent identity at admission; (d) check
   namespace for family parity; (e) durable settlement reload across restart
   belongs to the composing transport.
6. Nothing at THIS boundary rests on synthetic/mock behavior: engine, ledger
   commit, grants, and execution are the production components; the absence
   of live cross-process transport is standing Delta-0 scope, disclosed by
   the builder, unchanged by T05.

Confirmation condition met: fresh attacks could not cause unauthorized or
cross-wired execution to settle as valid governed execution.
