# Current Status

Updated: 2026-07-17

## Objective

Implement `.agents/plans/2026-07-11-spec-full-implementation.md`: execute the
SEA Forge full-system plan (spec-full.md M0–M8) task by task, committing after
each milestone gate. Work happens on the `full-spec` branch; `ci-cd` was merged
to `main` and pushed to origin first.

## Worktree State

On branch `full-spec`. The mechanical crate graduation (Task 1 / M0a) is complete
and green. The minimum kernel tests (35 tests) and P1–P4b pass unchanged. No
behavior changes were made during the split.

## Changed Files

- `Cargo.toml` — added 10 new kernel crate members to workspace.
- `justfile` — added `no-async-kernel` recipe; wired into `ci`.
- `Cargo.lock` — refreshed by the workspace expansion.
- `crates/sea-forge-core/src/lib.rs` — reduced to ids/types/errors + `RECORD_VERSION`.
- `crates/sea-forge-cli/src/main.rs` — added `mod pipeline`.
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

- Tasks 2–17 from the implementation plan (M0b through M8 + DoD sweep).
- Continue milestone-ordered implementation, committing after each task.
- Stale stash `stash@{0}` remains from the initial workspace cleanup; will drop
  once the log-file reset is no longer a safety-net concern.

## Verification

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- Task 4 — M0d extension ABI and registry: `crates/sea-forge-extension`
  implements `ExtensionRegistry` (load/save/register/import/adopt), `RegistryEntry`
  with `TrustLevel` and `ExtensionStatus`, `ExtensionInstallRecord` with
  `Compatibility`, `ProjectionAdapter` trait, `ProjectionRecord`, descriptor
  validation, and 8 unit tests covering descriptor validation, empty-registry,
  built-in registration, import-starts-disabled, adopt-makes-active, and
  authority-surface→policy-surface mapping.

- `cargo test --workspace --all-features --locked`: 71 tests passed (35 existing + 23 ledger + 5 domainforge + 8 extension), 0 failed.
- `just proof`: P1–P4b passed.
- `just no-async-kernel`: passed.
- `cargo build --workspace --all-targets --locked`: passed.

## Blockers

- None.

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
