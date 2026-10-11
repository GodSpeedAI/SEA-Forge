# T07 Independent Adversarial Verification — ProofCompleted → RealityTrace (E7)

Verifier: fresh independent adversarial verifier, not a builder of this work.
Date: 2026-08-25
Status: IN PROGRESS (battery frozen before any builder-report read)

## Freeze state (BEFORE)

- `cd /home/sprime01/projects/sea-rs && just e2e-prereg-check` → **PASS** (exit 0)
- plan hash expected == observed: `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`
- spec sha256sum (same file): `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`

## Frozen contract extracted (attack targets)

- Edge E7 (~line 599): `ProofCompleted`, source `swe_seed`, target `realitytrace`, P0.
  Required payload (7): `work_request_id`, `proof_result_id`, `proof_contract`,
  `proof_status`, `expected_outcome`, `operational_settlement_ref`,
  `domain_model_ref`.
- Invariants: (a) ProofCompleted REFERENCES operational evidence rather than
  rewriting SEA-Forge observation history; (b) proof success alone MUST NOT
  create EvidenceRecorded developmental truth without expected-versus-observed
  binding.
- event_ownership: `ProofCompleted` producer = `swe_seed` EXCLUSIVELY;
  `OperationalSettlement` producer = `sea_forge`.
- Identity rules: ENV-I2 (no placeholder/fallback pseudo-identity, fail
  closed), ENV-I4 (derived envelopes identify causal parents),
  ENV-I5 (no silent rewrite of upstream facts), ENV-I6 (idempotent durable
  writes under stable identity), ENV-I7 (content-addressed refs where
  substrate permits), ENV-I8 (schema conformance ≠ truth).
- Falsification classes applicable here: `forged_parent_reference`,
  `missing_evidence_artifact`, `cross_wired_work_request_id`,
  `duplicate_delivery`, `replay_after_restart`, `wrong_domain_model_hash`,
  `missing_domain_model`.
- Plan T07 falsifiers to reproduce:
  1. proof accepted with forged/missing operational settlement ref ⇒ must be impossible;
  2. altered observed data while referenced settlement says otherwise ⇒ mismatch detected,
     upstream fact not silently replaced.
- Composition claim under attack: emitter binds ONLY to real adjudicated
  settlements from the T06 surface (`operational_settlement.rs`) via actual
  `OperationalSettlementFacts`; `operational_settlement_ref` is claimed
  sha256-content-addressed ⇒ ATTACK THAT.
- Debt that must NOT reproduce: T04 D1/D2/D3 (field validation completeness /
  mandatory both-direction causality / namespace checks); T05 A9/A10d/A11; T06
  minor observations.

## Planned attack battery (frozen BEFORE implementation inspection)

Seams: [E] = emission seam (swe-seed-core public API), [I] = ingestion seam
(sxr-core native gate). Static analysis alone is insufficient for F1/F2; every
attack below is executed empirically via out-of-repo harnesses in
/tmp/opencode/t07-verifier with path deps on both crates unless marked static.

