# SEA Forge Agent Guide

Build SEA Forge as a governed capability-execution kernel. Authorize every side
effect before execution, then trace, evidence, settle, and record it. A process
exit is never success unless settlement accepts the declared outcome.

## Start Here

1. Follow the user's request and the nearest `AGENTS.md`; a nested guide governs
   its subtree.
2. Read `.agents/current_status.yml` before continuing active work.
3. Read the governing spec before changing behavior:

   * `.agents/specs/spec-minimum.md` defines the minimum kernel, persisted
     records, ID grammar, lifecycle, authority hashes, and proofs P1–P4b.
   * `.agents/specs/spec-full.md` adds milestones without weakening the minimum.
   * `.agents/specs/Shell-SPEC.md` defines tools, commands, secrets, and CI.
   * Feature-specific `.agents/specs/spec-*.md` files govern named subsystems.
4. If instructions and a normative spec conflict, stop and report the conflict.

Surface contradictions, hidden assumptions, unnecessary scope, and debt instead
of encoding them in the implementation.

## Commands

Run `just` from the repository root. It is the project's canonical
human-and-agent command-line interface and prints the grouped recipe list when
called without a recipe.

Prefer an existing `just` recipe over an equivalent raw Cargo command. Raw Cargo
commands remain appropriate for diagnostics or when no repository recipe exists,
but they must not silently bypass a stronger project-specific gate.

### Setup and fast feedback

```sh
just setup                              # pinned tools and dependencies
just doctor                             # JSONL environment diagnostics
just sync                               # after toolchain or lockfile changes
just check-fast                         # context + fmt + typecheck
just fmt-check                          # verify formatting
just crate-check sea-forge-core
just crate-test sea-forge-core <test_name>
just crate-test sea-forge-cli <test_name>
```

Prefer the narrowest relevant crate and test while iterating.

For a bounded Rust change, use this progression unless the governing spec
requires something stronger earlier:

```text
rust-analyzer semantics
→ targeted source/test/spec inspection
→ just crate-check <crate>
→ just crate-test <crate> <test>
→ just check-fast
→ broader relevant tests
→ milestone/task gates
→ just ci / just proof when required
```

Do not repeatedly pay workspace-wide verification cost after small localized
edits when a crate-scoped check settles the immediate question.

### Verification recipes

| Scope                     | Command                |
| ------------------------- | ---------------------- |
| Agent handoff             | `just context-check`   |
| Rust quality              | `just check`           |
| Rust tests                | `just test`            |
| Current-platform CI union | `just ci`              |
| Minimum proofs P1–P4b     | `just proof`           |
| Workbench                 | `just workbench-check` |

`just ci` runs context, formatting, lint, typecheck, security, tests, the
dependency boundary, and build on the current platform. It does **not** run
`just proof`, Workbench gates, or another platform's tests. Run the applicable
extra gates and report every skipped platform test with its reason.

Run the current milestone gate named by the governing feature spec. Use
`just no-async-kernel` as a focused check when dependency boundaries change.

A fast inner loop never substitutes for a required milestone gate, proof,
cross-platform check, evidence requirement, or settlement criterion. Optimization
changes when verification cost is paid, not what must ultimately be proven.

For Workbench work, follow `workbench/AGENTS.md`. Useful root recipes include
`workbench-contracts-gate`, `workbench-tauri-test`, `workbench-package`,
`workbench-package-inventory`, `workbench-dev-up`/`-down`, and
`workbench-storybook-up`/`-down`. Packaging and inventory apply only to release
or packaging tasks.

For frontend development and evidence, use the recipes for their declared
scope:

```sh
just workbench-dev-up                 # Vite renderer at http://localhost:1420
just workbench-dev-down               # stop that Vite server
just workbench-e2e-agent-browser      # renderer + accessibility; no Tauri IPC mock
just workbench-e2e-case-authoring     # case-authoring proof journeys; mocked Tauri IPC
just workbench-e2e-real [filter]      # packaged Tauri/WebKit + real server/SFWP
```

