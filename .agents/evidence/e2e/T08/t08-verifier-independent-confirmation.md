# T08 Independent Adversarial Verification — Evidence File

- Role: fresh independent adversarial verifier (did not build T08).
- Contract frozen at: `.agents/specs/e2e-preregistration.yml` (E8 ~L629, I8 ~L774, I10 ~L785, event_ownership ~L323) + `.agents/plans/e2e-plan.yml` `T08:` block.
- Freeze check BEFORE battery design: `just e2e-prereg-check` → PASS, exit 0. Plan SHA256 = `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`; spec SHA256 = `0c57e54310d08b3a01d3b001ac9d8baf859a930e97e8b04267d9eba797bd0e39`.
- Builder report `t08-report.md` deliberately NOT yet read at time of this writing.

## Pre-committed attack battery (designed from contract only)

Falsifiers to reproduce per plan block: (1) missing expected/observed accepted ⇒ impossible; (2) forged parent/proof/settlement refs accepted ⇒ impossible; (3) receipt alone creates SettlementRecorded/CapabilityUpdated ⇒ impossible.

| ID | Falsifier / attack |
|-----|-------------------|
| V01 | F1a: Rust emitter accepts EvidenceRecorded payload missing `expected_outcome`. |
| V02 | F1b: Rust emitter accepts payload missing `observed_outcome` (incl. empty string). |
| V03 | F1c: Python GSA gate accepts EvidenceRecorded missing/empty `expected_outcome` or `observed_outcome` (None key-missing, "", whitespace). |
| V04 | F2a: Forged `proof_result_ref` pointing at nonexistent artifact accepted by Python gate. |
| V05 | F2b: Refs re-pointed at real-but-different artifacts (format-valid, wrong content) — does any out-of-band binding detect? Attacks disclosed "structural-only" pinning honesty boundary. |
| V06 | F2c: Digest-pinned refs mutated (digest suffix mismatch) — emitter and/or gate validation behavior. |
| V07 | F2d: Undelivered ghost refs (T07-D2/D4 marker theme) — consumer trusting refs whose upstream bytes were never delivered. |
| V08 | F3a: Ingest one completely valid EvidenceRecorded via real runtime method; assert NO SettlementRecorded/CapabilityUpdated/promotion anywhere reachable. |
| V09 | F3b: Enumerate every promotion-capable surface reachable from provisional record + `ingest_canonical_evidence_recorded`; confirm lawful path only via explicit record_settlement; no shortcut added by ingestion. |
| V10 | Producer forgery A: non-realitytrace producer restamping/deriving EvidenceRecorded (ownership exclusivity). |
| V11 | Producer forgery B: realitytrace surface emitting SettlementRecorded/CapabilityUpdated (reverse direction). |
| V12 | Identity placeholder mirrored-set gap: placeholder variants ("", "unknown", "TODO", "placeholder", "<none>", "N/A", "null", "*", "?") probed against Python set vs Rust set for divergence. |
| V13 | Cross-wired work_request_id: evidence work_request_id ≠ proof/settlement ref's work_request_id. |
| V14 | Duplicate/replay under stable identity: identical event resubmitted; check idempotency without promotion/duplication. |
| V15 | Replay with fresh event_id + mutated difference/observed under same identity refs — silently accepted as independent evidence? |
| V16 | I10 mutation probe: divergent resubmission attempting silent rewrite of upstream expected/observed inside GSA state. |
| V17 | Namespace/schema tampering: wrong casing event_type, unknown extra fields, type-confused payload values (dict/int for outcome strings), schema drift. |
| V18 | Difference fabrication: `difference` not derived from expected/observed (claims "match" while expected≠observed) — is any derivational/cryptographic binding actually enforced? |
| V19 | Fixture authenticity: regenerate committed fixture through REAL public emitter API and compare bytes/semantics. |
| V20 | Trust placement (T05-A9/T06 theme): admission based on envelope self-claims (e.g., producer field) without out-of-band verification. |

Debt-theme attacks explicitly included: T07-D1 digest pinning at composition seams (V05/V06/V07), T05-A9/T06 trust placement (V20), T07-D2/D4 undelivered-ref markers (V05/V07).

## Execution log

(appended below as attacks execute)

## Verdict

(pending)

---

## Execution log (empirical, after battery pre-commitment)

### Harnesses (outside repos; no repo files modified)

- Rust: `/tmp/opencode/t08-verifier-rs` (path dep on sxr-core) — attacks RV1–RV7.
- Python: `/tmp/opencode/t08-verifier/test_t08_attacks.py`, run with
  `PYTHONPATH=/home/sprime01/projects/godspeed_agent` and the repo venv
  interpreter — 60 tests, all passing. No test file was added inside any repo.

### Attack table