| ID  | Seam | Class / falsifier | Attack |
|-----|------|-------------------|--------|
| A01 | E+I  | F1 forged_parent_reference | Emit + ingest ProofCompleted citing nonexistent settlement ref. Must be impossible/rejected at BOTH seams. |
| A02 | E    | F1 composition claim | Probe emitter public API for ANY path producing a settlement ref NOT derived from real T06 adjudicated `OperationalSettlementFacts` (constructor exposure, From/impl traits, serde round-trip, Debug-derived bypass, test-only constructor reachable from prod API). |
| A03 | I    | F1 missing_evidence_artifact | Ingest with empty/absent `operational_settlement_ref`; also whitespace-only. Must reject. |
| A04 | I    | F1 foreign-cycle | Settlement adjudicated for work_request X, cited by proof claiming work_request Y (cross-wired work_request_id). Must reject. |
| A05 | I    | F2 divergence | Re-submit ProofCompleted whose observed-data-bearing claims diverge from referenced settlement facts (altered outcome fields) while keeping the same settlement ref. Mismatch MUST be detected; upstream facts MUST NOT be silently replaced. |
| A06 | I    | F2 status flip | Same settlement ref, flipped `proof_status` on resubmit. Divergence honesty characterized. |
| A07 | I    | F2 expected_outcome mutation | Mutated `expected_outcome` vs settlement-declared expectation on resubmit. Must be detected or explicitly first-claim-sticks with divergence surfaced — characterize which, and whether silence counts as silent replacement (ENV-I5). |
| A08 | I    | content-address forgery | Take real delivered settlement JSON, mutate bytes, recompute claimed sha256 digest into the ref. Does resolution detect digest↔content mismatch? Fail-closed required. |
| A09 | E+I  | wrong producer | Stamp ProofCompleted with producer ≠ swe_seed at ingestion; also check emitter stamps itself as swe_seed. Impersonation must be rejected (event_ownership). Both directions incl. derivation paths. |
| A10 | I    | ENV-I2 placeholder identity | `domain_model_ref` = placeholder/constant/fallback pseudo-hash ("unknown", all-zeros sha, "default-model"). Must fail closed. |
| A11 | I    | wrong/stale domain model | `domain_model_ref` resolving to a different model than locally-resolved canonical model (drift). Must be detected/rejected. |
| A12 | E+I  | missing-field battery | Remove each of the 7 required fields one-at-a-time; ingest each. All 7 must reject (T04-D1 debt must NOT reproduce here). Also malformed types per field. |
| A13 | I    | schema/namespace tampering | Tampered event_type/namespace/schema-version fields; wrong-type payload smuggled under ProofCompleted name. Namespace checks must hold (T04-D3 debt must NOT reproduce). |
| A14 | I    | ENV-I6 duplicate delivery | Duplicate delivery under STABLE envelope identity → idempotent (no double-write, no corruption). |
| A15 | I    | replay_after_restart | Replay same claims under FRESH event_id → characterize: accepted-as-duplicate-with-same-facts vs divergent-claims handling; honesty of first-claim-sticks semantics probed via A05–A07 results. |
| A16 | I    | causality gap / forged parent | Cite a settlement whose own causal parent refs were stripped/mutated (settlement JSON rewritten to drop parents, digest recomputed consistently). Chain-integrity must break somewhere (ingest resolution or digest mismatch). |
| A17 | I    | resolution boundary abuse | Cite syntactically-valid refs NEVER delivered to sxr (undelivered bundle). Characterize: fail-closed vs silent acceptance. Silent acceptance = falsification. |
| A18 | E    | emitter-settlement coupling | Verify emitter refuses settlement facts lacking adjudication markers (provisional/unadjudicated states from T06 surface) if the surface distinguishes them; verify emitted ref equals the digest of the exact settlement artifact ingested downstream (round-trip digest equality). |

## Execution protocol

1. Inspect production code paths only AFTER battery freeze: emitter
   `SWE_SEED/crates/swe-seed-core/src/federation/proof_completed.rs` (+mod.rs),
   ingestion `sxr/sxr-core/src/proof_ingest.rs` (+lib wiring), T06 surface
   `operational_settlement.rs`.
2. Build harness `/tmp/opencode/t07-verifier` (path deps on swe-seed-core and
   sxr-core); run A01–A18 empirically.
3. Run `cargo test -p swe-seed-core --test convergence_t07_proof_completed` and
   `cargo test -p sxr-core --test convergence_t07_proof_ingestion` (exact
   invocations per repo layout), then `just e2e-gate T07` from sea-rs.
4. ONLY THEN read `.agents/evidence/e2e/T07/t07-report.md`; record divergences.
5. Verdict rule: CONFIRM requires ALL of — frozen requirements survive review;
   production-path implementation both sides; gates green; no attack
   falsifies; no known counterexample remains; nothing resting solely on
   synthetic/mock behavior at THIS boundary.

## Results

(pending)

## Verdict

(pending)

## Freeze state (AFTER)

- `just e2e-prereg-check` → **PASS** (exit 0), SHA unchanged:
  `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`
- Gates: `cargo test -p swe-seed-core --test convergence_t07_proof_completed` →
  **14 passed, exit 0**; `cargo test -p sxr-core --test
  convergence_t07_proof_ingestion` → **12 passed, exit 0**;
  `just e2e-gate T07` (sea-rs) → **exit 0**.

## Execution

Empirical harness: `/tmp/opencode/t07-verifier` (out-of-repo Cargo bin; path
deps on `SWE_SEED/crates/swe-seed-core` and `sxr/sxr-core`; happy path built
EXCLUSIVELY through the real T06 `OperationalSettlementAdjudicator::adjudicate`
→ real `emit_proof_completed_verified` → native `sxr_core::proof_ingest::ingest`).
Final run: **43 attacks, 0 unexpected outcomes** (exit 0). Two harness-side
corrections were made during bring-up (double `sha256:` prefix; divergence
replays initially carried the ORIGINAL idempotency key — a genuine re-emission
carries a fresh key; the same-key case is itself correct ENV-I6 duplicate
semantics, verified separately as A14b).

