# Current Status

Updated: 2026-07-13

## Objective

Implement `.agents/plans/2026-07-11-spec-full-implementation.md`: execute the
SEA Forge full-system plan (spec-full.md M0–M8) task by task, committing after
each milestone gate. Work happens on the `full-spec` branch; `ci-cd` was merged
to `main` and pushed to origin first.

## Worktree State

On branch `full-spec`. Tasks 1–8 are implemented at their package gates; M0
is complete, M1 jail sandbox and M2a case engine are conformance-green. The
minimum kernel tests and P1–P4b remain the unchanged compatibility floor.
Task 9 (M2b plan templates) is next.

## Changed Files

- `crates/sea-forge-sandbox/src/lib.rs` — SandboxClass, ExecutionSandbox trait,
  select_sandbox, SandboxSpec/Handle/Error/RelPath types.
- `crates/sea-forge-sandbox/src/local.rs` — LocalSandbox backend (existing behavior).
- `crates/sea-forge-sandbox/src/jail.rs` — JailSandbox backend (Linux Landlock).
- `crates/sea-forge-sandbox/tests/conformance_m1.rs` — M1 conformance tests.
- `crates/sea-forge-runtime/src/lib.rs` — uses sandbox backend from grant's class.
- `crates/sea-forge-core/src/types.rs` — added ExecutionStatus::SandboxViolation.
- `crates/sea-forge-settlement/src/lib.rs` — settlement basis `jail_violation`.
- `crates/sea-forge-authority/src/lib.rs` — ActionGrant exposes sandbox_class(),
  relaxed hardcoded local-only check to allow any granted class.
- `crates/sea-forge-cli/src/commands/migrate.rs` — new `sea-forge migrate` command.
- `crates/sea-forge-cli/src/commands/inspect.rs` — finds run dirs in both v0.1 flat
  and v0.2 case-nested layouts.
- `crates/sea-forge-cli/src/commands/mediated.rs` — migrated roots report
  `legacy_digest_only` assurance without requiring a signer.
- `crates/sea-forge-ledger/src/types.rs` — `LedgerStream::verify` checks
  `legacy_import` files against recorded sha256/size.
- `crates/sea-forge-cli/tests/conformance_m0_migrate.rs` — M0 migration gate tests.
- `Cargo.toml` — added 10 new kernel crate members to workspace.
- `justfile` — added `no-async-kernel` recipe; wired into `ci`.
- `Cargo.lock` — refreshed by the workspace expansion.
- `crates/sea-forge-core/src/lib.rs` — reduced to ids/types/errors + `RECORD_VERSION`.
- `crates/sea-forge-cli/src/main.rs` — added `mod pipeline` and `Migrate` command.
- `crates/sea-forge-cli/src/pipeline.rs` — moved from `sea-forge-core`.
- `crates/sea-forge-cli/src/commands/{run,recall}.rs` — updated imports.
- `crates/sea-forge-cli/src/tests/lifecycle.rs` — updated evidence imports.
- `crates/sea-forge-cli/Cargo.toml` — added kernel crate dependencies.
- New crates: `sea-forge-domain`, `sea-forge-authority`, `sea-forge-planner`,
  `sea-forge-sandbox`, `sea-forge-runtime`, `sea-forge-trace`, `sea-forge-evidence`,
  `sea-forge-settlement`, `sea-forge-capability`, `sea-forge-extension`,
  `sea-forge-ledger` (foundation).
- `Cargo.toml` / `Cargo.lock` — added 11 new kernel crate members, added
  `ed25519-dalek` to workspace dependencies.

## Completed

- Merged `ci-cd` into `main` and pushed to `origin/main`.
- Task 1 — M0a mechanical crate graduation: moved 14 slice modules into 10 new
  kernel crates plus `pipeline.rs` into `sea-forge-cli`, fixed cross-crate imports,
  added required dependencies, added `no-async-kernel` check, and verified the
  gate: `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace`,
  `just proof`, `just no-async-kernel` all pass.
- Noted and fixed one test-path issue: `runtime::tests::timeout_child_helper` became
  `tests::timeout_child_helper` after the move; this is a path reference update,
  not a logic change.
- Task 2 — M0b sea-forge-ledger: complete. All §12 M0 ledger conformance fixtures
  pass: 1000-record multi-stream append with ULID/ordinal/chain/MMR verification;
  one-byte alteration / truncate / reorder / duplicate detection with typed
  `ledger_integrity_error`; Ed25519 signed checkpoints with chain verification;
  MMR inclusion proofs; global checkpoints committing all stream roots;
  independent witness receipts detecting fork substitution (and rejecting
  self-witnessing); secret sentinel redaction rejecting plaintext private keys
  and API keys while accepting approved ciphertext commitments; key rotation
  with old checkpoints verifying under snapshotted key refs; crash recovery
  quarantining incomplete tails. CLI `ledger verify|prove` subcommands added.