The Vite development server has no native Tauri bridge and must therefore show
the fail-closed unavailable state for governed data. The installed Playwright
suite and `workbench-e2e-case-authoring` remain mocked-IPC evidence only; do
not use either for an integrated Tauri/server claim. `workbench-e2e-real`
requires the documented Linux native driver prerequisites and packages the
application itself.

Do not run destructive, publishing, or external-write recipes without approval:
`just pr` pushes and opens a pull request, and `publish-bootstrap` publishes a
crate. `clean` deletes build output. Use `secrets-*` only for an explicitly
requested secrets task; never expose decrypted values. `integration` currently
fails closed because no integrations are declared.

### Rust workstation tooling

The development workstation may provide shared tools globally, including:

* `sccache`
* `mold`
* `cargo-nextest`
* `cargo-mutants`
* `cargo-deny`
* `cargo-llvm-cov`
* `just`
* `clippy`
* `rustfmt`

Global availability is capability, not project policy.

The repository's toolchain pin, `justfile`, specs, configuration, CI, and
verification contracts remain authoritative. Do not add project-local
installation or configuration for globally supplied developer tooling merely
because it exists on the workstation.

Do not modify the project's Rust toolchain, nextest configuration, dependency
policy, mutation exclusions, coverage thresholds, tracing/evidence
configuration, CI, or proof gates merely to normalize SEA Forge to a generic
workstation baseline. Such changes require project-specific justification.

When build performance itself needs investigation, prefer instrumented
diagnostics such as Cargo timings and `sccache --show-stats` over speculative
configuration changes.

## Project Map

* `crates/`: Rust workspace; preserve the crate boundaries in the specs.
* `workbench/`: separate Bun and Tauri workspaces with a nested `AGENTS.md`.
* `.agents/specs/`: normative behavior and conformance gates.
* `.agents/plans/`: implementation plans aligned with the specs.
* `.agents/{OBSERVED_DEBT,LESSONS,OPEN_QUESTIONS}.md`, `current_status.yml`: durable
  handoff, debt, verified lessons, and owner decisions.
* `.sea-forge/`: gitignored runtime output, never source.
* `.ua/`: generated Understand-Anything projection; never hand-edit it.
* `justfile`: canonical command surface. Do not duplicate recipe bodies here.

## Architecture and Code

* Normalize intent into typed operations; never interpolate intent into shell
  commands or paths.
* Decide all authority requests before side effects. Default deny, use one
  authority fabric, and keep policy out of sandbox and runtime mechanisms.
* Keep authority and isolation independent; never weaken either by fallback.
* Record complete outcomes for allow, denial, escalation, failure, and timeout.
* Settle from evidence, not exit status. Keep JSONL truth and history append-only,
  views rebuildable, and required identity and governance metadata intact.
* Validate workspace-relative paths with the specified safe-join algorithm.
  When implementing child processes, use argv execution, a minimal explicit
  environment, and enforced timeouts.
* Keep kernel crates synchronous. Only `sea-forge-agent` and
  `sea-forge-server` may contain async-runtime or HTTP-client dependencies.
* Use the repository's pinned stable Rust toolchain, Rust 2021 edition, rustfmt
  defaults, typed enums, typed errors, and serde `snake_case` for persisted
  enums. Avoid `unsafe`.
* Prefer domain concepts and canonical vocabulary. Name types for identity and
  methods for behavior; make illegal states apparent in names and types.
* Prefer explicit interfaces, local reasoning, observable behavior, and
  abstractions that represent real domain concepts. Hide mechanisms behind
  capabilities; keep domain objects free of infrastructure concerns.
* Follow current implementations and tests, not roadmap pseudocode. Never
  hand-edit generated zones; change their source or generator and regenerate.

## Testing

Use test-driven development for logic, fixes, state transitions, and behavior:
write a focused failing test, implement, then refactor.

Cover allow, deny, escalate, malformed input, timeout, nonzero exit, and false
success where applicable. Denied paths must prove the absence of side effects.
Keep minimum tests offline, and never delete or weaken a conformance test to
make a gate pass.

### Test execution discipline

During implementation:

1. Identify the observable behavior that distinguishes correct from incorrect.
2. Add or identify the focused test first when practical.
3. Confirm it fails for the expected reason.
4. Implement the smallest coherent change.
5. Run the narrowest relevant crate/test recipe immediately.
6. Broaden verification only as the affected surface expands.
7. Run every required milestone, CI, proof, or settlement gate before claiming
   the work complete.

Do not use full-workspace compilation as a substitute for understanding the
affected boundary.

### Expensive evidence

Mutation testing and coverage are explicit evidence-generation operations, not
ordinary edit/check-loop steps.

When repository recipes exist, prefer those recipes. Otherwise:

```sh
cargo mutants
cargo llvm-cov nextest --workspace
```

Use mutation testing when required to demonstrate that tests discriminate
correct from incorrect behavior. Coverage measures execution reach; it does not
prove behavioral discrimination or correctness.

Do not add mutation or coverage to every development iteration unless the
governing spec explicitly requires it.

`cargo-deny` remains the repository's dependency-policy instrument; use the
canonical project recipe when one exists.

## Workflow and Permissions

* Investigate before asking. Use `rg --files`, then `rg`, and read narrow source,
  test, spec, and history ranges until evidence is sufficient.
* Use `$understand-chat`, `$understand-explain`, or `$understand-diff` only when
  relationships or blast radius would otherwise require broad reading. Verify
  graph results against source and tests.
* Prefer `rust-analyzer` semantics for Rust structural questions before
  grep-and-recompile loops.
* Read every file before editing it; inspect one nearby pattern and its tests.
* Keep changes small and milestone-scoped. Preserve unrelated worktree changes
  and avoid unrelated formatting.
* Ask before adding or upgrading dependencies; changing persisted schemas, ID
  grammar, policy precedence, exit codes, public interfaces, CI, or deployment;
  deleting files; or expanding the current milestone.
* Update a spec or ADR when changing a public contract, persisted schema,
  architecture boundary, proof level, or implementation-defined behavior.
* Review diffs for correctness, security, compatibility, test strength,
  needless complexity, and architectural coherence.
* Keep secrets, credentials, private keys, `.env` contents, and sensitive
  payloads out of code, fixtures, logs, traces, evidence, and instructions.
* Never bypass authority, weaken a sandbox, allow unknown operations, or
  silently fall back when policy or jail infrastructure is unavailable.
* Make small focused commits when the task calls for commits. Do not push, open
  a pull request, deploy, or publish unless the user asks.

## Handoff

Keep `.agents/current_status.yml` resumable: objective, worktree state, changed
files, completed work, verification, remaining steps, blockers, and decisions.

Record only concrete out-of-scope debt in `OBSERVED_DEBT.md`, verified reusable
project lessons in `LESSONS.md`, and unresolved owner choices in
`OPEN_QUESTIONS.md`. Update existing entries instead of duplicating them.

Run `just context-check` before handoff.

### Build-lock and artifact discipline

Only one agent may perform compile-heavy work against the shared build directory
at a time. Do not launch overlapping Cargo builds merely to reduce wall-clock
time.

Preserve hot incremental build state. Do not use routine `cargo clean` as a
response to slow compilation or ordinary test failure.

Watch build-artifact growth when compile-heavy work is active:

```sh
du -sh target
```

If `target/` exceeds the project's documented build-artifact ceiling, first
determine what is consuming space and whether an active build owns those
artifacts. Remove stale artifacts before destroying useful active build state.

Use a full `cargo clean` only when:

* the project's documented build-cache policy explicitly requires it,
* stale/corrupt artifacts are supported by evidence,
* a true clean rebuild is part of the requested verification, or
* the configured artifact ceiling has been exceeded and narrower cleanup cannot
  safely recover sufficient space.

Never clean a target directory while another agent or process is compiling into
it.

Do not increase Cargo build jobs or test concurrency to compensate for slow
builds without evidence that memory and scheduling permit it. If a build or
test is OOM-killed, diagnose memory pressure and the failing process rather than
retrying with greater parallelism.

Global `sccache` and linker acceleration may reduce repeated build cost, but
they do not weaken the single-writer build-lock rule or any SEA Forge
verification requirement.
