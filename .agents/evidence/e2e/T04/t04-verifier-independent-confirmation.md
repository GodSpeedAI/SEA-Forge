# T04 Independent Adversarial Verifier — Confirmation Record

Verifier: fresh independent adversarial verifier (not a builder of T04).
Date: 2026-08-25
Scope: frozen edge E4 (governed_work_submission), swe_seed -> sea_forge, P0.

## Phase 0 — Freeze check

- Command: `just e2e-prereg-check` from /home/sprime01/projects/sea-rs
- BEFORE verifier work: PASS — expected == observed ==
  `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`
- AFTER verifier work: PASS — same SHA intact.
- Attack battery below was designed and written to this file BEFORE any
  production code inspection and BEFORE reading `.agents/evidence/e2e/T04/t04-report.md`.

## Attack battery (designed pre-inspection; executed empirically)

Harnesses OUTSIDE the repos, path-dep on real crates, no mocks:
- `/tmp/opencode/t04-verifier` (swe-seed-core): drives REAL
  `accept_work_requested` + `build_governed_work_request`; emits an
  independently produced request+packet pair to
  `/tmp/opencode/t04-verifier/out/real_pair.json`.
- `/tmp/opencode/t04-verifier-cons` (sea-forge-server): loads that REAL pair
  and drives every consumer attack through REAL
  `accept_governed_work_request`.

Result legend: REFUSED = attack failed to falsify (requirement holds);
EMITTED/ACCEPTED(!) = attack landed; OBSERVED = neutral observation.

| ID | Requirement attacked | Result | Where preserved |
|----|----------------------|--------|-----------------|
| BASE | Real chain E1->E3->E4 through real emitter, then real gate | EMITTED + ACCEPTED | both suites' happy paths |
| A1-producer | Opaque command-only cannot be emitted | PARTIAL: argv-shaped TEXT accepted as `intent` (non-empty check only); full semantic contract still bound. Not "command-only" in frozen sense | producer refuses blank intent/actor/criterion (IncompleteIntent); consumer A1 |
| A1-consumer | Plan falsifier 1: argv/command-only payload settles? | REFUSED — OpaqueCommand listing all missing semantic fields | sea-rs suite t04_opaque_command_only_request_cannot_settle |
| A2-naive | Plan falsifier 2: foreign-cycle packet attached | REFUSED — CrossWiredContextPacket | both suites |
| A2-aggressive (fresh) | Foreign packet with context_packet_ref AND caused_by rewritten consistently to it | REFUSED — CrossWiredContextPacket (work_request_id equality is load-bearing) | this file + harness output |
| A2-producer | Foreign packet offered to emitter | REFUSED pre-emission | SEED t04_cross_wired..._before_emission |
| A3-local-drift | Plan falsifier 3: SEA-Forge resolves different valid model locally | REFUSED — DomainDrift | FORGE t04_different_domain_model_is_rejected |
| A3-packet-drift | Packet declares drifted model | REFUSED — DomainDrift | FORGE t04_drift_on_the_referenced_context_packet... |
| A3-producer | Drifted E1/E3 parents at emitter | REFUSED at boundary pre-emission | SEED t04_different_domain_model..._before_emission |
| A3prime (fresh) | `domain_model_ref.model_hash` names another model while bound hash stays == local | ACCEPTED(!) — gate never reads domain_model_ref | FINDING D1 below |
| A4 | Wrong producer stamp reaching sea-rs gate (I3): sea-forge / godspeed-agent / context-kernel / realitytrace / mystery-agent | REFUSED — NotAuthoritative / UnknownAgent for all five | FORGE t04_non_swe_seed_producer_stamp_is_rejected |
| A4-packet-selfprod | Referenced packet re-stamped swe-seed | REFUSED — NotAuthoritative | same |
| A5-exact-collapse (fresh) | proof_contract := settlement_criteria array (I6) | REFUSED — CollapsedObligations | FORGE t04_proof_contract_may_not_collapse... |
| A5-empty-criterion | proof_contract present without obligation | REFUSED — OpaqueCommand | same |
| A6-empty-chain (fresh) | provenance.chain emptied | REFUSED — CausalityMissing | FORGE t04_lost_causal_lineage_is_rejected |
| A6-drop-E1-parent (fresh) | WorkRequested caused_by stripped, packet citation kept | ACCEPTED(!) — gate enforces packet lineage only | FINDING D2 below |
| A6-forged-parent (fresh) | Well-formed nonexistent parent id appended | OBSERVED accepted — gate is transport-free; no store exists to resolve parent existence against; recorded parents must be well-formed ids (ENV-I4 shape rule holds) | this file |
| A7 (fresh) | Placeholder identity through sea-rs side: fallback constant EVEN WHEN local misresolves to it; all-zero; deadbeef | REFUSED — PlaceholderIdentity for all three | FORGE t04_placeholder_pseudo_identity_is_rejected |
| A8 (fresh) | actor {} / actor_id "   " / actor null / intent "   " / affordance "" / ref "  " / criteria ["  ",""] | REFUSED — OpaqueCommand for all seven | FORGE t04_empty_semantic_fields_are_opaque_commands_too |
| A9-wr-rewrite (fresh) | payload.work_request_id rewritten post-hoc | REFUSED — CrossWiredContextPacket (no silent reconcile, ENV-I3/I5) | harness output |
| A9-producer-mismatch | Contract vs WorkRequested envelope correlation disagreement | REFUSED — ContractEnvelopeMismatch | SEED t04_contract_envelope_mismatch... |
| A10 (fresh) | Duplicate delivery / re-emission identity (ENV-I6) | OBSERVED: gate is pure — no durable write inside boundary; identical input twice -> identical result; emitter idempotency_key stable across re-emission, event_ids differ by design | this file |
| A11 | Required-field omission battery (each of the eight) | 7 of 8 REFUSED (OpaqueCommand); omit `domain_model_ref` ACCEPTED(!) | FINDING D1 below |
| A12-wrong-namespace (fresh) | payload namespace swapped | OBSERVED accepted — gate binds identity via domain_model_hash; namespace not separately checked at this gate | FINDING D3 below |
| A12-schema-v2 / type-swap | schema_version v2 / event_type swap | REFUSED — MalformedEnvelope / WrongEventType | harness output |
| A13-producer (fresh) | Placeholder digest unlocks VerifiedDomainIdentity? | REFUSED — type-level impossible; no code path manufactures identity | SEED t04_fallback_pseudo_identity... |