- Task 3 — M0c DomainForge semantic adapter: `crates/sea-forge-domainforge`
  created with `domainforge-core = "=0.13.0"`, default features off. Implements
  `load_validate(SeaSourceSet) -> DomainModel` using DomainForge's parser → graph →
  validation pipeline; `DomainModelRef` with `semantic_model_sha256` over canonical
  4-tuple; authority normalization table (Reject/Deny→deny, Escalate→escalate,
  Allow→allow, NotApplicable→deny-if-required); real `.sea` fixture; conformance
  tests covering valid parse → stable ref, invalid syntax → domain_model_error,
  source-hash drift rejection, no-side-effects-on-invalid-input, and normalization.
  pass: 1000-record multi-stream append with ULID/ordinal/chain/MMR verification;
  one-byte alteration / truncate / reorder / duplicate detection with typed
  `ledger_integrity_error`; Ed25519 signed checkpoints with chain verification;
  MMR inclusion proofs; global checkpoints committing all stream roots;
  independent witness receipts detecting fork substitution (and rejecting
  self-witnessing); secret sentinel redaction rejecting plaintext private keys
  and API keys while accepting approved ciphertext commitments; key rotation
  with old checkpoints verifying under snapshotted key refs; crash recovery
  quarantining incomplete tails. CLI `ledger verify|prove` subcommands added.

## Remaining

- Tasks 8–17 from the implementation plan (M2a case engine through M8 + DoD sweep).
- Continue milestone-ordered implementation, committing after each task.
- Stale stash `stash@{0}` remains from the initial workspace cleanup; will drop
  once the log-file reset is no longer a safety-net concern.

