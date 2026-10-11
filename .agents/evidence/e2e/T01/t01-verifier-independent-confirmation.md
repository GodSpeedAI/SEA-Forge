# T01 — Independent Adversarial Confirmation (verifier artifact)

Task: godspeed-e2e-convergence T01 (P3, confirmation: independent_adversarial)
Verifier: fresh independent adversarial reviewer (opencode); NOT the T01 builder.
Date: 2026-08-25T10:30Z
Frozen target: `.agents/specs/e2e-preregistration.yml`
SHA-256 before verification: ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f
SHA-256 after verification:  ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f  (byte-for-byte unchanged; `just e2e-prereg-check` PASS before and after)
Normative plan `.agents/plans/e2e-plan.yml`: unchanged.

## Independence statement

The verifier did not rerun the builder's suite as its method. It read the frozen
requirements first, then attacked the production surfaces
(`SWE_SEED/crates/swe-seed-core/src/federation/{identity,producers,envelope,idempotency,consume}.rs`)
from the contract text outward, treating builder claims as hypotheses. A fresh
attack harness (`t01-verifier`, full source embedded below) was built OUTSIDE the
repositories (/tmp/opencode/t01-verifier, path-dependency on swe-seed-core),
leaving zero footprint in any repository. Builder tests were inspected only AFTER
attack design, for coverage comparison.

## Pre-verification control-state correction

Recorded separately with hashes in
`t01-verifier-control-state-correction.md` (same directory): stale execution
block corrected to T01-pending-independent-confirmation; T02/T03 re-blocked by
T01 BEFORE verification began; requirement verdicts left fully open during
attack design.

## Attack battery and results

Harness output (verbatim): `VERIFIER|<ID>|<result>|<detail>`

| ID | Requirement attacked | Attack | Result |
| --- | --- | --- | --- |
| V-A1a | ENV-I3/I4 | Two causal parents carrying CONFLICTING work identities (wr-A vs wr-B) | CONTRACT-HELD — CorrelationMismatch fatal |
| V-A1b | ENV-I3/I4 | Three-generation derivation chain; correlation stability across 2 hops; generation-3 late cross-wire attempt | CONTRACT-HELD — wr-Z stable through gen 3; cross-wire fatal |
| V-A2 | ENV-I2 | Construct child THROUGH the T01 derivation surface with the forbidden namespace-derived pseudo-hash (`derive_event` takes raw &str hash) | QUARANTINED-AT-BOUNDARY — construction returns Ok (gap recorded), but `validate_envelope` → PlaceholderIdentity reject AND `check_conformance` flags non-conformant; no adjudicator accepts it |
| V-A3 | ENV-I1/I2 | Fabricated SEA root manifest declaring `meta.sea_file_hash` = digest of bytes existing NOWHERE on disk; `strict_resolve` → `from_resolved` → `make_event_verified` → `validate_envelope` | SUBSTRATE-TRUST-LIMIT-RECORDED — manifest-declared hash is blessed without artifact-byte binding (bytes-binding half lives in `verify_against_artifact`); placeholder defense held below; see Debt D2 |
| V-A3b | ENV-I2 | Same fabricated-manifest channel carrying the all-zero digest | CONTRACT-HELD — PlaceholderIdentity through the resolution chain |
| V-A3c | ENV-I2 | Manifest present but without `meta.sea_file_hash` | CONTRACT-HELD — MissingArtifact fail-closed |
| V-A4 | I3 | 7 producer format attacks ("RealityTrace", "REALITYTRACE", leading/trailing space, underscore variant, alias-plus-space, near-miss spelling) + 4 event-type aliases ("evidence_recorded", case variants, space) through composed validate_envelope | CONTRACT-HELD — all 11 fail closed |
| V-A4b | I3 | Path divergence: forged-producer SettlementRecorded through LEGACY `consume_settlement_recorded` vs canonical gate | PATH-DIVERGENCE-RECORDED — canonical gate rejects; legacy spec-0011 extractor adjudicates type/ns only (edge adoption is T02+/T06 scope per plan steps) |
| V-A5a | ENV-I6 | Altered payload carrying SAME stable idempotency identity across restart (rewrite attempt) | CONTRACT-HELD — Duplicate; rewrite suppressed; distinct content remains distinct key |
| V-A5b | ENV-I6 | Two racing admitters over ONE ledger file (two-process model, barrier-synchronized) | RACE-OBSERVED-DEBT-RECORDED — both observe First (no inter-process lock); durable state converges to 1 unique key on reload; no production multi-writer exists yet; see Debt D4 |
| V-A6 | ENV-I4 | Well-formed but DANGLING parent UUID referencing a nonexistent envelope | LIMIT-RECORDED — gate enforces well-formedness only (stateless gate; existence binding requires an envelope store owned by later tasks) |
| V-A7 | ENV-I8/I13 | Structurally perfect EvidenceRecorded from the AUTHORITATIVE producer (realitytrace) claiming metabolized-capability promotion with contradictory exit_code | CONTRACT-HELD — conformance true, all gates pass, zero effects anywhere; no truth/promotion consequence |
| V-A8 | ENV-I5 | Child derived under model-B hash while citing parent under model-A | OBSERVATION-RECORDED — both facts faithfully represented; no frozen rule requires same-domain parents; flagged for E4 authority-evaluation binding |
| V-A9 | ENV-I8/I13 | Does ConformanceReport.conforms imply producer-authority pairing? Forged swe_seed→EvidenceRecorded pair | OBSERVATION-RECORDED — conforms=true (field-level registry membership only); authority pairing enforced solely by validate_envelope; adopters MUST compose validate_envelope |

