# T03 — Navigation Ingress and Work Contract (E0, E1) — builder evidence

Plan: `.agents/plans/e2e-plan.yml` task T03 (P2, builder_with_teeth). Frozen
target: `.agents/specs/e2e-preregistration.yml` SHA
ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f (intact).

## What was built

GodSpeed-Agent (`godspeed_nav/canonical_events.py`, new):

- Canonical v1 envelope projector mirroring the wire family byte-for-byte in
  shape (schema_version/event_id/source_agent/event_type/occurred_at/
  idempotency_key/payload/provenance; sha256 idempotency over
  `correlation|type|sorted-payload`).
- `verify_domain_model_hash`: malformed/all-zero/standalone-fallback
  pseudo-hashes refused fail-closed (ENV-I2 adoption on the GSA side).
- E0 `project_desired_direction`: canonical DesiredDirection envelope stamped
  by its authoritative producer (`external-environment`) with the frozen
  required fields (direction_id/direction_type/desired_state); incomplete
  directions rejected.
- E1 `project_work_requested`: projects a SELECTED EXECUTABLE affordance into
  a WorkRequested envelope stamped by the exclusive producer
  (`godspeed-agent`), carrying work_request_id correlation, affordance_id,
  desired_outcome, settlement_criteria, and verified domain identity; the E0
  direction event is recorded as causal parent when supplied.
- Fail-closed teeth: destination-only inputs (no pathway / no payment estimate)
  raise `destination_only_not_spendable`; missing settlement criteria raise
  `settlement_criteria_required`. This composes with GSA's existing type-level
  rule (`Affordance.__post_init__` already refuses pathway-less affordances;
  `Potential.to_affordance()` is forbidden) — the projection layer now enforces
  it at the canonical boundary too.

SWE_SEED (`crates/swe-seed-core/src/federation/work_ingress.rs`, new):

- `accept_work_requested(envelope, expected_hash)` — the canonical E1 ingress:
  producer authority (only godspeed_agent may emit WorkRequested), identity
  gate + drift against local resolution, and contract completeness
  (work_request_id / affordance_id / desired_outcome / non-empty trimmed
  settlement criteria). Missing/empty required fields return the new typed
  `ConsumeError::DestinationOnly` — an ungrounded objective can never settle
  as a work contract.
- `ConsumeError::DestinationOnly` added to the shared error family.

## Cross-language production evidence

`tests/test_canonical_events.py::test_writes_golden_fixture_for_swe_seed_ingress`
regenerates `tests/fixtures/t03_work_requested.json` from GSA's REAL projector;
SWE_SEED's `t03_golden_fixture_from_gsa_projector_is_accepted` loads that file,
decodes it as a canonical Envelope, and runs it through the real ingress gate.
Fixture committed; test skips honestly if the repo is absent.

## Teeth → proof map

| Plan tooth / frozen falsifier | Proof |
| --- | --- |
| Destination-only input ⇒ no canonical WorkRequested emitted | Python `test_destination_only_affordance_cannot_become_work_requested`, `test_pathway_without_payment_is_still_destination_only`; Rust `t03_destination_only_*`, `t03_work_request_without_settlement_criteria_is_rejected` |
| Cross-wired domain-model identity ⇒ reject before acceptance | Rust `t03_cross_wired_domain_identity_is_rejected` (drift), `t03_fallback_pseudo_hash_cannot_settle_a_work_contract` |
| Wrong producer (incl. SWE_SEED self-producing E1) ⇒ rejected at ingress | Rust `t03_non_godspeed_producer_cannot_enter_work_contracts` |
| E0 production authority | Python `test_desired_direction_projects_with_external_producer`; Rust `e0_desired_direction_produced_only_by_external_environment` |
| Pseudo-identity anywhere in the path | Both sides' placeholder batteries |

## Verification performed

```
GSA:  uv run --extra dev python -m pytest tests/test_canonical_events.py   # 11 passed
      full suite: 607 passed, 5 pre-existing failures + 1 pre-existing e2e
      collection error — VERIFIED pre-existing by removing my files and
      reproducing identical failures without them (ruvector/distribution env).
SEED: cargo test -p swe-seed-core --test convergence_t03_work_ingress      # 10 passed (fixture consumed, no skip)
GATE: just e2e-gate T03                                                    # exit 0
fmt:  touched files clean
```

## Honest scope notes

- The physical GSA→SWE_SEED hand-off surface is the canonical envelope itself
  (file/sink transport follows SWE_SEED's existing dispatch pattern); T04's
  governed submission builds the next hop. What settles here: producer
  direction, contract completeness, identity binding, and both falsifiers are
  machine-enforced at the boundary.
- Runtime ledger rows were left untouched; the projector accepts their dict
  shapes as-is.

## Worktree-integrity incident note (post-settlement sweep)

During the final integrity sweep, ~24 SWE_SEED files that were CLEAN at
session start were found modified with pure rustfmt churn (module/import
reorder + line reflow; e.g. `mod gateway_cli` reordering in main.rs,
vec![] reflows). The burst coincided with this agent's file-write/build
activity at 12:58:39 (mechanism not conclusively identified — no repo hook,
justfile recipe, or build script invokes rustfmt; suspected toolchain-side
format-on-write). Every affected diff was verified formatting-only against
HEAD (whitespace/comma-normalized hashes + manual hunk review of all six
non-comma-only cases), then restored via `git checkout --`. Post-restore:
working tree = session-start state PLUS exactly the authorized convergence
edits; full suite re-run green (363 passed / 0 failed); e2e-gates T01/T02/
T03 re-run exit 0. Originally-dirty spec-0020 files (gateway/catalog.rs,
gateway/mod.rs, gateway/serve.rs, gateway/discover.rs, context_client WIP,
docs, agents) were never reverted and remain byte-identical to their
pre-session content plus, where applicable, this task's authorized edits.