## Gate commands and exit statuses (verifier-executed)

```
cargo test -p swe-seed-core --test convergence_t04_governed_submission   # 10 passed, exit 0
cargo test -p sea-forge-server --test convergence_t04_governed_ingress   # 12 passed, exit 0
just e2e-gate T04                                                        # runs both suites, exit 0
just e2e-prereg-check                                                    # PASS before AND after
```

Full gate log preserved: `/tmp/opencode/t04-verifier/gate-t04-full.log`.
Note: running the SWE suite regenerates the cross-repo fixture from real
emitter output (builder-designed behavior); no other repo state changed.
No commits; no cargo clean; SWE_SEED spec-0020 WIP untouched.

## Production-surface judgment (per T03 precedent)

Both surfaces are production `src/` modules exported at their crate facades
(`swe_seed_core::federation::build_governed_work_request`,
`sea_forge_server::governed_work_ingress::accept_governed_work_request`),
pure/transport-free by design. Neither has a non-test caller yet — identical
to the settled T03 pattern (`accept_work_requested`). The fixture is genuine
emitter output regenerated each run, not a hand-written shim. No synthetic or
mock behavior exists AT THIS BOUNDARY. Transport wiring into NDJSON/socket is
disclosed open debt (Delta-0 stands) and out of T04 scope.

## Divergences vs builder claims (step E)

Builder's falsifier->proof map reproduced exactly where it overlaps my
independent battery. My harsher variants (aggressive causality rewrite,
fallback-as-local, five forger stamps, omission battery) all hold. Three
items the builder's report does not disclose:

- **D1 (strictness gap, projection field):** the sea-rs gate neither requires
  `payload.domain_model_ref` nor checks it equals the bound
  `payload.domain_model_hash` (A11/A3prime ACCEPTED). Judgment: NOT an E4
  falsification. The spec's identity_rules define canonical identity solely
  via model_hash (ENV-I1/I2); the gate enforces that channel fail-closed on
  BOTH envelopes against an out-of-band locally resolved digest; the real
  emitter guarantees ref==hash==parents' hashes by construction (divergence
  unrepresentable via production emission); `GovernedWorkIntent` does not
  consume the ref field, so no downstream decision at this boundary can be
  poisoned by a corrupted label. Candidate debt: add
  ref-presence/ref==hash check for literal frozen-field conformance.
- **D2 (hardening gap):** gate demands citation of the ContextPacketCreated
  parent but accepts stripping the WorkRequested parent. Producer always
  records both (substrate-enforced). No work/context/domain cross-wire is
  enabled by this; candidate debt for the transport that carries E4.
- **D3 (consistency nit):** SWE_SEED consume.rs rejects wrong `namespace`;
  the sibling sea-rs gate does not check it. Identity binding unaffected
  (hash-based). Candidate debt.

Also observed (neutral): argv-shaped intent TEXT passes the emitter's
non-empty check (A1-producer). Mechanical validation of intent semantics is
out of reach for any boundary; the frozen falsifier targets requests lacking
the semantic CONTRACT, which are refused on both sides.

## Verdict

**CONFIRM**

Rationale per strict rule:
1. Frozen requirements survive review: all three E4 invariants hold
   empirically under harsher-than-builder attacks; ENV-I1/I2/I3/I4/I5
   relevant portions, event_ownership (E4 = swe_seed exclusively, enforced at
   the consumer for both the request and its referenced packet), I1/I3/I6
   hold at this boundary.
2. Production-path implementation exists on both sides (real src modules,
   facade exports, real logic, real cross-repo fixture).
3. Gates green: prereg-check PASS before/after; 10/10 + 12/12; e2e-gate T04
   exit 0.
4. Attacks fail to falsify: no attack achieved acceptance of an opaque-only
   request, a cross-wired packet (even fully rewritten), a drifted/placeholder
   domain identity, a forged stamp, collapsed obligations, or severed packet
   causality.
5. No known counterexample remains: D1–D3 are conformance-strictness gaps on
   projection/label surfaces that cannot alter which work, whose context, or
   which model governs at this boundary; none realizes a preregistered
   falsification outcome. They are recorded here as debt candidates and MUST
   be carried into `.agents/OBSERVED_DEBT.md` by an authorized writer.
6. Nothing at this boundary rests on synthetic/mock/test-only behavior
   (T03-precedent judgment above).

Confirmation condition met: fresh independent verifier could not cross-wire
work, context, or domain identity and still obtain an accepted governed
request.