## Attack table (results)

| ID | Falsifier | Result | Evidence |
|----|-----------|--------|----------|
| A01 | F1 forged settlement ref vs delivered real settlement | REFUSED `SettlementBindingMismatch` | harness A01 |
| A02a | facts paired with foreign settlement envelope (emission) | REFUSED `SettlementNotAdjudicated("facts name … supplied envelope is …")` | harness A02a |
| A02b | in-process forged settlement + fabricated facts (pub types, deliberate adjudicator bypass) | Emission accepts structurally; sxr REFUSES forged parent (`CausalityMissing`) — two-party contract fail-closed | harness A02b; see Divergences D3 |
| A03 | empty/whitespace settlement ref | REFUSED `OpaquePayload{missing:[operational_settlement_ref]}` | harness A03 ×2 |
| A04e/i/i2 | foreign-cycle / cross-wired work_request_id, both seams | REFUSED `CrossWiredWorkRequest` / `SettlementBindingMismatch` | harness A04×3 |
| A05a | altered expected_outcome resubmit (fresh event id+key) | REFUSED `ClaimMismatch{diverged:["expected_outcome"]}`; original record provably intact (redelivery still Duplicate) | harness A05a |
| A06 | flipped proof_status resubmit | REFUSED `ClaimMismatch{diverged:["proof_status"]}` | harness A06 |
| A07 | mutated proof_contract resubmit | REFUSED `ClaimMismatch{diverged:["proof_contract"]}` | harness A07 |
| A05d | CHARACTERIZE: mutated settlement copy re-observed under NEW proof_result_id (self-consistent recomputed ref) | ADMITTED as `First` with tampered observed_effects; original prid1 record intact (`DuplicateDelivery`) AND both digests coexist auditably (`ClaimMismatch{operational_settlement_ref}` when prid2 re-claims original ref) | harness A05d; see Divergences D1 |
| A08 | content-address forgery: mutated bytes vs unchanged cited digest | REFUSED `SettlementBindingMismatch` (recomputed ≠ cited) | harness A08 |
| A09i1/i2 | sea_forge / unknown stamp on ProofCompleted | REFUSED `ProducerForge` | harness A09 |
| A09i3 | restamped settlement delivery | REFUSED `SettlementBindingMismatch("… not the authoritative sea_forge")` | harness A09 |
| A09e/e2 | non-sea_forge settlement at emission; emitter self-stamp | REFUSED at boundary (`forge attempted by 'godspeed_agent'`); emitted envelope stamps swe-seed in source_agent+origin | harness A09 |
| A10i1–i4,e | fallback pseudo-hash `dc144cb…`, all-zeros, "unknown", ref rewriting identity | REFUSED `PlaceholderIdentity`/`MalformedIdentity`/`RefIdentityMismatch`; `VerifiedDomainIdentity::from_resolved(Fallback)` errs | harness A10 |
| A11 | drift vs locally-resolved model | REFUSED `HashDrift` | harness A11 |
| A12(+t) | missing-field battery: each of 7 fields removed; type corruptions | ALL 7 REFUSED together-listed `OpaquePayload`; wrong vocab/scheme/types refused (T04-D1 debt NOT reproduced) | harness A12 rows |
| A13 | schema v2 / wrong event_type / payload+ref namespace tampering / malformed parent id | REFUSED `MalformedEnvelope`/`WrongType`/`WrongNamespace`×2/`CausalityMissing` (T04-D3 NOT reproduced) | harness A13 rows |
| A14a/b | stable-identity redelivery; same-event-id mutated redelivery | Consequence-free `DuplicateDelivery`; identities count unchanged; first record provably not rewritten | harness A14 |
| A15a/b | replay after restart (serde round-trip of IngestState): identical / divergent claims | Identical → consequence-free duplicate; divergent → `ClaimMismatch` after restart | harness A15 |
| A16a | settlement parent stripped from proof chain | REFUSED `CausalityMissing` | harness A16a |
| A16b | CHARACTERIZE: delivered settlement copy with stripped INTERNAL parents, self-consistent digest | ADMITTED as `First` (sxr checks stamp/id/digest/correlation; does not re-derive settlement's internal chain) | harness A16b; see Divergences D1 |
| A17/A17b | never-delivered ghost ref (no bytes); unknown settlement id without bytes | Ghost ref with KNOWN parent id: admitted `First` with honest unresolved markers (`referenced_settlement_sha256=None`, `observed_effects=None`); UNKNOWN settlement id: REFUSED `CausalityMissing` even undelivered | harness A17; builder-disclosed boundary |
| A18 | round-trip digest equality + golden-fixture authenticity | emitted ref == sxr `content_digest(delivered settlement)` byte-equal; golden fixture independently ingests via native gate (`First`) and its cited ref matches recomputed digest | harness A18 |

## Gate commands and exits

1. `just e2e-prereg-check` (before): PASS, exit 0
2. `cargo test -p swe-seed-core --test convergence_t07_proof_completed`: 14 passed, exit 0
3. `cargo test -p sxr-core --test convergence_t07_proof_ingestion`: 12 passed, exit 0
4. `just e2e-gate T07`: exit 0 (regenerates fixture via REAL emitter, runs 14+12)
5. `just e2e-prereg-check` (after): PASS, exit 0, SHA unchanged

## Divergences vs builder claims

Builder report verified accurate on every checkable claim (module inventory,
gate order, refusal taxonomy, fixture flow, scope notes). Undisclosed or
partially disclosed findings:

- **D1 (residual hardening gap, undisclosed)**: sxr pins the settlement EVENT
  ID but not its DIGEST out-of-band. A compromised/lying producer can re-observe
  a MUTATED settlement copy under a NEW `proof_result_id` (A05d: admitted,
  tampered `observed_effects` recorded) or deliver a settlement copy with its
  internal causal parents stripped (A16b: admitted). Not a literal violation:
  tooth 2's scenario (proof claims diverging from referenced evidence) is
  detected exhaustively; first-record integrity is preserved and both digests
  coexist auditably (proven behaviorally); ENV-I5 "not silently rewritten"
  holds verbatim. But admission is silent (no alarm). Smallest future fix:
  cross-claim settlement-digest consistency for one settlement event id, or a
  pinned expected digest in `IngestParams`. Record as debt candidate for T08.
- **D2 (disclosed by builder, independently confirmed)**: undelivered bundle ⇒
  admission proceeds WITHOUT content resolution but with explicit None markers
  (report "Honest scope notes" bullet 2). Unknown settlement ids still refuse
  undelivered (A17b).
- **D3 (partial disclosure)**: emission-side composition is convention +
  consistency enforcement, not cryptographic binding: fully in-process forged
  settlement + hand-built facts (all-pub types) emits structurally IF one
  deliberately bypasses the adjudicator (A02b). Downstream refuses fail-closed.
  Report language "binds ONLY to real adjudicated settlements" is accurate for
  every path that uses the adjudicator, but the API cannot detect an in-process
  bypass; no signatures are verified anywhere on this boundary (signing.rs
  exists, unused here).
- **D4 (disclosed)**: neither surface has binary/CLI callers yet (library-level
  production modules per T03–T06 precedent; builder's final scope note admits
  CLI wiring is future scope). Not a test shim: real modules in src/, exported
  public APIs, exercised end-to-end through real adjudicator→emitter→gate flow.
- Optional non-settlement refs (`artifact_refs`, `trace_root`, `output_ref`,
  `proof_evidence_refs`) accept scheme-shaped non-digest values (ENV-I7
  residual) — disclosed by builder, confirmed.

## Verdict

**CONFIRM**

Basis: frozen requirements survive review; production-path implementation on
both sides (real emitter fed exclusively by real T06 adjudicator output on the
happy path; native sxr gate module, no test shim); freeze PASS before and
after with unchanged SHA; all gates green (14+12 tests, e2e-gate exit 0);
43 empirical attacks executed across both seams with ZERO violations of the
frozen contract text or either plan tooth; golden fixture proven authentic
(digest equality through the real emitter API); no counterexample to any
frozen requirement remains. Residual items D1–D4 are characterized boundary
observations/debt candidates, none falsifying frozen text; D1 is recommended
for T08 consideration because that edge consumes these comparison inputs.

No fixes were implemented by this verifier; no repo files other than this
evidence file were created or modified; pre-existing dirty state untouched.