| ID | Falsifier | Result | Evidence location |
| --- | --- | --- | --- |
| V01/RV1 | Emitter accepts missing expected_outcome | REFUSED (`MissingBindingInput{expected_outcome}`); also refused for Null | harness RV1; sxr test `t08_tooth1_missing_or_null_expected_binding_refuses_emission` |
| V02/RV2a-b | Emitter accepts missing/null observed_outcome | REFUSED both (`MissingBindingInput{observed_outcome}`) | harness RV2a/RV2b |
| V02c/RV2c | Degenerate observed="" | EMITTED at emitter seam — but Python gate refuses blank strings (`difference_binding_mismatch` path); cannot bind end-to-end. Recorded as minor emitter-side hardening note, not a contract breach (value present; provisional only) | harness RV2c + V03b variants |
| V03a/b | Python gate accepts missing expected/observed (key-missing, None, "", "   ") | ALL REFUSED (`incomplete_evidence_recorded`; whitespace via `difference_binding_mismatch`) through BOTH pure gate AND runtime method; zero ledger writes | pytest v03a/v03b/v03c |
| V03d | Empty-list observed outcome | ADMITTED (present value; verdict=divergent; inert provisional). Undocumented in builder report — minor hardening gap, no frozen requirement broken | pytest v03d |
| V04a/b | Well-formed refs citing unknown artifacts | ACCEPTED structurally (provisional-only). Matches builder's disclosed scope note 2 exactly | pytest v04a/v04b |
| V05 | Refs swapped between real artifacts under stable identity | DUPLICATE (consequence-free) — identity dedup fires first; fresh-key variant hits claim stability (V15) | pytest v05 |
| V06 | Malformed digest forms in refs ("sha256:XYZ", bare hex, uppercase, short) | ALL REFUSED (`invalid_ref`); emitter side: `InvalidRef`; cited≠resolved settlement → `SettlementRefMismatch` | pytest v06; harness RV3a/RV3b |
| V07 | Ghost refs to bytes delivered nowhere | ACCEPTED structurally (inert provisional). Disclosed by builder (scope note 2) | pytest v07 |
| V07b | Forged causal parents WITHOUT pinning | ACCEPTED when deployment supplies no `known_parent_ids`. Disclosed by builder (scope note 2) | pytest v07b |
| V07c | Forged causal parents WITH out-of-band pinning | REFUSED BY NAME (`forged_parent_reference`) | pytest v07c |
| V08/F3a | Valid receipt promotes anything | NO: real NavigationRuntime + real store → exactly one row in `canonical_execution_evidence`, ZERO rows in settlements/capabilities/trajectories/repetition_schedules; classification=provisional | pytest v08 |
| V09/F3b | Promotion surface reachable from record/ingestion | NO: `ProvisionalEvidence` callable surface = {to_ledger_record} only; runtime ingestion source never references record_settlement/_update_capability_evidence/_schedule_repetition | pytest v09 + static inspection of runtime.py:494-559 |
| V09b | Evidence alone drives explicit settlement to metabolization | Explicit record_settlement with EMPTY inputs succeeds but yields status=failed score=0.0 lifecycle=latent metabolization=0.0 — I9 intact; ingestion adds no shortcut | pytest v09b |
| V10 | Producer forgery (7 impostor stamps + origin contradiction) | ALL REFUSED (`producer_forge`); "REALITYTRACE" case-variant refused | pytest v10/v10b |
| V11 | realitytrace emits SettlementRecorded/CapabilityUpdated | IMPOSSIBLE: zero occurrences of either type in sxr-core/src/**/*.rs; emitter stamp is a constant (RV4) | pytest v11; harness RV4 |
| V12 | Mirrored placeholder-set divergence | NONE: Rust set == Python set == {placeholder,unknown,none,<missing>,tbd}; core variants refused on both id fields on both sides | harness RV5 + pytest v12 |
| V12b | Extended variants (todo,<none>,n/a,null,*,?) | Pass the placeholder CHECK on both sides equally, but dominated downstream: work_request_id → cross_wired refusal; affordance_id → executable-affordance existence check fails closed. No Rust/Python divergence found | harness RV5 + pytest v12b |
| V13 | Cross-wired work_request_id | REFUSED (`cross_wired_work_request`) | pytest v13 |
| V14 | Identical replay | CONSEQUENCE-FREE duplicate; single ledger row | pytest v14 |
| V15 | Fresh event_id + mutated difference, consistently restacked | REFUSED BY NAME (`claim_mismatch`, diverged fields enumerated) | pytest v15 |
| V16/I10 | Divergent resubmission / silent rewrite | Stale idem-key ⇒ consequence-free duplicate; fresh key ⇒ named refusal; persisted first record byte-faithful to original upstream facts; upstream facts immutable in ledger | pytest v16 |
| V17 | Schema tampering: wrong event_type casing, schema_version, namespace (both levels), domain_model_ref rewrite, pseudo-hashes | ALL REFUSED | pytest v17..v17e |
| V17f/g | Type-confused int outcome; smuggled extra top-level field | ADMITTED (documented): fields present + internally consistent; inert provisional. Undocumented in builder report — minor hardening gap | pytest v17f/v17g |
| V18 | Fabricated difference: stale digests over mutated values; false equivalent verdict | BOTH REFUSED (`difference_binding_mismatch`); verdict is FORCED derivation from carried values — cannot claim equivalence for unequal values even with full restack | pytest v18a/b/c; harness RV7 |
| V19 | Fixture authenticity | AUTHENTIC: repo's own regen path (REAL T07 ingest + REAL E8 emitter public APIs) rewrote fixture byte-identically (sha 24ba9a1f… before = after; tree untouched). My independent real-chain recomputation matched every comparison fact | cargo test t08_regenerates…; sha256 before/after logged above |
| V20/T05-A9-T06 | Hand-forged self-consistent envelope (pure Python, never touching sxr) claiming producer=realitytrace | ADMITTED — producer exclusivity at this boundary is a wire-field convention, not authenticated origin. Bounded: record is inert; promotion requires explicit independent action. Consistent with builder's disclosed trust placement but MORE explicit than report states | pytest v20 |

