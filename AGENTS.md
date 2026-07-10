# SEA Forge Agent Guide

Build SEA Forge as a governed capability-execution kernel. Preserve its core
invariant: every side effect is authorized before execution, then traced,
evidenced, settled, and recorded in capability memory. A successful process is
not a successful run unless settlement accepts the declared outcome.

## Commands

The repository is currently specification-first; the Rust workspace may not
exist yet. Once its root `Cargo.toml` has been created, prefer narrow feedback
before workspace-wide checks.

```sh
# Fast feedback
cargo fmt --all -- --check
cargo check -p sea-forge-core
cargo test -p sea-forge-core <test_name>
cargo test -p sea-forge-cli <test_name>

# Required before declaring a minimum-slice change complete
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace
```

For end-to-end acceptance, run the proof commands in
`.agents/specs/spec-minimum.md` §12.2. During full-system work, also run the
current milestone gate in `.agents/specs/spec-full.md` §12 and §17. Never claim
a skipped platform test passed; report it as skipped with the reason.

## Source of Truth and Scope

1. Follow the user's current request.
2. Follow the nearest `AGENTS.md`; a nested file overrides this one in its tree.
3. Follow `.github/copilot-instructions.md` when present. If it conflicts with a
   spec, stop and surface the conflict rather than guessing.
4. Follow `.agents/specs/spec-minimum.md` for kernel types, lifecycle, ID grammar,
   record formats, authority hashes, audit shape, and fail-closed behavior.
5. Follow `.agents/specs/spec-full.md` only for additive extensions. It does not
   redefine the minimum kernel.

The minimum spec is the active implementation baseline and a prerequisite for
the full spec. Implement the two-crate synchronous slice first. Add full-system
milestones only in Appendix A order (M0 through M8), and keep minimum proofs
P1–P4b green after every milestone. Do not pull roadmap items forward without an
explicit requirement.

When a requirement is unclear, check the relevant spec section and existing
tests or code. If no precedent resolves a behavior that affects schemas,
authority, persistence, or public interfaces, ask instead of inventing it.

## Project Map

- `.agents/specs/spec-minimum.md`: normative minimum vertical slice and build order.
- `.agents/specs/spec-full.md`: additive full-system milestones and conformance gates.
- `.agents/plans/`: implementation plans; keep them aligned with the specs.
- `crates/sea-forge-core/`: minimum kernel types and synchronous lifecycle modules.
- `crates/sea-forge-cli/`: one-shot CLI, including `run`, `validate`, `inspect`,
  and `recall`.
- `.sea-forge/`: runtime output only; never use it as checked-in source code.

As the full system graduates modules into crates, preserve the crate boundaries
in full spec §6.2. Kernel crates remain synchronous. Tokio belongs only in
`sea-forge-server` or an explicitly isolated runtime adapter.

## Architecture Invariants

- Normalize intent into typed operations; never interpolate intent into shell,
  paths, or argv.
- Decide all authority requests before executing any operation. Default deny.
- Keep policy out of sandbox/runtime modules; they receive allowed execution
  requests, not policy files.
- Treat authority and sandboxing as separate controls. Neither replaces the other.
- Never add a second permission path for a new ingress or extension.
- Persist allow, deny, and escalate decisions with deterministic hashes, common
  audit fields, trace events, and evidence records.
- Denied, escalated, timed-out, and failed runs still settle and leave complete,
  cross-linked records. Denial is a governed outcome, not an internal crash.
- Keep JSON/JSONL records versioned and backward-readable when changes are additive.
  JSONL truth is append-only; indexes and capability/capital views are rebuildable
  projections, never competing sources of truth.
- Write each run directory once. A repeated intent creates a new run and does not
  mutate prior evidence.
- Settlement evaluates evidence and required artifacts independently of process
  exit status.
- Generated work products carry deterministic identity, provenance, ownership,
  license, review, maturity, case, run, and evidence metadata from creation.

## Rust Conventions

- Use stable Rust, edition 2021, and `rustfmt` defaults.
- Model domain states and errors with typed enums; avoid magic strings outside
  serialization boundaries.
- Use serde `snake_case` for persisted enums and derive the traits required by
  minimum spec §7.
- Keep functions small and deterministic where the spec requires replayability.
- Return typed errors with machine-readable classes; do not panic for expected
  input, policy, I/O, execution, or validation failures.
- Avoid `unsafe` unless the platform isolation boundary requires it and the change
  documents its invariant and adds focused tests.
- Do not add dependencies speculatively. Keep the kernel free of network and async
  dependencies during the minimum slice.
- Before adding a new pattern, find and follow the closest current implementation
  and test. Do not copy roadmap pseudocode over working repository conventions.

## Testing and Evidence

- Use test-driven development for logic, bug fixes, state transitions, and
  behavior changes: write a failing focused test, implement, then refactor.
- Put pure domain and path-safety cases in unit tests; exercise lifecycle,
  persistence, child processes, exit codes, and cross-links in integration tests.
- Cover allow, deny, escalate, malformed input, timeout, nonzero exit, and false
  success. Never delete or weaken a failing conformance test to make a change pass.
- Assert absence of side effects on denied paths, not only the returned verdict.
- Verify record deserialization, reference resolution, event order, artifact hashes,
  stable identities, and settlement basis.
- Tests must not require network access in the minimum slice. Gate OS-specific jail
  tests by platform and run real Landlock/Seatbelt tests where supported.

## Workflow

- Read every file before editing it and inspect one nearby pattern plus its tests.
- Keep changes small and milestone-scoped. Compile and test each dependency-ordered
  step from the minimum spec Appendix A before starting the next.
- Preserve user changes and unrelated worktree edits. Do not reformat unrelated files.
- Update specs or an ADR when changing a public contract, persisted schema,
  architecture boundary, proof level, or implementation-defined behavior.
- Review diffs for correctness, security, compatibility, and needless complexity.
- Do not commit, push, open a pull request, deploy, or publish unless the user asks.

## Safety Boundaries

Always:

- Validate workspace-relative paths with the specified safe-join algorithm.
- Use argv-based process execution; never invoke a shell.
- Give child processes a minimal explicit environment and enforce timeouts.
- Keep secrets, credentials, private keys, `.env` contents, and sensitive payloads
  out of code, fixtures, logs, traces, evidence, and agent instructions.

Ask first:

- Adding or upgrading dependencies; changing persisted schemas, ID grammar, policy
  precedence, exit codes, or public interfaces; editing CI/deployment configuration;
  deleting files; or expanding work beyond the current spec milestone.

Never:

- Write outside authorized roots, inherit the parent environment wholesale, weaken
  sandbox class, bypass authority, treat unknown operations as allowed, or silently
  fall back when a policy engine or jail is unavailable.
- Directly edit generated zones such as `src/gen`, AST, IR, manifests, or generated
  semantic fixtures. Change their authority source or generator and regenerate.
- Commit runtime output under `.sea-forge/`, vendored/generated dependencies, secrets,
  or credentials.
