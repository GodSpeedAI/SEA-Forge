# Current Status

Updated: 2026-07-11

## Objective

Correct the draft full-system architecture so SEA Forge governs a world defined
by real DomainForge `.sea` semantics without changing the implemented v0.1
minimum kernel.

## Worktree State

The minimum implementation is merged and pushed on `main`. Two local
documentation commits contain the Genesis alignment; they have not been pushed.
The DomainForge architecture correction is documentation-only and builds on
those commits. Unrelated untracked
`.agents/plans/upgraded-sea.code-workspace` and `.omc/` content remains
untouched.

## Changed Files

- `.agents/specs/spec-full.md`
- `docs/decisions/ADR-001-domainforge-semantic-boundary.md`
- `ARCHITECTURE.md`
- `README.md`
- `.agents/CURRENT_STATUS.md`

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
- Clarified that minimum `capabilities.jsonl` is capability-attempt memory and
  that v0.1 settlement is kernel-local verification, not independently declared
  strong settlement.
- Added the full-spec `SettlementAuthority`/`SettlementDeclaration` boundary,
  reliability weighting, declarer standing and independence, promotion and
  contraction rules, manufactured-settlement threats, and M4a gates.
- Audited `types.rs`, `settlement.rs`, `capability.rs`, and `pipeline.rs`; the
  clarification matches current v0.1 behavior and requires no Rust/schema change.
- Inspected the sibling DomainForge repository and confirmed that
  `domainforge-core` is the canonical Rust library for SEA grammar, AST,
  semantic Graph, validation, authority evaluation, and in-memory projections.
- Corrected the full spec's direction from “DomainForge outputs `.sea`” to
  “authored or governed-synthesized `.sea` → DomainForge semantic model →
  validated projections.”
- Added the `sea-forge-domainforge` M0/M2/M5 boundary, `DomainModelRef`,
  fail-closed authority normalization, direct-library/no-direct-side-effect
  rules, milestone gates, and proof requirements.
- Added ADR-001 and aligned the architecture map and README. The v0.1 JSON
  `model.sea` remains unchanged and is now clearly labeled as a lifecycle stub,
  not a DomainForge model.

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
- Genesis alignment: `devbox run -- just context-check`, `git diff --check`, and
  unchanged P1-P4b passed on 2026-07-11. No Rust changed, so the prior 35-test
  implementation result remains the applicable runtime verification.
- DomainForge documentation correction: `git diff --check` and
  `devbox run -- just context-check` passed on 2026-07-11. No Rust, manifest,
  persisted v0.1 schema, or executable behavior changed, so code gates were not
  rerun.

## Remaining

None for this documentation correction. The full v0.2 specification remains
draft and unimplemented by design; its M0–M8 checklist is the roadmap, not
unfinished minimum work.

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
- Preserve v0.1 records and proofs; strong settlement is an additive full-spec
  declaration outside immutable run directories.
- Treat raw accepted/rejected/escalated counts as observations. Only qualifying,
  independently declared, reliability-weighted outcomes can promote capability.
- DomainForge owns `.sea` syntax, semantic graph construction, concept identity,
  validation, and deterministic projections. SEA Forge owns final authority,
  isolation, side effects, evidence, and settlement.
- Integrate `domainforge-core` through a side-effect-free first-party adapter at
  M0; bind plans to its semantic model at M2; add governed synthesis and
  in-memory projections at M5. Do not add the dependency to v0.1 crates.
