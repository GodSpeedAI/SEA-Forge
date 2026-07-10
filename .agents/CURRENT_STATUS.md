# Current Status

Updated: 2026-07-10

## Objective

Implement `.agents/specs/spec-minimum.md` as the synchronous two-crate governed
kernel and keep the specification aligned with implementation-defined choices.

## Worktree State

Implementation is isolated at `.worktrees/feature-spec-minimum` on branch
`feature/spec-minimum`. Four atomic implementation/conformance commits contain
the completed slice. Nothing has been pushed.

## Changed Files

- Core lifecycle: `crates/sea-forge-core/src/{ids,types,errors,trace,evidence,authority,domain,planner,sandbox,runtime,settlement,capability,pipeline}.rs`.
- CLI: `crates/sea-forge-cli/src/main.rs`, `src/commands/`, and integration tests.
- Dependencies: workspace and crate `Cargo.toml` files plus `Cargo.lock`.
- Contract/docs: `.agents/specs/spec-minimum.md`, implementation plan,
  `README.md`, `justfile`, and repository-local memory.

## Completed

- Implemented plan → decide-all authority → workspace/runtime → trace/evidence
  → settlement → case close → semantic envelope/capability append.
- Added fail-closed canonical `AuthorityAction` coverage for executable,
  reserved, and unclassified surfaces while keeping planner operations closed.
- Added deterministic policy/request/identity hashing, hard boundary precedence,
  minimal child environments, argv-only execution, timeout termination, safe
  path joins, artifact hashing, and stable pre-mint identity.
- Added `run`, hidden `validate`, `recall`, and `inspect` CLI commands with the
  specified exit semantics.
- Replaced the placeholder proof recipe with executable P1–P4b checks.
- Added conformance tests covering accepted/denied/escalated outcomes, false
  success, nonzero exit, timeout, kill-9 JSONL durability, config/input failures,
  deterministic authority, reserved surfaces, symlink escape, minimal env,
  case state, recall, validator, inspection, and repeated artifact identity.
- Updated `spec-minimum.md` for `AuthorityAction`, final-write ordering,
  in-place execution-evidence hashing, atomic case closure, and the fixed
  conformance harness.

## Verification

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo build --workspace --all-features --locked`: passed.
- `cargo test --workspace --all-features --locked`: 35 passed, 0 failed,
  0 ignored.
- `just proof`: P1–P4b passed.
- `devbox run -- just context-check`: passed.
- `devbox run -- just check`: passed (Cargo Deny reported only unmatched
  allowlist warnings; advisories, bans, licenses, sources, and gitleaks passed).
- `devbox run -- just test`: 35 passed, 0 failed, 0 ignored.
- Final correctness, readability, architecture, security, and performance diff
  review: passed; no open findings.

## Remaining

None for the minimum slice. Branch integration is intentionally left to the
user; no push, pull request, deployment, or publication was performed.

## Blockers

None.

## Decisions

- Separate non-executable `AuthorityAction` from planner/runtime `Operation` so
  reserved and malformed surfaces can fail closed without becoming executable.
- Write `semantic-envelope.json` before `run_finished`; atomically close the case
  before `case_closed`; append the envelope to `capabilities.jsonl` as the final
  lifecycle commit write.
- Stream stdout/stderr directly to their final artifact paths and hash in place;
  copy workspace work products into artifacts exactly once.