## Verification

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo test --workspace --all-features --locked`: 73 tests passed (35 existing +
  23 ledger + 5 domainforge + 8 extension + 2 migration), 0 failed.
- `just proof`: P1–P4b passed.
- `just no-async-kernel`: passed.
- `cargo build --workspace --all-targets --locked`: passed.
- `cargo test -p sea-forge-cli conformance_m0_migrate --locked`: passed.
- Task 6 migration gate: lossless genesis import, idempotence guard, ledger
  verify corruption detection, and `legacy_digest_only` inspect assurance pass.
- Task 7 — M1 jail sandbox backend: `SandboxClass` enum (`local|jail|microvm`),
  `ExecutionSandbox` trait (§11.2), `LocalSandbox` (existing behavior), and
  `JailSandbox` (Linux Landlock via the `landlock` crate). Landlock ruleset
  allows read-write to workspace+artifacts, read-only to `/`, denies all other
  writes. Thread-based restriction (no `unsafe`/`pre_exec`) keeps the main
  thread unrestricted. Runtime selects backend from `grant.sandbox_class()`;
  unavailable class returns `unsupported_sandbox_class_error`. Settlement adds
  `jail_violation` basis when `ExecutionStatus::SandboxViolation` is detected.
  Conformance tests: jail blocks write outside workspace, schema_error for
  untrusted argv0 on local, class identity, unavailable-platform refusal.
- `cargo test --workspace --all-features --locked`: 80 tests passed, 0 failed.
- `just proof`: P1–P4b passed.
- `just no-async-kernel`: passed.
- Task 5 resolver slice: 12 `sea-forge-authority` tests passed, including typed
  deny/escalate/boundary/degraded/allow resolution and order independence.
- Task 5 execution-boundary slice: authority no longer depends on sandbox;
  move-only, non-serializable grants bind the exact action/run/item/workspace;
  public sandbox materialization and runtime execution consume a matching grant.
- Task 5 canonical-decision slice: `LedgerStream::commit_typed` returns an
  opaque committed-record reference; the CLI commits every authority decision
  before writing `authority.json` or issuing its exact-action grant.
- Task 5 review fixes: boundary dimensions now intersect and incompatible
  boundaries deny; grants require a one-use decision issued by the same engine
  plus a non-deserializable committed ref; authority evidence commits before
  the decision that cites it.
- Task 5 candidate slice: authority decisions now persist candidate verdicts,
  winning source, resolution reason, sandbox grant, boundaries, and controls;
  v0.2 engine declarations reject fail-open modes and unavailable required
  engines deny through the typed resolver.
- Task 5 view/extension slice: authority compatibility output materializes only
  after its committed decision and records current/failed freshness; failed
  materialization preserves verifiable ledger truth. Extension registry saves
  are ledger-first, and imported adoption consumes an exact-action grant plus a
  committed authority reference.
- Task 5 ingress slice: run commits intent, plan, identity, policy, request,
  evidence, and decision before effects; validate, recall, and inspect now pass
  through the same authority engine and ledger-backed exact-action check before
  reading protected data. Minimum v0.1 read behavior remains compatible; v0.2
  policies require explicit read rules.
- Task 5 hardening: v0.2 identity maps fail unresolved identities closed and
  require sponsors for automated agents; complete protected operation names and
  fail-closed engine declarations parse in one schema; DomainForge candidates
  compose through the typed resolver; grants bind timeout, environment keys,
  workspace, artifacts, sandbox class, boundaries, controls, and expiry.
- Required-integrity policies now produce signed stream/global checkpoints and
  independently signed witness receipts before any command-start event. Missing
  or duplicate witnesses fail closed before workspace effects. Authority decision,
  audit, and opaque-constraint mirrors rebuild from ledger records; inspect and
  recall surface ledger assurance.
- Task 5 / M0-G3 complete: v0.2 policy snapshots require all authority surfaces,
  RBAC permissions, SoD rules, source hashes, and canonical bundle hashes;
  configured DomainForge evaluation runs through real CLI ingresses; opaque
  constraints preempt matching work; per-record assurance proves inclusion in
  the exact signed/witnessed checkpoint; authority audit records preserve the
  resolved disposition, canonical resource subject, and case linkage.
- Independent final review: approved with no findings.
- Task 6 — M0f `sea-forge migrate`: lossless v0.1 → v0.2 case layout migration.
  `sea-forge migrate` enumerates legacy source files (excluding views/append-only
  `authority/` and `capabilities.jsonl`), emits `legacy_import` genesis ledger
  entries with byte sha256/path/size/legacy record version, signs an initial
  global checkpoint, and relocates run directories under
  `.sea-forge/cases/<case_id>/runs/<run_id>/` and case files to
  `.sea-forge/cases/<case_id>/case.json` without rewriting record bytes. Migration
  is idempotence-guarded by `.sea-forge/migration.json`. `LedgerStream::verify`
  verifies each `legacy_import` file against its recorded hash and size, so a
  corrupted legacy file fails `ledger verify`. `inspect` finds runs in both v0.1
  flat and v0.2 nested layouts and reports `legacy_digest_only` for migrated
  records. Conformance tests verify byte hashes committed, IDs resolvable, ledger
  verify green, re-migration refused, corruption detected, and inspect assurance
  labeling.

## Tasks 1–4 Specification Reconciliation

- Substrate map, reconciliation matrix, M0 gate evidence, and deferrals:
  `.agents/reports/2026-07-12-tasks-1-4-spec-reconciliation.md`.
- The standalone proposed patch, complete patched specification, and patch guide
  were not found in the repository or nearby project tree, so no `git apply` or
  `git apply --check` was possible. The proposals in the user request were
  evaluated manually against the code.
- `spec-full.md` now defines deterministic verdict resolution with `allow` as
  least restrictive, an opaque exact-action/context authorization boundary,
  canonical ledger-before-view failure semantics, M0-G1–G6, completion-claim
  levels, and cumulative release boundaries.
- The implementation plan assigns those implementation and proof obligations to
  Task 5 without prescribing an `AuthorizedAction` type or a parallel authority
  or persistence system.
- Public runtime execution and sandbox materialization now consume opaque,
  one-use, context-bound authority grants; direct ungranted effects do not compile.

## Blockers

- `devbox run -- just check` reaches `cargo deny` and fails because
  `domainforge-core 0.13.0` depends on `xxhash-rust 0.8.16` under `BSL-1.0`,
  which `deny.toml` does not allow. Tests, formatting, clippy, context-check,
  P1–P4b, and no-async-kernel pass. Changing the license allowlist requires
  explicit approval and is not folded into Task 5.

## Decisions

- Pipeline moved to `sea-forge-cli` (not kept in `sea-forge-core`) because keeping
  it in core would create a circular dependency once authority/runtime/sandbox
  moved to their own crates. This matches the plan’s allowance: “pipeline.rs stays
  in core or moves to cli — keep wherever the diff is smallest.”
- `sea-forge-extension` starts with a minimal lib.rs re-exporting descriptor
  types from `sea-forge-core`; it will gain registry logic in Task 4 (M0d).
- The `timed_out_child_pid_is_no_longer_alive` test’s child-process test-name
  argument was updated to match the new crate-local test path; this is a required
  mechanical reference update, not a logic or proof change.
