# T09 — Fresh Independent Adversarial Verification (E9/I9)

Verifier: independent adversarial verifier, not a builder of T09.
Date: 2026-08-26
Verdict: **PENDING** (finalized at end of this file)
Freeze SHA256 of `.agents/specs/e2e-preregistration.yml`:
`ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`
Pre-check `just e2e-prereg-check`: PASS (exit 0), observed == expected hash.

## Method

1. Read frozen contract FIRST (E9 spec lines 664–694; I9 lines 781–787;
   settlement_semantics.developmental_settlement lines 204–211; event_ownership
   lines 323–373; ENV-I1..I8 lines 281–321; T09 plan block
   `.agents/plans/e2e-plan.yml` lines 1092–1178).
2. Designed the attack battery below from the contract BEFORE opening
   `t09-report.md`. The report is read only AFTER all attacks execute and gates run.
3. Attacks are empirical: external pytest harness under `/tmp/opencode/t09_verify/`
   (outside both repos) invoking `godspeed_nav` modules with PYTHONPATH pointing at
   `/home/sprime01/projects/godspeed_agent`. No test files added inside godspeed_agent.

## Contract surface under attack (as read from frozen spec)

- E9: godspeed_agent -> memory_ledger, P0; five types (SettlementRecorded,
  CapabilityUpdated, RepetitionPlanned, LearningProposalCreated,
  CoherenceBreakDetected); common payload = work_request_id, source_evidence_refs,
  developmental_decision, domain_model_ref.
- E9 invariants: GSA independently evaluates evidence sufficiency; CapabilityUpdated
  requires its own declared capability evidence; ONE successful proof/execution must
  not establish metabolized capability; developmental records retain provenance to
  justifying evidence.
- I9: promotion requires repeated settlement under DECLARED RELEVANT variation; not
  inferable from tool access/explanation/imitation/one proof/one execution.
- event_ownership: SettlementRecorded/CapabilityUpdated/RepetitionPlanned/
  LearningProposalCreated producer=godspeed_agent exclusively. NOTE: CoherenceBreakDetected
  appears in E9.event_types but has NO row in event_ownership.owners (spec gap, not an
  implementation fact); E9 edge source=godspeed_agent still covers it.
- ENV-I1/I2 identity fail-closed + no pseudo-hash; ENV-I4 causal parents; ENV-I6 durable
  idempotency; I10/I15 no silent rewrite/replay safety.
- Plan teeth: (1) single pristine success + metabolized request => refused/not earned;
  (2) identical-envelope redelivery => zero duplicate history; (3) stripped
  source_evidence_refs => quarantine, never persisted as authoritative truth.
- Known confound honored: statuses BELOW metabolized may carry explicitly declared lower
  thresholds (`ACTIVE_MIN_SUCCESSFUL_SETTLEMENTS = 1` is such a declared bar). Attacks
  distinguish gaming from lawful declared thresholds.

## Planned attack battery (written BEFORE reading t09-report.md)

Prime falsifier candidates identified from static reading, to be settled empirically:

- F1: source_evidence_refs citing nonexistent evidence ids may pass shape validation
  (no existence resolution wired by default at runtime surface — runtime passes
  `local_model_sha256` but NOT `known_parent_ids`).
- F2: N distinct SettlementRecorded envelopes sharing one settlement_ref but distinct
  event_ids/variations might accumulate as independent repetitions (trust placement).
- F3: exactly-two-successful-settlement metabolization attempt (machinery math predicts
  refusal: reuse_frequency<3 and settlement_events<3 force metabolization_score <= 4/6).

