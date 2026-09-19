# AGENTS.md

Durable operating contract for coding agents in SEA Forge. Keep this file project-specific, behavior-changing, and earned; task state belongs in `.agents/`, and detailed subsystem guidance belongs in scoped specs/docs or nested `AGENTS.md`.

## 1. Scope and Precedence

SEA Forge is a governed capability-execution kernel: authorize every side effect before execution, then trace, evidence, settle, and record it. Process exit is not success unless settlement accepts the declared outcome.

Before substantial work:

1. Read this file.
2. Read `.agents/CURRENT_STATUS.md` and `.agents/current_status.yml`.
3. Read the governing spec under `.agents/specs/`.
4. Inspect affected code, tests, and configuration.
5. Load additional docs only when needed.

Instruction precedence:

1. Runtime/system safety and user constraints.
2. Nearest applicable scoped `AGENTS.md`.
3. This root `AGENTS.md`.
4. Governing normative spec in `.agents/specs/`.
5. Active plan in `.agents/plans/`.
6. Current implementation and tests.
7. General engineering defaults.

If instructions and a normative spec conflict, stop and report the conflict. Surface contradictions, hidden assumptions, unnecessary scope, and debt rather than encoding them silently.

## 2. Instruction Topology & Routing

Instructions live at the narrowest scope that completely governs them:

* **`crates/` (`crates/AGENTS.md`)**: Governs all 22 Rust workspace crates. Owns the synchronous kernel boundary (19 synchronous kernel crates vs 2 async edge crates: `sea-forge-server` and `sea-forge-agent`), the package-scoped feedback loop (`just crate-check`, `just crate-test`), domain invariants, SFWP contract synchronization, and Rust build/toolchain discipline.
* **`workbench/` (`workbench/AGENTS.md`)**: Governs the desktop frontend (Bun workspace + Tauri 2 host + React 19 renderer). Owns renderer/host boundaries, generated schemas/tokens, and E2E testing evidence rules (`workbench-e2e-*`).
* **`.agents/` (`.agents/AGENTS.md`)**: Governs the durable agent workbench. Owns normative specs vs tactical plans, status handoff contracts verified by `just context-check` (`scripts/check-agent-context.sh`), and memory ledgers (`OBSERVED_DEBT.md`, `LESSONS.md`, `OPEN_QUESTIONS.md`).

## 3. Investigation and Retrieval

Investigate before asking. Use deterministic tools and repository utilities to settle mechanically answerable questions; do not spend reasoning effort inferring facts those tools can establish directly.

Use the cheapest tool that can settle the question:

* `graft map` for first-pass repository orientation.
* `graft ask "<question>" --source` for ranked architectural/behavioral context with source spans.
* `graft callers <symbol>` (`--direction out`, `--depth N`) for exact call-graph/blast-radius questions.
* `graft skeleton <file>` for signatures/spans without whole-file reads.
* `graft grep "<literal>"` for exhaustive literal matches across indexed files.
* `zvec_grep_search` or `zg` for semantic/conceptual discovery when wording/location is unknown.
* `rg --files` for inventory and `rg` for known paths, symbols, identifiers, literals, config keys, errors, or regexes.
* `rust-analyzer` semantics for Rust structural questions before grep-and-recompile loops.
* `$understand-chat`, `$understand-explain`, or `$understand-diff` only when relationships/blast radius would otherwise require broad reading; verify graph results against source/tests.

Use Graft/zvec to narrow the search space, then verify anchors with `rg` and read only relevant source/test/spec/history ranges. Ranked semantic results are not exhaustive. For unindexed files, fall back to `rg`/`grep`. If Graft truncates a span, open that exact range before finalizing.

## 4. Universal Change Boundaries

Always:

* Make the smallest effective change that fully satisfies the requested outcome; do not deliver MVP-like, partial, placeholder, or knowingly incomplete work unless explicitly requested.
* Read every file before editing it; inspect one nearby implementation pattern and its tests.
* Preserve unrelated worktree changes and avoid drive-by formatting/refactoring.
* Keep changes milestone-scoped and architecture-consistent.
* Change generators/source definitions instead of hand-editing generated zones.
* Run verification proportional to the claim.
* Record consequential state, evidence, debt, lessons, or open questions in `.agents/`.

Ask first before:

* adding/upgrading dependencies;
* changing persisted schemas, ID grammar, policy precedence, exit codes, public interfaces, CI, deployment, or architecture boundaries;
* deleting files or expanding the current milestone;
* destructive, publishing, external-write, or difficult-to-reverse operations.

Never:

* bypass authority, weaken a sandbox, allow unknown operations, or silently fall back when policy/jail infrastructure is unavailable;
* expose secrets, credentials, private keys, `.env` contents, or sensitive payloads in code, fixtures, logs, traces, evidence, or instructions;
* weaken/delete conformance tests or required gates merely to make checks pass;
* fabricate evidence, passing tests, proof results, or completion status;
* hand-edit `.ua/` or other generated zones;
* treat `.sea-forge/` runtime output as source.

## 5. Commands and Repository Verification

Run `just` from the repository root; it is the canonical human/agent command surface. Prefer existing recipes over equivalent raw Cargo commands; raw Cargo is for diagnostics or missing recipes and must not bypass stronger project gates.

Required gates by claim:

* handoff -> `just context-check`
* Rust quality -> `just check`
* Rust tests -> `just test`
* current-platform CI union -> `just ci`
* minimum proofs P1–P4b -> `just proof`
* Workbench -> `just workbench-check` (see `workbench/AGENTS.md`)
* dependency-boundary change -> `just no-async-kernel` (see `crates/AGENTS.md`)
* milestone/feature work -> governing spec's named gate

`just ci` does not include `just proof`, Workbench gates, or other-platform tests. Run applicable extra gates and report skipped platform tests with reasons. A fast inner loop changes when cost is paid, not what must ultimately be proven.

Do not run `just pr`, `publish-bootstrap`, `clean`, secrets recipes, or other destructive/external-write recipes without explicit approval.

## 6. Testing & Development Discipline

Use test-driven development for logic, fixes, state transitions, and behavior when practical:

1. Identify the observable behavior distinguishing correct from incorrect.
2. Add or identify the focused test first.
3. Confirm it fails for the expected reason before implementing.
4. Implement the smallest coherent change.
5. Run the narrowest relevant crate/test recipe.
6. Broaden verification with blast radius.
7. Run every required milestone, CI, proof, or settlement gate before completion.

Global workstation tools are capabilities, not project policy. Repository toolchain pins, `justfile`, specs, configuration, CI, and verification contracts remain authoritative.

## 7. Git, Handoff, and Completion

Use conventional commits: `type(scope): imperative summary`. Before committing, inspect the diff and run blast-radius-appropriate verification. Do not drive-by format/cleanup/rename/refactor, or overwrite/discard unrelated working-tree changes.

Follow `.agents/AGENTS.md` for handoff state requirements (`CURRENT_STATUS.md`, `current_status.yml`) and memory ledgers (`OBSERVED_DEBT.md`, `LESSONS.md`, `OPEN_QUESTIONS.md`).

Before declaring completion:

1. Inspect the final diff.
2. Run proof commands proportional to the claim.
3. Confirm no required gate, assertion, authority path, or sandbox was weakened/bypassed.
4. Record unresolved failures, debt, or uncertainty in `.agents/`.
5. Update `.agents/CURRENT_STATUS.md` and `.agents/current_status.yml`.
6. Run `just context-check` before handoff.
7. Run `graft build` after code changes that affect indexing.
8. Leave the next move explicit if the case remains open.

Hard cap for this file: fewer than 150 lines and no more than 32 KiB.
