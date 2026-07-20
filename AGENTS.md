# SEA Forge Agent Guide

Build SEA Forge as a governed capability-execution kernel. Preserve its core
invariant: every side effect is authorized before execution, then traced,
evidenced, settled, and recorded in capability memory. A successful process is
not a successful run unless settlement accepts the declared outcome.

## Commands

The Rust workspace and Shell-SPEC foundation are implemented; the governed
minimum kernel is not. Prefer narrow feedback before workspace-wide checks.

```sh
# Fast feedback
cargo fmt --all -- --check
cargo check -p sea-forge-core
cargo test -p sea-forge-core <test_name>
cargo test -p sea-forge-cli <test_name>

# Required before declaring a minimum-slice change complete
devbox run -- just context-check
devbox run -- just check
devbox run -- just test
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

Work from the requested outcome or job-to-be-done, not the literal wording alone.
Surface hidden assumptions, unnecessary scope, contradictions, and debt in the
request; do not silently turn them into implementation debt.

## Project Map

- `.agents/specs/spec-minimum.md`: normative minimum vertical slice and build order.
- `.agents/specs/spec-full.md`: additive full-system milestones and conformance gates.
- `.agents/specs/Shell-SPEC.md`: implemented foundation, tools, secrets, and CI.
- `.agents/plans/`: implementation plans; keep them aligned with the specs.
- `.agents/CURRENT_STATUS.md`: resumable handoff for active work.
- `.agents/OBSERVED_DEBT.md`: out-of-scope problems discovered while working.
- `.agents/LESSONS.md`: durable project lessons that prevent future mistakes.
- `.agents/OPEN_QUESTIONS.md`: unresolved decisions that require user judgment.
- `crates/`: Rust workspace; preserve the boundaries in the applicable spec.
- `.sea-forge/`: runtime output only; never use it as checked-in source code.
- `.ua/`: Understand-Anything knowledge graph of the repo; rebuildable projection.
  See "Codebase Knowledge Graph" below.

As the full system graduates modules into crates, preserve the crate boundaries
in full spec §6.2. Kernel crates remain synchronous. Tokio belongs only in
`sea-forge-server` or an explicitly isolated runtime adapter.

## Codebase Knowledge Graph

`.ua/knowledge-graph.json` is an Understand-Anything graph of this repo: every
file, function, class, and dependency as a queryable node/edge graph, grouped
into architectural layers with a guided tour. It is a rebuildable projection, not
source of truth — never hand-edit it; regenerate via the `understand` skill.

Prefer the graph over reading code blind when orienting. Load the `understand`,
`understand-chat`, `understand-explain`, `understand-diff`, or
`understand-dashboard` skills to ask about structure, trace a call path, scope a
change's blast radius, or open the visual dashboard. It does not replace reading
the specs or the actual code before a change.

The graph stays current via `.githooks/post-commit`: non-source commits refresh
`.ua/meta.json` at zero token cost; source commits emit a trigger the in-session
agent acts on to run the incremental update. `.ua/intermediate/` and `.ua/tmp/`
are scratch; `.ua/config.json` carries `autoUpdate` and output language.

## Architecture Invariants

- Normalize intent into typed operations; never interpolate it into shell or paths.
- Decide all authority requests before side effects. Default deny; use one authority
  fabric, and keep policy out of sandbox/runtime modules.
- Authority and isolation are distinct controls; never weaken either by fallback.
- All outcomes, including denial and failure, leave complete governed records.
- Settlement evaluates evidence, not process exit alone.
- JSONL truth and run history are append-only; views are rebuildable projections.
- Generated work products retain the identity and governance metadata the specs require.

## Rust Conventions

- Use stable Rust, edition 2021, and `rustfmt` defaults.
- Model domain states and errors with typed enums.
- Use serde `snake_case` for persisted enums and derive the traits required by
  minimum spec §7.
- Return typed errors with machine-readable classes; do not panic for expected
  input, policy, I/O, execution, or validation failures.
- Avoid `unsafe` and new dependencies unless required and justified. Keep the
  minimum kernel free of network and async dependencies.
- Follow the closest current implementation and test, not roadmap pseudocode.

## Testing and Evidence

- Use test-driven development for logic, bug fixes, state transitions, and
  behavior changes: write a failing focused test, implement, then refactor.
- Cover allow, deny, escalate, malformed input, timeout, nonzero exit, and false
  success. Never delete or weaken a failing conformance test to make a change pass.
- Assert absence of side effects on denied paths, not only the returned verdict.
- Keep minimum tests offline. Report unsupported platform tests as skipped.

## Workflow

- Investigate before asking. Use `rg --files` for path discovery, `rg` for text
  search, and targeted line-range reads for file contents; run independent,
  complementary searches in parallel. Stop when evidence is sufficient and cite
  relevant files and lines. Search specs, tests, history, and authoritative online
  sources when relevant; do not ask for facts that can be discovered safely.
- Ask only when user judgment, authority, or missing intent materially changes the
  outcome. Keep questions concise and include a recommendation with its trade-off.
- Read every file before editing it and inspect one nearby pattern plus its tests.
- Keep changes small and milestone-scoped. Compile and test each dependency-ordered
  step from the minimum spec Appendix A before starting the next.
- Preserve user changes and unrelated worktree edits. Do not reformat unrelated files.
- Update specs or an ADR when changing a public contract, persisted schema,
  architecture boundary, proof level, or implementation-defined behavior.
- Review diffs for correctness, security, compatibility, needless complexity, and coherence to the overall system design.
- Do many small commits; DO NOT push, open a pull request, deploy, or publish unless the user asks.

## Local Agent Memory

Use `.agents/` for repository-local memory, not chat transcripts or routine work
logs. Read the relevant files before work and keep entries concise, dated, and
evidence-linked. Update an existing entry instead of duplicating it.

- `CURRENT_STATUS.md` is the handoff. Keep the active objective, completed work,
  changed files, verification results, remaining steps, blockers, and decisions
  accurate enough that a new agent can continue without reconstructing the task.
- `OBSERVED_DEBT.md` records concrete debt, gaps, risks, or defects noticed during
  work but outside the current scope. Include location/evidence, impact, and a
  suggested next move. Do not expand the task to fix it without authorization.
- `LESSONS.md` contains only verified, project-specific learning that will improve
  future agent work or prevent an easy/repeated mistake. Do not add generic advice,
  speculation, one-off debugging chronology, or facts obvious from current code.
- `OPEN_QUESTIONS.md` contains only unresolved choices that cannot be answered by
  repository or external research and genuinely require user judgment.

At handoff, refresh `CURRENT_STATUS.md`, capture qualifying debt or lessons, and
remove or resolve stale entries. Tell the user when a directory has distinct
commands, architecture, risks, generated-file rules, or conventions that warrant
a scoped local `AGENTS.md`; recommend its scope and key rules rather than silently
creating instruction sprawl.
Run `just context-check` before handoff; its required status sections and
change-coupling rules are tool-agnostic and also run in CI.

## Safety Boundaries

- Validate workspace-relative paths with the specified safe-join algorithm.
- Use argv-based process execution; never invoke a shell.
- Give children a minimal explicit environment and enforce timeouts.
- Keep secrets, credentials, private keys, `.env` contents, and sensitive payloads
  out of code, fixtures, logs, traces, evidence, and agent instructions.
- Always use portable paths and avoid OS-specific features unless explicitly required. Do not assume
  a specific shell, filesystem, or OS behavior.

Ask first:

- Adding or upgrading dependencies; changing persisted schemas, ID grammar, policy
  precedence, exit codes, or public interfaces; editing CI/deployment configuration;
  deleting files; or expanding work beyond the current spec milestone.

Never:

- Write outside authorized roots, inherit the parent environment wholesale, weaken
  sandbox class, bypass authority, treat unknown operations as allowed, or silently
  fall back when a policy engine or jail is unavailable.
- Never hand-edit generated zones; change the source or generator and regenerate.
- Commit runtime output under `.sea-forge/`, vendored/generated dependencies, secrets,
  or credentials.

## Codebase Search

When the edit location is unknown, localize before reading broadly. Use `rg --files` or glob patterns to find likely files, then `rg` with exact symbols, errors, config keys, domain terms, and naming variants. Search likely implementations, callers, tests, and configuration in parallel. Prefer file-only results first, then inspect only the strongest matches.

Read narrow ranges with `sed` or `awk`, usually 30–80 lines around a match, and avoid rereading content already in context. Cap noisy output, change search terms when results are weak, and stop once the implementation, execution path, tests, and relevant dependencies are identified. Use Understand Anything when this would otherwise require a long multi-file search chain.

## Understand Anything

Use Understand Anything when uncertainty about location, relationships, execution flow, or change impact would otherwise require broad exploratory reading. Do not use it when direct search and local inspection are sufficient.

Use the smallest operation that resolves the uncertainty:

- `/understand-chat <question>` — semantically locate relevant code, responsibilities, flows, or relationships.
- `/understand-explain <path-or-symbol>` — deeply explain one file, function, class, module, or component.
- `/understand-diff` — identify dependencies, affected components, and likely ripple effects of current changes.
- `/understand <directory>` — analyze or refresh only the relevant subsystem when its graph data is missing or stale.
- `/understand` — analyze the whole repository only when no usable graph exists or repository-wide refresh is genuinely required.

Examples:

```text
/understand-chat Where is authorization enforced for API requests?
/understand-chat What components participate in compiling a .sea file?
/understand-explain crates/compiler/src/lowering.rs
/understand-diff
/understand packages/application-compiler
```

Treat results as navigation and comprehension aids, not authoritative proof. Verify material conclusions against the source code, tests, configuration, and runtime behavior.

Do not generate dashboards, onboarding guides, domain views, or repository-wide analyses unless the task specifically requires them. Do not load the full graph into context when a focused query or explanation is sufficient.

On Codex, use `$understand`, `$understand-chat`, `$understand-explain`, and `$understand-diff` instead of slash-prefixed commands.