| ID | Falsifier / attack | Expected if E9/I9 hold |
|----|--------------------|------------------------|
| A01 | T1 runtime surface: ONE pristine successful SettlementRecorded then `request_capability_promotion(metabolized)` | Refused by name; zero CapabilityUpdated in ledger |
| A02 | T1 direct gate: hand-built CapabilityUpdated over single-settlement state via ingest/persist | Refused (capability_requires_repeated_settlement / undeclared_variation); nothing persisted |
| A03 | T1 publisher-equivalent paths: emit_developmental_event("CapabilityUpdated") directly with metabolized target on single-cycle state; also every non-CapabilityUpdated type rejected as promotion carrier | Refused; promotions count stays 0 |
| A04 | T1 fresh composition: exactly TWO successful settlements (score>=0.7), two DISTINCT pre-declared variations, request metabolized | Refused (machinery needs >=3 settlements/reuse>=3 => score<0.75). If admitted: falsifier |
| A05 | T2 exact-same-envelope redelivery through persist gate | status=duplicate; ledger file sha256 byte-identical before/after; counts unchanged |
| A06 | T2 redelivery after simulated restart (fresh state rebuilt via load_developmental_memory_state) | duplicate; still exactly one ledger row |
| A07 | T2 replay-with-mutated developmental_decision under SAME event_id | envelope_identity_conflict refused BY NAME; first record stands; ledger unchanged |
| A08 | T3 stripped refs battery: missing key / None / [] / "" / "   " / ["  "] / placeholder tokens | Every variant refused (incomplete_developmental_event or provenance_stripped) BEFORE any write; ledger byte-count unchanged |
| A09 | T3 refs to nonexistent evidence ids ("ev-forged-...") at direct gate AND runtime surface | Per task expectation: quarantine/refusal. If accepted: evaluate against F1 (admission trust placement) — record precisely which surface accepts |
| A10 | Causality battery: empty caused_by chain; malformed parent id; forged parent vs pinned known_parent_ids | causality_missing / malformed / forged_parent_reference refusals |
| A11 | Wrong producer stamps for ALL FIVE types (sea_forge, swe_seed, realitytrace, memory_ledger, context_kernel, execution_environment) + provenance.origin contradiction | producer_forge refused each time; nothing persisted |
| A12 | Placeholder/fallback identity: domain_model_hash = fallback pseudo-hash dc144cbd…, all-zero hash, short/garbage hash; placeholder work_request_id | fallback_domain_hash_forbidden / malformed_domain_identity / degenerate_field_value |
| A13 | Namespace tampering: payload.namespace wrong; domain_model_ref.namespace wrong; model_ref.model_hash != declared | namespace_tampering / ref_identity_mismatch |
| A14 | Four-field missing battery: drop each common field one-at-a-time; degenerate shapes (work_request_id=123 int, decision={} , decision=[], model_ref non-object) | incomplete_developmental_event reporting ALL missing together; degenerate_field_value / degenerate_developmental_decision refusals |
| A15 | Restart reopen: rebuild state via replay-through-gate; prior settlements/promotions survive; post-reopen redelivery still duplicate | Replay restores history; dedup intact |
| A16 | Corrupted ledger line out-of-band: garbage JSON line + tampered checksum-valid-looking line injected into developmental_events.jsonl; reopen | Corrupt rows quarantined by store, never trusted into history; replay continues; gate functional (fail-closed) |
| A17 | CoherenceBreakDetected / LearningProposalCreated / RepetitionPlanned carrying promotion-shaped payloads | Admitted (if well-formed) ONLY as their own types; promotions/settlements counts unchanged; cannot later serve as variation evidence for promotion |
| A18 | Metabolization-score manipulation: settlement_score=1.5 / "high" / true refused; injected payload field "metabolization_score" must NOT influence the gate's own computation; missing score defaults 0.0 and fails stability | Degenerate refusals; gate computes its own score; no shortcut to metabolized |
| A19 | Cross-capability citation: settlements recorded under cap-A; promote cap-B citing cap-A's settlement refs | unresolved_capability_evidence refused |
| A20 | Post-hoc variation: record two successful settlements WITHOUT declared variation_context; promotion attempt asserts variations inline in the request | undeclared_variation refused (inline declarations ignored) |
| A21 | Same-settlement_ref multiplied under distinct event_ids + three distinct declared variations, scores 0.9 (F2): does triple-delivery of one logical settlement reach metabolized? | Record empirical result; judge within disclosed scope limit ("variation relevance enforced structurally") vs admission-trust falsification |

## Execution log

- [x] Harness built at `/tmp/opencode/t09_verify/test_t09_adversarial.py` (outside both repos;
      PYTHONPATH=/home/sprime01/projects/godspeed_agent; real LedgerStore JSONL files, no mocks).
- [x] A01..A21 executed: **74 passed, 0 failed** (21 planned attacks + parametrized expansions
      + 3 lawful-path controls C01–C03).
- [x] Builder suite: `uv run --extra dev python -m pytest tests/test_convergence_t09_developmental_memory.py -q`
      (from /home/sprime01/projects/godspeed_agent) → **94 passed**, exit 0.
- [x] Gate: `just e2e-gate T09` (from sea-rs) → exit 0 (94 green).
- [x] Post-freeze prereg check: PASS (exit 0); SHA unchanged
      `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`.

