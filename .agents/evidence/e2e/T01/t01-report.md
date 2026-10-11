# T01 — Canonical Semantic Envelope and Domain Identity (builder evidence)

Plan: `.agents/plans/e2e-plan.yml` task T01 (P3, independent_adversarial).
Frozen target: `.agents/specs/e2e-preregistration.yml` (ENV-I1..ENV-I8, I1,
I3, I13). Preregistration SHA-256:
`ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`
(verified intact by `just e2e-prereg-check` at evidence time).

## What was built

The frozen envelope/identity contract is enforced in the most complete existing
production binder — SWE_SEED `crates/swe-seed-core` federation module — using
only its existing dependencies (sha2/uuid/ulid/serde_json). No new envelope
family was created; the v1 wire schema (`sea.agent.event.v1.json`) and the
existing `Envelope` type are unchanged on the wire.

New enforcement surface (`src/federation/`):

- `identity.rs` — canonical identity gate: `VerifiedDomainIdentity` can only be
  constructed from a well-formed SHA-256 digest that is not a known
  pseudo-hash (standalone fallback constant sha256("agentic_capability_loop"),
  all-zero) and, when an artifact is claimed, equals that artifact's recomputed
  content hash. Missing/unreadable/mismatching artifacts are typed errors;
  nothing manufactures a hash. `strict_resolve()` replaces the fallback branch
  of environment resolution with an error. Fallback-sourced resolutions are
  refused by `VerifiedDomainIdentity::from_resolved`.
- `producers.rs` — exclusive event-producer registry (invariant I3): every
  canonical event type maps to exactly one authoritative producer using the
  preregistration's component-id vocabulary; hyphenated legacy agent stamps are
  aliased explicitly; unknown event types and unknown agents fail closed.
- `envelope.rs` additions — `make_event_verified()` (identity-gated
  construction), `derive_event()` (requires ≥1 distinct parent; records
  `caused_by:<event_id>` entries in `provenance.chain`, preserving the v1
  schema's top-level shape; carries `work_request_id` unchanged and refuses
  correlation mismatches — ENV-I4/I3), accessors `causal_parents()` /
  `work_request_id()`.
- `consume.rs` additions — `validate_envelope()` composes producer authority +
  canonical identity + drift + causality checks as the single boundary gate;
  `check_conformance()` returns a pure structural `ConformanceReport` that
  cannot perform effects or evaluate claim truth (ENV-I8/I13 separation).
- `idempotency.rs` — `IdempotencyLedger`: append-only, fsync-per-admission key
  set; duplicates never rewrite storage; reload-after-restart still dedupes
  (ENV-I6); corrupt ledger lines fail closed.
- Tests: `tests/convergence_t01_envelope.rs` (22 tests).

## Preregistered falsifier → proof map

| Frozen falsifier / attack | Proof |
| --- | --- |
| Missing model yields apparently valid canonical hash | `env_i1_missing_artifact_is_rejected_not_manufactured`, `strict_resolution_without_any_manifest_errors_instead_of_falling_back` |
| Wrong producer emits another component's authoritative event | `teeth_2_evidence_recorded_forged_by_non_realitytrace_is_rejected`, `i3_settlement_recorded_owned_by_godspeed_agent_not_swe_seed`, `i3_unknown_event_types_and_agents_fail_closed` |
| Derived envelope loses causal-parent identity | `env_i4_derived_envelopes_record_distinct_causal_parents`, `env_i4_validate_envelope_rejects_forged_parent_ids`, `env_i4_derivation_requires_parents_and_rejects_duplicates` |
| CEP/schema success alone accepts a claim as true | `teeth_3_conformant_envelope_with_false_claim_has_no_truth_consequence` (conformance passes a self-contradictory claim; zero effects; effects only via explicit admission) |
| Fallback pseudo-identity settles as valid | `env_i2_standalone_fallback_pseudo_hash_is_rejected`, `resolved_from_fallback_source_cannot_become_verified_identity`, `envelopes_can_only_be_built_through_the_identity_gate` |
| Replay/duplicate delivery duplicates consequence | `env_i6_duplicate_delivery_admits_exactly_once_across_restart` |
| Upstream correlation rewritten downstream | `env_i3_correlation_cannot_silently_change_across_derivation` |

## Verification performed

```
cargo test -p swe-seed-core --test convergence_t01_envelope   # 22 passed
cargo test -p swe-seed-core                                   # all suites green,
                                                              # 0 failed (2 pre-existing ignored)
cargo fmt (touched files)                                     # clean
clippy over touched code                                      # clean (one ptr_arg fixed)
```

Gate binding: `just e2e-gate T01` (SEA Forge root) runs the T01 suite plus the
adjacent v1-contract and federation-parity pins.

## Honest status

Builder-run teeth only: **confirmation = PENDING — fresh independent
adversarial review not yet obtained; NOT SETTLED** (plan requires
independent_adversarial CONFIRM before T01 settles and verdicts flip).
Requirement verdicts in `.agents/status/e2e-current-status.yml` therefore stay
open; this file is the builder evidence for the future verifier. Known scope
boundary honored: T01 does not prove any runtime edge uses the envelope
correctly (that is T02+); adoption of these gates by Context Kernel and
GodSpeed-Agent binders lands with their edge tasks.