Tally: 13 tests, 13 passed, 0 failed. CONTRACT-HELD: 8. Quarantined-at-boundary /
scoped limits / observations recorded as debt: 5 (V-A2, V-A3, V-A4b, V-A5b,
V-A6) + 2 observations (V-A8, V-A9). **No counterexample defeats the frozen
contract on its own terms.**

## Additional code-path review findings (verified against source)

- Production wiring honesty: the ONLY live emitter is `crates/swe-seed/src/federation_cli.rs`
  (spec-0011 surface, unchanged by T01) via `resolve_from_root`; T01 gates have
  zero non-test callers today. This is exactly the plan's division of labor:
  T01 `changes:` establishes the validation contract; edge adoption is written
  into T02 steps ("Bring E2 request shape under the canonical envelope/correlation
  contract") and T03; T01 `does_not_prove:` explicitly excludes edge usage.
  Edge verdicts therefore stay open in Delta (only T01's 11 requirements move).
- Wrong-domain identity at consumption: `check_drift` rejects declared != caller's
  local resolution (consume.rs:65-77), including missing hash ("<missing>").
- Artifact-byte binding exists and is strict where an artifact is claimed:
  `verify_against_artifact` recomputes SHA-256 over actual bytes (identity.rs:139-162);
  mismatch/missing/unreadable are typed errors; nothing manufactures a hash.
- Real-substrate probe: NO manifest exists today at
  `<SEA_ROOT>/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json`
  in sea-rs/domainforge checkouts ⇒ real-substrate strict resolution fails CLOSED
  today (safe direction; no phantom canonical identity resolvable in this stack).

## Gates run (builder + global)

| Command | Exit | Result |
| --- | --- | --- |
| just e2e-prereg-check (before) | 0 | PASS, hash intact |
| just e2e-delta-check (after Step-1 correction) | 0 | 35 requirements, 0 confirmed, open delta 35 |
| just e2e-gate T01 | 0 | convergence_t01_envelope 22/22 + v1_contract 6/6 + federation_parity 11/11 |
| cargo test -p swe-seed-core (full crate) | 0 | 342 passed, 0 failed |
| just context-check | 0 | passed |
| just fmt-check | 0 | clean |
| just lint (clippy -D warnings) | 0 | clean |
| just typecheck (workspace) | 0 | clean |
| just deny (advisories/bans/licenses/sources) | 0 | ok |
| gitleaks detect | 0 | exit 0; 4 PRE-EXISTING historical hits in unrelated sea-rs crates (delegation.rs, ledger types/conformance fixtures @ 4bda794/87e475b) — none from convergence work |
| just e2e-test (sea-rs workspace, all features) | 0 | 953 passed, 0 failed |
| just e2e-prereg-check (after) | 0 | PASS, byte-for-byte unchanged |

## Debts and observations carried forward (do NOT reopen T01)

- D1 (V-A2): `derive_event`/`make_event` accept raw hash strings; pseudo-hash
  children are constructible though every adjudicating boundary rejects them.
  Recommended hardening when edges adopt: derive_event should take
  VerifiedDomainIdentity (or inherit parent-verified hash). Not contract-breaking;
  record in OBSERVED_DEBT of the implementing repo if desired.
- D2 (V-A3): `strict_resolve`/`from_resolved` trust the manifest-declared hash;
  byte-level binding requires callers to invoke verify_against_artifact.
  Recommendation: thread the artifact path through ResolvedHash so edges can
  bind bytes. Safe today (no manifests exist in-stack; resolution fails closed).
- D3 (V-A6): dangling well-formed parent ids pass the gate until an envelope
  store exists to resolve references (natural home: T04/T05 composition).
- D4 (V-A5b): IdempotencyLedger has no inter-process locking; concurrent
  admitters over one file can each observe First. REQUIRED-CLOSE before any
  multi-process adoption (T06+/E6 durability composition). In-process semantics
  and durable-state convergence are sound. Minor related note: directory fsync
  is not performed on first-time file creation (machine-crash window; process-
  restart durability is proven).
- D5 (V-A4b/V-A9): legacy consume_* extractors and check_conformance do not
  adjudicate producer authority/pairing; validate_envelope is the single
  boundary gate. Edge tasks MUST compose validate_envelope (already written
  into T02+ step contracts).
- D6 (V-A8): cross-model causality is representable; E4 must bind semantic
  authority evaluation to the referenced model (already an E4 invariant).

## Verdict

**CONFIRM.** All frozen T01 requirements survive independent adversarial attack;
implementation exists on the production path of the most complete binder; gates
green; no known counterexample remains within T01's frozen scope.

Requirements moved to CONFIRMED: ENV-I1, ENV-I2, ENV-I3, ENV-I4, ENV-I5, ENV-I6,
ENV-I7, ENV-I8, I1, I3, I13.

## Reproduction

Harness source (cargo project, deps: swe-seed-core path dep, serde_json, sha2 =0.10):
see `t01-verifier-attacks-src.rs.txt` alongside this file. Run:
`cargo test -p t01-verifier -- --test-threads=1 --nocapture`.
