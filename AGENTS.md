# SEA Forge Agent Guide

Build SEA Forge as a governed capability-execution kernel. Authorize every side
effect before execution, then trace, evidence, settle, and record it. A process
exit is never success unless settlement accepts the declared outcome.

## Start Here

1. Follow the user's request and the nearest `AGENTS.md`; a nested guide governs
   its subtree.
2. Read `.agents/CURRENT_STATUS.md` before continuing active work.
3. Read the governing spec before changing behavior:
   - `.agents/specs/spec-minimum.md` defines the minimum kernel, persisted
     records, ID grammar, lifecycle, authority hashes, and proofs P1–P4b.
   - `.agents/specs/spec-full.md` adds milestones without weakening the minimum.
   - `.agents/specs/Shell-SPEC.md` defines tools, commands, secrets, and CI.
   - Feature-specific `.agents/specs/spec-*.md` files govern named subsystems.
4. If instructions and a normative spec conflict, stop and report the conflict.

Surface contradictions, hidden assumptions, unnecessary scope, and debt instead
of encoding them in the implementation.

## Commands

Run `just` from the repository root. It is the project's command-line interface
and prints the grouped recipe list when called without a recipe.

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

### Verification recipes

| Scope | Command |
| --- | --- |
| Agent handoff | `just context-check` |
| Rust quality | `just check` |
| Rust tests | `just test` |
| Current-platform CI union | `just ci` |
| Minimum proofs P1–P4b | `just proof` |
| Workbench | `just workbench-check` |

`just ci` runs context, formatting, lint, typecheck, security, tests, the
dependency boundary, and build on the current platform. It does **not** run
`just proof`, Workbench gates, or another platform's tests. Run the applicable
extra gates and report every skipped platform test with its reason.
Run the current milestone gate named by the governing feature spec. Use
`just no-async-kernel` as a focused check when dependency boundaries change.

For Workbench work, follow `workbench/AGENTS.md`. Useful root recipes include
`workbench-contracts-gate`, `workbench-tauri-test`, `workbench-package`,
`workbench-package-inventory`, `workbench-dev-up`/`-down`, and
`workbench-storybook-up`/`-down`. Packaging and inventory apply only to release
or packaging tasks.

Do not run destructive, publishing, or external-write recipes without approval:
`just pr` pushes and opens a pull request, and `publish-bootstrap` publishes a
crate. `clean` deletes build output. Use `secrets-*` only for an explicitly
requested secrets task; never expose decrypted values. `integration` currently
fails closed because no integrations are declared.

## Project Map

- `crates/`: Rust workspace; preserve the crate boundaries in the specs.
- `workbench/`: separate Bun and Tauri workspaces with a nested `AGENTS.md`.
- `.agents/specs/`: normative behavior and conformance gates.
- `.agents/plans/`: implementation plans aligned with the specs.
- `.agents/{CURRENT_STATUS,OBSERVED_DEBT,LESSONS,OPEN_QUESTIONS}.md`: durable
  handoff, debt, verified lessons, and owner decisions.
- `.sea-forge/`: gitignored runtime output, never source.
- `.ua/`: generated Understand-Anything projection; never hand-edit it.
- `justfile`: canonical command surface. Do not duplicate recipe bodies here.

## Architecture and Code

- Normalize intent into typed operations; never interpolate intent into shell
  commands or paths.
- Decide all authority requests before side effects. Default deny, use one
  authority fabric, and keep policy out of sandbox and runtime mechanisms.
- Keep authority and isolation independent; never weaken either by fallback.
- Record complete outcomes for allow, denial, escalation, failure, and timeout.
- Settle from evidence, not exit status. Keep JSONL truth and history append-only,
  views rebuildable, and required identity and governance metadata intact.
- Validate workspace-relative paths with the specified safe-join algorithm.
  When implementing child processes, use argv execution, a minimal explicit
  environment, and enforced timeouts.
- Keep kernel crates synchronous. Only `sea-forge-agent` and
  `sea-forge-server` may contain async-runtime or HTTP-client dependencies.
- Use stable Rust 2021, rustfmt defaults, typed enums, typed errors, and serde
  `snake_case` for persisted enums. Avoid `unsafe`.
- Prefer domain concepts and canonical vocabulary. Name types for identity and
  methods for behavior; make illegal states apparent in names and types.
- Prefer explicit interfaces, local reasoning, observable behavior, and
  abstractions that represent real domain concepts. Hide mechanisms behind
  capabilities; keep domain objects free of infrastructure concerns.
- Follow current implementations and tests, not roadmap pseudocode. Never
  hand-edit generated zones; change their source or generator and regenerate.

## Testing

Use test-driven development for logic, fixes, state transitions, and behavior:
write a focused failing test, implement, then refactor. Cover allow, deny,
escalate, malformed input, timeout, nonzero exit, and false success where
applicable. Denied paths must prove the absence of side effects. Keep minimum
tests offline, and never delete or weaken a conformance test to make a gate pass.

## Workflow and Permissions

- Investigate before asking. Use `rg --files`, then `rg`, and read narrow source,
  test, spec, and history ranges until evidence is sufficient.
- Use `$understand-chat`, `$understand-explain`, or `$understand-diff` only when
  relationships or blast radius would otherwise require broad reading. Verify
  graph results against source and tests.
- Read every file before editing it; inspect one nearby pattern and its tests.
- Keep changes small and milestone-scoped. Preserve unrelated worktree changes
  and avoid unrelated formatting.
- Ask before adding or upgrading dependencies; changing persisted schemas, ID
  grammar, policy precedence, exit codes, public interfaces, CI, or deployment;
  deleting files; or expanding the current milestone.
- Update a spec or ADR when changing a public contract, persisted schema,
  architecture boundary, proof level, or implementation-defined behavior.
- Review diffs for correctness, security, compatibility, test strength,
  needless complexity, and architectural coherence.
- Keep secrets, credentials, private keys, `.env` contents, and sensitive
  payloads out of code, fixtures, logs, traces, evidence, and instructions.
- Never bypass authority, weaken a sandbox, allow unknown operations, or
  silently fall back when policy or jail infrastructure is unavailable.
- Make small focused commits when the task calls for commits. Do not push, open
  a pull request, deploy, or publish unless the user asks.

## Handoff

Keep `.agents/CURRENT_STATUS.md` resumable: objective, worktree state, changed
files, completed work, verification, remaining steps, blockers, and decisions.
Record only concrete out-of-scope debt in `OBSERVED_DEBT.md`, verified reusable
project lessons in `LESSONS.md`, and unresolved owner choices in
`OPEN_QUESTIONS.md`. Update existing entries instead of duplicating them. Run
`just context-check` before handoff.
