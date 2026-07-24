# Tasks 1–4 Specification Reconciliation

Date: 2026-07-12

## Input inventory

Found:

- `.agents/specs/spec-full.md`
- `.agents/specs/spec-minimum.md`
- `.agents/plans/2026-07-11-spec-full-implementation.md`
- `.agents/CURRENT_STATUS.md`
- commits `eb62638` through `e20f822`
- task tests under `crates/*/tests` and crate-local test modules
- proof commands in `justfile`

Not found after repository and nearby-project searches:

- proposed full-spec patch
- complete proposed patched specification
- patch guide
- task-specific reports or proof artifacts for Tasks 1–4 beyond git history,
  tests, `.agents/CURRENT_STATUS.md`, and command output

Therefore `git apply --check <patch-path>` cannot be run. The reconciliation
uses the proposed requirements in the user request and manually adapts valid
semantics to the repository.

## Substrate map

| Area | Actual implementation | Public entry points | Existing proof | Relevant task |
|---|---|---|---|---|
| Ledger | `crates/sea-forge-ledger/src/{types,signing}.rs`; separate from the minimum pipeline | `LedgerStream::{open,append,verify,prove_inclusion,create_checkpoint}`, global checkpoint/witness/recovery APIs; CLI `ledger verify|prove` | 7 unit + 16 M0 ledger conformance tests | Task 2 |
| Authority | `crates/sea-forge-authority/src/lib.rs`; minimum local policy evaluates one verdict per operation | `AuthorityPolicyBundle::load`, `PolicyAuthorityEngine::{new,evaluate}` | 7 authority tests plus CLI lifecycle allow/deny/escalate tests | Task 1 substrate; Task 5 owns full fabric |
| Runtime/sandbox | `crates/sea-forge-runtime/src/lib.rs`, `crates/sea-forge-sandbox/src/lib.rs`; CLI pipeline evaluates all operations, then calls raw lower-layer APIs | `runtime::execute(&ExecutionRequest, ...)`, `sandbox::materialize(root, &Operation)` | 5 runtime tests, 2 sandbox path tests, CLI lifecycle tests | Graduated in Task 1; non-bypassable mediation is Task 5 |
| DomainForge boundary | `crates/sea-forge-domainforge/src/lib.rs`; in-memory parser/graph validation and candidate normalization | `load_validate`, `normalize_authority` | 5 conformance tests | Task 3 |
| Extension ABI | `crates/sea-forge-extension/src/lib.rs`; JSON registry, descriptors, install/projection records | registry load/save/register/import/adopt, descriptor validation, `ProjectionAdapter` | 8 unit tests | Task 4 |

Crate dependencies remain directed from CLI orchestration toward kernel crates;
kernel crates do not depend on CLI. `sea-forge-ledger`,
`sea-forge-domainforge`, and `sea-forge-extension` do not create a second
authority implementation.

## Reconciliation matrix