## Attack results (empirical)

| ID | Result | Evidence |
|----|--------|----------|
| A01 | REFUSED `capability_requires_repeated_settlement` (real `Capability.from_settlements` machinery); 0 CapabilityUpdated rows | harness test_a01 |
| A02 | REFUSED same code at direct gate; promotion_count stays 0 | test_a02 |
| A03 | RepetitionPlanned/LearningProposalCreated/CoherenceBreakDetected carrying promotion payloads: persisted as their own types ONLY; promotions stay 0 | test_a03 |
| A04 | Exactly-2-successful-settlements + 2 distinct declared variations ⇒ metabolized REFUSED `i9_not_earned` (machinery needs ≥3 settlements/reuse ≥3 ⇒ score ≤4/6 <0.75) | test_a04 |
| A05 | Exact redelivery ×2: `duplicate`; ledger file sha256 byte-identical; 1 row | test_a05 |
| A06 | Redelivery after restart-replay: `duplicate`; still 1 row | test_a06 |
| A07 | Mutated decision, same event_id ⇒ REFUSED BY NAME `envelope_identity_conflict`; ledger byte-identical | test_a07 |
| A08 | missing/None/[]/""/"   "/["  ","\t"]/["unknown"]: refused pre-write (`provenance_stripped` at projector, `incomplete_developmental_event` at gate); ledger unchanged | test_a08_a09 |
| A09 | **FABRICATED refs to nonexistent evidence ids ADMITTED at BOTH direct-gate (`first`) and runtime surface (`first`)** — F1 confirmed; see Divergences D1 | test_a08_a09[nonexistent_ids] |
| A10 | Empty chain ⇒ `causality_missing`; malformed parent ⇒ `malformed_parent_reference`; forged parent vs pinned ids ⇒ `forged_parent_reference` (pinning works WHEN supplied) | test_a10 |
| A11 | All five types × six foreign stamps (sea_forge/swe_seed/realitytrace/memory_ledger/context_kernel/execution_environment) ⇒ `producer_forge`; origin contradiction ⇒ `producer_forge` | test_a11, test_a11b |
| A12 | Fallback pseudo-hash dc144cbd…, all-zero, short, non-hex, uppercase digests refused at projector; placeholder work_request_ids ⇒ `degenerate_field_value` | test_a12, test_a12b |
| A13 | payload.namespace / domain_model_ref.namespace tampering ⇒ `namespace_tampering`; model_ref hash mismatch ⇒ `ref_identity_mismatch` | test_a13 |
| A14 | Each common field dropped ⇒ `incomplete_developmental_event` (incl. domain_model_ref); int work_request_id / {} / [] decision / non-object model_ref ⇒ degenerate refusals | test_a14, test_a14b |
| A15 | Replay-through-gate restores settlements/promotions; post-reopen dedup intact | test_a15 |
| A16 | Garbage line + checksum-tampered row injected out-of-band ⇒ quarantined to `developmental_events.corrupt.jsonl`, excluded from history, gate stays functional (fail-closed) | test_a16 |
| A17 | CoherenceBreak/LearningProposal ids cited as variation_evidence ⇒ `unresolved_capability_evidence`; they never become capability state | test_a17 |
| A18 | settlement_score 1.5/-0.2/"high"/true refused; injected `metabolization_score` field carries NO power (gate computes its own; single settlement still refused) | test_a18, test_a18b |
| A19 | cap-A settlements promoting cap-B ⇒ `unresolved_capability_evidence` | test_a19 |
| A20 | 3 successful settlements WITHOUT declared contexts + inline "declared_variations" in request ⇒ `undeclared_variation` (post-hoc gaming dead) | test_a20 |
| A21 | One logical settlement_ref triple-delivered under 3 distinct variations, scores 0.95 ⇒ metabolized REFUSED `capability_requires_repeated_settlement`: cited-ref resolution maps each cited string to exactly ONE first-match settlement (seen-key dedup), so ref-multiplication cannot inflate repetition count | test_a21 |
| C01–C03 | Controls: lawful `active`(1 success), `repeated`(2), `metabolized`(≥3 varied high-scoring) paths remain OPEN — the I9 bar is a threshold, not a refuse-all wall | tests_c01..c03 |

## Builder-report comparison (read AFTER all attacks)