### Debt-theme verdicts

- T07-D1 (out-of-band digest pinning at composition seams): PRESENT and CONFIRMED AS DISCLOSED. Producer seam pins proof_result_ref to actual ingested bytes and cross-checks settlement resolution (SettlementRefMismatch); consumer seam is structural-only for refs, out-of-band for parent ids via known_parent_ids. No contract text requires consumer-side byte verification; ENV-I8/I13 fence it; provisionality bounds blast radius.
- T07-D2/D4 (undelivered-ref markers): ghost refs admitted as inert rows (V07). Observable, non-promoting, non-mutating.
- T05-A9/T06 (trust placement): envelope self-claims are the admission basis (V20). Legacy adapters.py fallback pseudo-hash debt noted by builder stands pre-existing and is refused by the T08 gate.

### Commands and exits

| Check | Command | Result |
| --- | --- | --- |
| Freeze BEFORE | `just e2e-prereg-check` (sea-rs) | PASS, exit 0 |
| Freeze AFTER | same | PASS, exit 0; spec SHA ef571089…879f unchanged; plan SHA 0c57e543…0e39 unchanged |
| Rust suite | `cargo test --quiet --test convergence_t08_evidence_emission` (sxr-core) | 7 passed; exit 0 |
| Python suite | pytest tests/test_convergence_t08_evidence_recorded.py | 43 passed; exit 0 |
| Gate | `just e2e-gate T08` | exit 0 (7 Rust + 43 Python green) |
| Independent Rust battery | cargo run (/tmp/opencode/t08-verifier-rs) | 19 checks pass (RV6 split into RV6a/b after root-causing my own helper's idempotency-key format as divergence cause; authenticity settled by byte-identical regen) |
| Independent Python battery | pytest /tmp/opencode/t08-verifier/test_t08_attacks.py | 60 passed |

### Divergences vs builder claims

1. Builder's honest-scope disclosures (structural-only ref pinning; unpinned-parent acceptance; provisional-only semantics) are ACCURATE — attacks confirmed each disclosure rather than falsifying it.
2. UNDER-ARTICULATED (not dishonest): producer exclusivity is enforced against stamp VALUES with no cryptographic origin authentication; a fully hand-forged self-consistent envelope passes all gates (V20). Report implies but does not state this.
3. UNDISCLOSED minor acceptances (new findings, none contract-breaking): empty-list observed outcome (V03d), int-typed expected outcome (V17f), unknown extra top-level envelope field (V17g). All land as inert provisional data with enforced internal consistency.
4. CONFIRMED nuance behind builder's debt entry: claim-stability state is in-memory; across a restart, a divergent resubmission with fresh event_id AND fresh idempotency_key would be admitted as an additional inert provisional row (first row remains; no mutation/promotion).
5. Fixture regeneration claim verified stronger than stated: byte-identical rewrite through the real API chain.

### Verdict

**CONFIRM**

All frozen requirements survive review and attack:
- Plan falsifier 1 (missing expected/observed accepted): IMPOSSIBLE — refused at emitter seam AND Python gate seam across every variant probed.
- Plan falsifier 2 (forged parent/proof/settlement refs accepted): IMPOSSIBLE for every forge form that carries semantic weight — malformed refs, digest mismatches, cited-vs-resolved settlement mismatch (emitter), forged parents under out-of-band pinning, stale/false differences. The residual acceptance class (well-formed refs to undelivered bytes; unpinned parents; self-claimed producer) is (a) disclosed truthfully by the builder, (b) fenced by frozen I13/ENV-I8 conformance≠truth, and (c) bounded to strictly provisional, non-promoting, non-mutating storage — it cannot produce settlement, capability, or metabolization by any reachable path.
- Plan falsifier 3 (receipt alone creates SettlementRecorded/CapabilityUpdated): IMPOSSIBLE — zero writes to any promotion ledger from any reachable surface; ProvisionalEvidence exposes no promotion callable; even an abused explicit settlement with empty inputs cannot reach metabolized capability (I9 holds).
- Production-path implementation verified both sides: real emitter over REAL T07 ComparisonInputs (fixture byte-identically regenerated through the real chain); real Python gate composing real NavigationRuntime store state (executable-affordance fail-closed, real ledger writes) — not shims.
- Gates green; freeze SHA unchanged before/after.
- No counterexample remains that falsifies E8, I8, or I10.

Residual observations recorded above (V02c, V03d, V17f/g, V20 articulation, restart-window claim state) are hardening notes, not contract breaches.