| Proposed requirement | Existing spec location | Existing code location | Existing proof | Classification | Smallest action | Debt or compatibility risk |
|---|---|---|---|---|---|---|
| Deterministic verdict resolution where allow cannot erase stricter controls | `spec-full.md` §§7.0, 10.0, 12, 17; existing order incorrectly places `allow` above blocking `escalate` | Minimum engine produces one `Allow|Deny|Escalate`; DomainForge returns one candidate normalization; no multi-engine resolver exists | Minimum allow/deny/escalate lifecycle tests; no pairwise resolver proof | `spec_only_now` | Specify a typed resolver: deny blocks; unresolved escalation blocks while retaining boundaries; boundaries intersect; permitted degraded controls accumulate; allow is identity. Assign implementation and algebraic tests to Task 5 | A scalar total order would erase boundary parameters or let boundary hide unresolved escalation |
| Consequential operations require authority bound to exact action and active context | §10.0 says one mediator but does not define the grant properties | `runtime::execute` and `sandbox::materialize` are public raw APIs; `pipeline::run_intent` gates only by convention | CLI lifecycle shows the current path evaluates first; no bypass/forgery/replay tests | `spec_only_now` | Specify an opaque, context-bound, expiring, non-deserializable execution grant or equivalent mediator-owned capability; implement in Task 5 | Current public APIs remain bypassable until Task 5; adding a decorative serializable token would worsen the design |
| Canonical ledger commit precedes compatibility/materialized views | §§7.0c, 9, 10.0a already establish ledger truth but do not fully state partial-failure/view freshness behavior | Ledger append exists; minimum pipeline, extension registry, trace/evidence/capability writers still persist directly | Ledger integrity tests; no ledger→view failure/rebuild proof | `spec_only_now` | Specify commit sequence, immutable committed ref, detectable projection state, and rebuildability; Task 5 wires current v0.2 authority/application writes, later owners wire their stores | No v0.2 application integration exists yet; minimum v0.1 files must remain unchanged for P1–P4b |
| `M0-G1` crate graduation and record contracts | §17 has only one aggregate M0 row | Workspace and graduated crates | workspace tests, `just no-async-kernel`, P1–P4b | `already_satisfied` for graduation; record v0.2 integration pending | Add internal evidence gate without renaming commits | Do not interpret G1 as all v0.2 records integrated |
| `M0-G2` ledger core and migration | §§7.0c, 10.0a, 12; Task 2 and Task 6 plan | Ledger core exists; migration does not | 23 ledger tests; no migration proof | `compatible_gap` | Define G2 as partial until Task 6 migration and composed ledger/view proof pass | Task 2 package-green is not G2 satisfied |
| `M0-G3` authority fabric | §§7.0, 10.0; Task 5 plan | Minimum local engine only | Minimum authority tests only | `spec_only_now` | Mark not started and assign resolver, grant boundary, mediation, and integration proofs to Task 5 | Must not credit minimum convention as non-bypassability |
| `M0-G4` DomainForge semantic boundary | §7.0a; Task 3 plan | Adapter and normalization exist | 5 tests | `already_satisfied` only for currently tested Task 3 slice; aggregate gate remains partial | Preserve implementation; require composed authority use in G6 | Existing proof does not cover Task 5 candidate combination |
| `M0-G5` extension ABI and registry | §7.0b; Task 4 plan | Registry/ABI exists | 8 tests | `already_satisfied` at Task 4 package gate | Preserve; authority-checked adoption remains Task 5 integration | Current `adopt` call is not itself the future authority boundary |
| `M0-G6` composed M0 proof | §17 aggregate M0 row | No composition yet | none | `not_started` | Require G1–G5, migration, no-bypass, canonical commit/view, and unchanged minimum proofs | Cannot be inferred by adding package test counts |
| Completion-claim levels | Existing spec distinguishes milestone gates and proof classifications but not repository-wide claim vocabulary | Status currently says tasks complete based on package tests | Package tests and minimum proofs | `compatible_gap` | Add explicit five-level claim vocabulary and prohibit promotion by implication | “Implementation complete” must not be read as integration proven |
| Release boundaries: alpha M0–M3, capability beta M0–M4a, memory beta M0–M4b, full M0–M8 | Appendix A dependency order supports these cumulative boundaries | M0 incomplete | none at release level | `spec_only_now` | Add cumulative release labels with prerequisites and narrow pilot proof | Labels are planning boundaries, not current release claims |

## Decision

Use **Option 3 — manually reconcile**. There is no patch file to check or
apply, the current specification already contains related but semantically
incorrect precedence text, and implementation of the authority boundary is
explicitly Task 5. No parallel authority wrapper or persistence path will be
introduced during reconciliation.

## M0 gate evidence map before reconciliation edits

| Gate | Status | Implementation refs | Proof refs | Remaining gap |
|---|---|---|---|---|
| M0-G1 | satisfied | `Cargo.toml`, graduated `crates/sea-forge-*` | workspace tests; `just no-async-kernel`; P1–P4b | v0.2 record use is composed in later gates |
| M0-G2 | partial | `crates/sea-forge-ledger` | ledger unit/conformance tests | migration, canonical application commit/view proof, composed wiring |
| M0-G3 | conformance_green | `sea-forge-authority` typed resolver, exact-action grants, v0.2 RBAC/SoD/identity policy, DomainForge composition, opaque constraints, ledger-first authority views, pre-action checkpoints/witnesses | `conformance_m0_authority`, authority unit tests, CLI lifecycle ingress/integrity tests, ledger checkpoint coverage tests, P1–P4b | none; Task 6 remains the next milestone task |
| M0-G4 | satisfied | `crates/sea-forge-domainforge` | `conformance_m0_domainforge.rs` | composed candidate use belongs to G3/G6 |
| M0-G5 | satisfied | `crates/sea-forge-extension` | 8 crate tests | authority-mediated adoption belongs to G3/G6 |
| M0-G6 | not_started | none | none | Tasks 5–6 and aggregate M0 proofs |

## Deferred proof ownership

- Task 5: pairwise resolver matrix and algebraic properties; deny/escalate/
  boundary/degraded execution semantics; grant forgery, mismatch, expiry,
  replay, sandbox downgrade, approval, and compensating-control tests; current
  ingress no-bypass proof; ledger-before-view failure/rebuild tests for records
  Task 5 begins persisting.
- Task 6: legacy migration, genesis commitments, and G2 closeout.
- M0-G6: complete composed proof, including all v0.2 records then in scope and
  unchanged minimum P1–P4b.

These are assignments, not claims that the behavior already exists.