Coverage agreement (independently reproduced): single-pristine-cycle refusal incl. the exact
machinery error code; eight stripped-ref variants never persisted; byte-identical idempotent
redelivery; `envelope_identity_conflict`; cross-capability refusal; post-hoc inline variation
ignored; active(1)/repeated(2)/metabolized-bar distinctions preserved; producer exclusivity;
identity gates; restart replay; corrupt-row quarantine; fixture produced by the REAL emitter
(regeneration through the public projector API — noted here explicitly per verification rules;
no fixture was regenerated during this verification).

### Divergences / gaps found by this verifier

- **D1 (substantive, disclosed-debt class): fabricated-but-well-formed evidence references are
  admissible by default.** Neither `source_evidence_refs` nor `caused_by` parent ids are resolved
  for existence on any default surface; `runtime.emit_developmental_event` passes
  `local_model_sha256` but NEVER `known_parent_ids`. A durable developmental record citing
  nonexistent evidence ids was persisted at BOTH the direct gate and the runtime surface (A09).
  The builder's tooth-3 variant list stops at stripped/degenerate shapes and does not test
  fabricated ids; its scope notes disclose the equivalent trust placement ("settlement facts … do
  not re-derive from upstream evidence bytes"; filesystem forgery out of contract surface).
  Weighing: E9 requires provenance be RETAINED (it is, verbatim); ENV-I7 resolution is SHOULD-level
  ("when the underlying substrate permits"); the pinning mechanism exists and refuses forged
  parents when supplied (A10); identical admission-trust placement was accepted at T05-A9/T06/
  T08-V20. Judged NOT a MUST-level falsification of E9/I9 — logged as the surviving edge of the
  known admission-trust debt theme. Recommended follow-up: wire `known_parent_ids` from upstream
  evidence-ledger ids at the runtime seam, or resolve `source_evidence_refs` against recorded
  EvidenceRecorded ids.
- **D2 (spec nuance, not implementation):** `CoherenceBreakDetected` appears in E9.event_types but
  has NO row in event_ownership.owners. The implementation nonetheless enforces godspeed_agent
  exclusivity for it (A11), so behavior exceeds spec bookkeeping. Spec-side bookkeeping gap only.
- **D3 (cosmetic):** builder report says promotion attempts hit "EVERY reachable surface"
  including "publisher-path ingestion"; this verifier exercised runtime methods, direct gate
  calls, non-carrier types, and raw-row bypass quarantine (A16) but did not separately drive the
  external `agentic_capability_loop` NATS publisher drain path (no broker in environment; matches
  builder's own disclosed live-NATS limitation).
- **D4 (cosmetic):** "94 tests" = 45 test functions expanded by parametrization to 94 collected.

## Verdict

**CONFIRM**

Basis (step-F checklist):
1. Frozen requirements survive review: E9 four-field/payload/invariants and I9 repeated-
   settlement-under-declared-variation hold under 24 adversarial compositions incl. every plan
   tooth; refusals occur BEFORE any durable write in all cases.
2. Production-path implementation: gates compose the REAL existing lifecycle machinery
   (`Capability.from_settlements`, `compute_metabolization`, `infer_capability_lifecycle`);
   persistence over the real flock-guarded JSONL `LedgerStore`; no shims at this boundary.
3. Gates green: builder suite 94 passed (exit 0); `just e2e-gate T09` exit 0; prereg PASS
   before AND after with identical SHA `ef5710893…aa879f`.
4. No attack falsified E9/I9: metabolized is unreachable from one cycle (A01–A03), two cycles
   (A04), score/field manipulation (A18), post-hoc declaration (A20), cross-capability citation
   (A19), or settlement-ref multiplication (A21); idempotency and mutation-refusal hold (A05–A07);
   producer/identity/quarantine/corruption batteries hold (A11–A16).
5. Known confound honored lawfully: below-metabolized thresholds (active=1) are explicitly
   declared constants and lawful paths remain open (C01–C03).
6. Residual counterexample-class risk is confined to D1, which is SHOULD-tier (ENV-I7), inside
   the explicitly disclosed admission-trust debt theme, and mechanically mitigated whenever
   out-of-band pinning is supplied; it does not defeat any MUST-level frozen requirement at this
   boundary. Recorded for the debt ledger rather than hidden.

Verifier constraints honored: only this file created inside the repos; harness lives in
/tmp/opencode/t09_verify/; no commits, no reverts, no fixture regeneration, prereg untouched.
