# Implementation Plan — Workbench Product Completion Ralph Loop

**Created:** 2026-08-02
**Source of truth:** `.agents/reports/frontend-eval-20260802T142629Z.md`,
`docs/execution/PRODUCT_COMPLETION_DEFINITION.md`, and
`.agents/specs/sea-forge-governed-workbench-frontend-completion-eval-v0.1.md`
**Originating context:** The independent frontend evaluation of revision
`a942e44` returned `FAIL_CONTINUE` in mocked mode: 5 PASS, 45 PARTIAL, 78 FAIL,
14 failed hard gates, one untested hard gate, and eight P1 defect groups. The
owner accepted the scope, platform, feature, authorization, and exclusion
recommendations recorded below for a Ralph-style Codex `/goal` loop.
**Status of the work today:** The minimum kernel, Workbench shell, readiness,
case authoring, case/read/approval/run projections, supervised Linux sidecar,
package recipes, contract generation, and component tests exist. The evaluated
revision lacks a proven real-stack representative journey. The current
worktree is dirty and contains overlapping post-evaluation work in SFWP
correlation, generated contracts, the case-authoring machine, `justfile`, and
handoff documents. Preserve and verify that work; do not reset or overwrite it.

---

## 0. How to use this plan (agent operating instructions)

This plan is the implementation agent's durable work queue. Read it completely
once. On every later `/goal` iteration, reread this section, the current task,
`.agents/CURRENT_STATUS.md`, and the latest evaluator report. Do not re-plan the
whole program.

- Execute Tasks 0–12 in order. A later task may begin only when every dependency
  has observable passing evidence. Do not infer completion from a checkbox,
  prior agent prose, source presence, or an exit-zero executor.
- **Every task ends with a verification gate.** Do not mark a task done until
  its gate exits 0 and its named negative or teeth test has been observed to
  fail when the protected behavior is deliberately broken or substituted by a
  focused mutant.
- **One governed truth path, one real-stack proof.** Authorize every side effect
  before execution; derive UI state from committed records; keep execution,
  evidence, settlement, and capability distinct; never satisfy a journey with
  a mock, preview, synthetic citation, generic toast, or frontend-owned truth.
- Follow TDD for every behavior change: add the smallest focused failing test,
  run it and record the expected failure, implement, run the focused test, then
  run the task gate.
- Select only one atomic fix set per iteration. A fix set should normally touch
  one vertical slice and finish within one context window. If it cannot, split
  it at a source-backed contract boundary and leave both halves honest and
  green.
- Before editing, use `rg --files` and `rg`, read every target file, inspect one
  adjacent implementation and its tests, and classify required capabilities as
  `EXISTS`, `PARTIAL`, `MISSING`, `CONFLICTS`, `UNKNOWN`, or `NOT REQUIRED` with
  `file:line` evidence.
- Use `.agents/skills/building-sea-forge-workbench/SKILL.md` for every Workbench
  slice. Read only the references routed by that skill for the current task.
- Rust SFWP types are canonical. Regenerate JSON Schema and TS/AJV with
  `just workbench-contracts-generate`; never hand-edit generated zones.
- Preserve unrelated worktree changes. At the start and end of every iteration,
  capture `git status --short` and inspect `git diff --check`. Never use
  `git reset --hard`, `git checkout --`, or broad cleanup commands.
- Update `.agents/CURRENT_STATUS.md` after every completed atomic fix set with
  objective, worktree state, changed files, verification, remaining step, and
  blockers. Record the derived gate state and evidence; do not maintain a
  parallel checkbox state. Run `just context-check` before yielding.
- Log only concrete out-of-scope debt in `.agents/OBSERVED_DEBT.md`. Do not turn
  accepted exclusions into implementation work merely to raise the evaluator's
  raw score.
- Do not modify the evaluator spec to make the product pass. Do not weaken,
  delete, skip, or relabel a failing test. Do not replace real-stack evidence
  with mocked IPC evidence.
- Do not commit, push, open a PR, publish, deploy, delete files, or run secrets
  recipes unless the user separately requests it. CI workflow changes are
  authorized only in Task 12. Publishing remains unauthorized.

### Ralph iteration algorithm

Run this algorithm on every `/goal` turn:

1. Read `.agents/CURRENT_STATUS.md`, this section, the current task, and the
   newest `.agents/reports/frontend-eval-*.json` if one exists.
2. Run `git status --short`, `git diff --check`, and `just context-check`.
3. Find the first task whose `Done when` statement lacks current passing
   evidence. Within it, choose the smallest reproducible failure or missing
   acceptance assertion. Never skip forward to an easier route.
4. Reproduce it. If it already passes because of post-evaluation work, add or
   locate its teeth test, record the evidence, and continue to the next
   acceptance assertion without rewriting it.
5. Write a focused failing test before product logic. For UI behavior, cover
   the canonical Rust/service behavior first when applicable, then the typed
   host boundary, interaction state, component, and real-stack journey.
6. Implement only enough to pass that assertion while preserving the invariant
   chain. Run the focused gate, the task's cheap regression gate, and
   `git diff --check`.
7. If the task gate is fully green, record `PASSING`, the command, and evidence
   in the handoff. If not, record `FAILING` with the exact next failing command
   and continue on the next `/goal` iteration. Any upstream change that can
   affect a prior gate returns that gate to `UNVERIFIED` until rerun.
8. At Tasks 3, 7, 10, and 12, start a fresh read-only evaluator session using
   the canonical evaluator spec. The implementer may prepare fixtures and
   inputs but may not grade its own work.
9. On evaluator `FAIL_CONTINUE`, feed only reproducible in-scope defects and
   `next_fix_set` back into step 3. On `BLOCKED`, repair only the missing test
   environment/input. On `PASS_STOP`, continue only if the product-completion
   gates in Task 12 are not yet green.
10. Stop only when Task 12's complete exit condition is true. Near-green,
    mocked, manually narrated, or platform-skipped results are not terminal.

### Derived completion states

- `UNVERIFIED`: no current gate result exists for the present tree. This is the
  default even when implementation files or an old passing report exist.
- `FAILING`: the gate exits nonzero or required observable/teeth evidence is
  missing. This is the next-work state when dependencies pass.
- `PASSING`: the full gate exits 0 on the current tree and the named prohibited
  mutation is known to make its focused test fail. Record command, tree state,
  and evidence in `.agents/CURRENT_STATUS.md`.
- `BLOCKED`: a specific external input, authority, or environment prevents the
  gate from running. Record the exact blocker and resumption command. Hard or
  unfinished work is `FAILING`, not `BLOCKED`.

Recompute downstream and affected prior gates after relevant changes. A stale
`PASSING` result becomes `UNVERIFIED`; it never remains true because prose says
so.

### Authority granted by the owner for this plan

The owner authorized:

- additive SFWP inspect/command/event methods required by the in-scope
  representative journey;
- corresponding additive canonical Rust types and regenerated JSON Schema,
  TypeScript, and AJV projections;
- Linux-first package and real-stack test work under the already resolved U-06
  supervised-sidecar contract;
- Workbench command-surface and test-fixture changes needed for deterministic
  integrated evaluation;
- CI aggregation and Linux release-gate changes only in Task 12.

The owner did **not** authorize breaking SFWP behavior, changes to existing ID
grammar or policy precedence, incompatible persisted-schema/layout changes, new
dependencies, secret access, publishing, deployment, deletion, or macOS support
claims. If an in-scope task truly requires one of those, prove the need with a
failing test and source evidence, record it in `.agents/OPEN_QUESTIONS.md`, and
ask the owner. Do not route around the boundary.

### Accepted product scope and evaluator exclusions

The completion claim is the representative local Linux product in
`docs/execution/PRODUCT_COMPLETION_DEFINITION.md`, not 128/128 catalog breadth.
All stories are in scope except the exact owner-approved exclusions below.
Excluded routes must be absent or visibly labeled `Unsupported in this
release`; they must not advertise a spendable action, generate fake records, or
block the representative journey.

| Excluded story IDs | Accepted reason |
| --- | --- |
| `2.3` | Automated actors and sponsorship are not advertised in the first product claim. |
| `4.2`, `4.5`, `4.6` | ADLC/ODI/orchestration catalogs and agent endpoint browsing/probing are deferred; one real local template/environment/executor path remains required. |
| `6.1`, `6.3`–`6.6` | Known-intent, external-plan, ADLC/ODI, multi-agent, and spec-to-code case-entry families are deferred; the versioned-template path remains required. |
| `7.6` | General discretionary add-work is deferred; retry, replan, terminate, park, resume, and recovery remain required. |
| `8.6` | ACP permission requests are excluded with agent/ACP execution. |
| `9.4`–`9.7` | Agent configuration, HTTP/ACP agents, SWE_SEED agents, and dialogue intervention are not advertised. Governed local command execution remains required. |
| `10.1`–`10.6` | The Thoth manager loop is deferred. Source-linked, disclosure-controlled Thoth inspection remains required. |
| `11.2`, `11.3`, `11.7` | Capacity management, notifications, and generalized earlier-stage reactivation are deferred. Scoped command cancellation and interruption recovery remain required. |
| `14.2`, `14.6`–`14.10` | General spec-to-code, cognitive synthesis, IP productization, capitalization, stage-gate, and TransitionToken product surfaces are deferred. One governed projection and artifact-reuse path remains required. |
| `15.1`–`15.7` | Cross-cell federation, import, export, and adoption are deferred. |
| `16.3` | Extension and agent-endpoint lifecycle management is deferred. |

Non-story exclusions: macOS until packaged Seatbelt proof exists; Windows,
mobile, remote HTTP, multi-user auth, containers, Kubernetes, hosted control
planes, MicroVM, NATS, marketplaces, real ACP/SWE_SEED/witness/IFL integration
claims, crates.io publication, and CopilotKit. Linux Landlock is in scope.

### Dependency graph

```text
Task 0  freeze current truth, preserve dirty work, publish exact eval inputs
  └─ Task 1  identity-safe readiness and spendable affordances (SF-FE-001/010)
       └─ Task 2  typed refusal + committed source-record vocabulary (002/003)
            └─ Task 3  deterministic real Tauri/server evaluator plant
                 └─ Task 4  cell startup, readiness, Thoth affordance
                      └─ Task 5  domain/template/environment → immutable case
                           └─ Task 6  canonical command episode → settlement
                                └─ Task 7  horizon, human approval, intervention, recovery
                                     └─ Task 8  evidence, audit, replay, integrity
                                          └─ Task 9  memory/capability/projection/artifact reuse
                                               └─ Task 10 operational health and rebuild/reload
                                                    └─ Task 11 packaged Linux journey + a11y/security
                                                         └─ Task 12 CI, release artifacts, independent PASS_STOP
```

Tasks are sequential because Tasks 1–10 repeatedly extend the SFWP enum,
generated contracts, and the same real-cell journey. Do not parallelize edits
to those shared chokepoints in a Ralph loop.

### Global verification gates

After every atomic fix set, run the narrow test plus:

```bash
just context-check
just check-fast
just workbench-contracts-gate
git diff --check
```

After every completed task, run:

```bash
just check
just test
just proof
just workbench-check
```

When Rust SFWP contract types change, regenerate before either gate:

```bash
just workbench-contracts-generate
just workbench-contracts-gate
```

Use `just workbench-package` and the real-stack E2E gate only at the checkpoints
named below; they are intentionally slower.

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
| --- | --- |
| Governing completion claim and 12-step representative journey | `docs/execution/PRODUCT_COMPLETION_DEFINITION.md` §§Completion Claim Levels, Primary User Journey |
| Accepted architectural and owner decisions, including U-06/U-07 and agent authority | `docs/execution/DECISION_REGISTER.md` §§Resolved Decisions, U-06, U-07, Agent Authority Classification |
| Ordered cross-layer completion nodes N08–N13 | `docs/execution/EXECUTION_DAG.md` |
| Independent evaluator, hard gates, exact story inventory, and stop-loop protocol | `.agents/specs/sea-forge-governed-workbench-frontend-completion-eval-v0.1.md` Parts I, V, VIII, IX |
| Baseline defects SF-FE-001–007 and SF-FE-010 | `.agents/reports/frontend-eval-20260802T142629Z.md` §Blocking defects |
| Renderer/host/server boundary and generated-zone law | `workbench/AGENTS.md` §§Boundaries, Generated-zone rules |
| Required slice workflow and capability classification | `.agents/skills/building-sea-forge-workbench/SKILL.md` §§Mandatory repository inspection, Vertical-slice workflow |
| SFWP canonical registry/schema producer | `crates/sea-forge-server/src/sfwp/mod.rs`, `crates/sea-forge-server/src/bin/gen_sfwp_schema.rs` |
| Closed Tauri query/command bridge | `workbench/apps/desktop/src-tauri/src/bridge.rs` |
| Existing request recovery work overlaps the dirty tree | `crates/sea-forge-server/src/sfwp/correlation.rs`, `workbench/apps/desktop/src/machines/caseAuthoringMachine.ts` |
| Existing mocked browser harness is not integrated evidence | `workbench/apps/desktop/playwright.config.ts`, `workbench/apps/desktop/e2e/tauriMock.ts` |
| Existing package/supervisor proof | `workbench/apps/desktop/src-tauri/tests/packaged_stack.rs`, `workbench/apps/desktop/src-tauri/src/supervisor.rs` |
| Real records can be generated for a demonstration cell | `scripts/seed-cell.sh`, `justfile` recipes `cell-seed`, `cell-reset`, `workbench-demo` |
| Canonical commands and package inventory | `justfile` recipes `workbench-check`, `workbench-tauri-test`, `workbench-package`, `workbench-package-inventory`, `ci`, `proof` |
| Minimum compatibility and proofs P1–P4b must remain unchanged | `.agents/specs/spec-minimum.md` §12.2; `just proof` |
| Full-spec behavior is additive and must reuse one authority fabric | `.agents/specs/spec-full.md`; `docs/execution/DECISION_REGISTER.md` R-03 |

---

## Task 0 — Freeze the executable baseline and evaluation contract  (grounding · P0)

**Goal:** The current dirty tree is understood and preserved, every previously
completed node is proven rather than assumed, and a cold evaluator has exact
commands, identities, fixtures, and exclusions instead of placeholders.

**Why this shape:** The report evaluated `a942e44`, but the worktree contains
post-evaluation changes in the same files needed by Tasks 1–3. Reverting them or
redoing already-correct work would destroy user work and corrupt the baseline.

### Steps

1. Read `.agents/CURRENT_STATUS.md`, `git status --short`, and focused diffs for
   every modified file. Classify each change as relevant completed work,
   relevant incomplete work, or unrelated user work. Do not edit or reset it.
2. Re-run the focused tests named in the current status for U-06/U-07,
   idempotency, contracts, package inventory, and case-authoring recovery.
3. Create `.agents/reports/workbench-completion-eval-inputs.md` from the
   evaluator's paste-ready invocation. Fill repository root, build/test/start
   commands, fixture identities, real temporary-cell setup, and the exact
   exclusions above. No placeholder may remain.
4. Add a machine-checkable exclusion fixture, owned by the evaluator harness,
   containing the exact story IDs and reasons above. The evaluator input should
   reference it rather than duplicate an informal list.
5. Update `.agents/CURRENT_STATUS.md` with which Execution DAG nodes are proven
   complete, partial, or not yet proven at the current tree.

### Gate

```bash
just context-check
just check-fast
just workbench-contracts-gate
git diff --check
! rg -n '<[A-Z_]+>' .agents/reports/workbench-completion-eval-inputs.md
```

**Done when:** A new cold session can reproduce the baseline without asking for
a command, identity, fixture, or scope decision; removing one declared
exclusion or restoring a placeholder makes the input-validation test fail.

**Redesign trigger:** If a dirty change cannot be attributed or safely tested,
stop and ask the owner whether to preserve, finish, or isolate that exact diff.

---

## Task 1 — Make readiness actions identity-safe and spendable  (safety · P0)

**Goal:** Unresolved identity blocks every protected action with typed
`identity_unresolved` context and a lawful repair path, while every enabled
readiness action performs its advertised navigation/evidence action without
overlay interception.

**Why this shape:** SF-FE-001 violates authority-before-effect; SF-FE-010 makes
the highest-priority affordance inert. Both are direct, reproducible defects and
must close before broader journeys can be trusted.

### Steps

1. Map U-07's server-derived per-request identity through the host bridge and
   renderer session. Protected verbs require actor context; inspect verbs remain
   backward-compatible without it.
2. Replace readiness-only `canCreateCase` logic with a shared protected-action
   guard consuming validated identity, policy/readiness, and operation context.
3. Render the structured refusal, unchanged-effect statement, affected action,
   and identity repair route through the semantic protected-action components.
4. Wire “Inspect all capabilities” to the current source-backed capability or
   evidence target. Correct evidence-drawer stacking/pointer behavior so a
   closed or unrelated drawer cannot intercept the action.
5. Add Rust/host tests proving actor omission denies protected verbs without an
   effect, component tests for enabled/disabled reasons, and Playwright tests
   for unresolved/resolved identity plus the drawer-open click path.

### Gate

```bash
just crate-test sea-forge-server identity
just workbench-tauri-test
cd workbench && bun run test
cd workbench/apps/desktop && bun run e2e -- --grep "identity|Inspect all capabilities"
just workbench-check
```

**Done when:** The exact B6 fixture observes Create case disabled with
`identity_unresolved` and a usable repair action; a resolved actor can reach
case creation; the capability action changes route or evidence focus with the
drawer open. Mutating the guard back to readiness-only or removing the handler
makes a named test fail.

**Redesign trigger:** If identity cannot be trusted without renderer-supplied
claims, stop; use U-07's socket-derived identity path instead of adding a
frontend identity store.

---

## Task 2 — Preserve typed governance and committed source truth end to end  (contracts · P0)

**Goal:** Important readiness, authority, denial, approval, and assurance states
resolve to typed committed records/digests with freshness and lawful next
actions; free-form errors and code citations cannot masquerade as governance
truth.

**Why this shape:** SF-FE-002 and SF-FE-003 jointly block G3–G7, G12, and G14.
Fixing their visual symptoms without one canonical protocol vocabulary would
fork truth across routes.

### Steps

1. Inventory existing decision, approval, policy, self-model, event, evidence,
   and digest records. Reuse their canonical IDs and reducers; do not create UI
   status records.
2. Define the smallest additive SFWP projections for `SourceRecordRef`,
   freshness/rebuild standing, and structured refusal/approval context: typed
   reason, matched policy/boundary, actor/resource/purpose, downstream effect,
   eligible actors, expiry, side-effect standing, evidence, and lawful action.
3. Make readiness and sampled status producers return committed record/event or
   digest references. Keep a code location only as explanatory metadata; never
   wrap it as synthetic evidence.
4. Carry structured error classes unchanged through server, host, generated
   validation, query/command clients, and semantic components. Unknown variants
   render `unknown`/stale and never success.
5. Add conformance tests for denial, escalation, expiry, stale source,
   unavailable rebuild, unsupported variant, approver ineligibility, and no
   side effect. Add route/component tests for reason, boundary, effect, source,
   freshness, and next action.

### Gate

```bash
just workbench-contracts-generate
just crate-test sea-forge-server conformance
just workbench-contracts-gate
just workbench-tauri-test
cd workbench && bun run test
just check
just test
```

**Done when:** The evaluator's B7 denial and readiness-source probes reach typed
machine-readable source records and show structured reason/boundary/effect/next
action. Replacing a record reference with a source-code citation or a typed
denial with free text makes conformance and component tests fail.

**Redesign trigger:** If no canonical record exists for a claimed state, either
derive it honestly from existing committed records with provenance or render
`unknown`/unsupported. Do not mint a record merely to satisfy the UI.

---

## Task 3 — Build the deterministic real-stack evaluation plant  (test architecture · P0)

**Goal:** One noninteractive command launches a temporary Linux cell, real
server sidecar, Tauri host, and renderer; drives deterministic identities and
fixtures through the closed bridge; captures traces/screenshots/records; and
cleans up without mocked `__TAURI_INTERNALS__`.

**Why this shape:** Most PARTIAL findings lack runtime evidence. More mocked
fixtures cannot prove authority timing, reconnect, restart, sandboxing,
append-only history, or record linkage. Real-stack proof is infrastructure for
every remaining task, not final polish.

### Steps

1. Extend the existing seed/reset and `packaged_stack.rs` patterns into a
   temporary-cell fixture builder that produces records only through real
   kernel/server operations. Support healthy, unresolved-identity, denial,
   escalation, false-success, stale, expired, interrupted, and tampered-copy
   scenarios without fabricated production evidence.
2. Add a Linux real-Tauri driver/harness that exercises the compiled app and
   sidecar. Reuse installed tools where possible. Any new dependency requires
   owner approval before addition.
3. Add `just workbench-e2e-real filter=''` as the one test entry point. It must
   accept an optional test-name filter, allocate unique explicit temporary
   roots/sockets, enforce timeouts, retain artifacts on failure, and terminate
   only processes it started.
4. Keep mocked Playwright tests as component-speed evidence, label them mocked,
   and remove them from integrated completion claims.
5. Prove hello/version negotiation, generated payload validation, owner-only
   socket, cursor reconnect, request-status recovery, sidecar adoption/spawn,
   and renderer inability to access socket/files directly.
6. Run a fresh evaluator checkpoint with the Task 0 inputs; record the new
   report paths and next in-scope defect set in `.agents/CURRENT_STATUS.md`.

### Gate

```bash
just workbench-package
just workbench-package-inventory
just workbench-e2e-real "hello|identity|reconnect|request recovery"
just workbench-tauri-test
```

**Done when:** The harness reports integrated mode, produces inspectable real
record IDs, and passes after server reconnect/restart. Disabling the server,
replacing the sidecar with a version mismatch, or injecting mocked Tauri IPC
makes the gate fail rather than silently falling back.

**Redesign trigger:** If native UI automation needs an unapproved dependency or
unsupported host facility, stop for the narrow dependency/environment decision.
Do not substitute Vite-only Playwright and call it integrated.

---

## Task 4 — Complete cell startup, readiness, and bounded Thoth affordance  (journey 1–3 · P1)

**Goal:** A fresh or existing cell starts without overwrite, validates/migrates
compatible history, displays resolved identity/policy/model/integrity/sandbox
readiness, and answers what the installation can do with disclosure-controlled,
source-linked claims.

**Why this shape:** This is the entry contract for every later side effect. A
case journey is invalid if startup silently defaults, loses history, or presents
declared capability as demonstrated.

### Steps

1. Implement open/select/initialize with safe-join validation, existing-history
   detection, explicit compatible migration, version negotiation, and visible
   refusal for incompatible history. Ask before any newly required persisted
   layout/schema change.
2. Complete fail-closed startup/config preflight: policy, sandbox, self-model,
   DomainForge, identity, root/socket, and integrity. Invalid state exposes its
   source record, affected capability, and lawful repair.
3. Complete readiness vocabulary and assurance without collapsing stale into
   degraded or installed into demonstrated.
4. Complete the canonical direct Thoth inspection port for identity,
   capability, operation/authority requirements, projection support, current
   affordances, failure/denial, and evidence. Retrieval remains disclosure-first
   and grants no execution authority.
5. Add real-cell scenarios for fresh, existing, migration, incompatible,
   healthy, stale, blocked, degraded, denial, and restricted disclosure.

### Gate

```bash
just workbench-e2e-real "cell|readiness|Thoth"
just crate-test sea-forge-server readiness
just crate-test sea-forge-thoth ask
just workbench-check
just proof
```

**Done when:** In-scope stories `1.1`–`3.9` except excluded `2.3` pass against
real records; a restricted Thoth probe proves no protected content is retrieved
before the disclosure decision. Existing history and an invalid config both
survive with zero unauthorized overwrite/effect.

**Redesign trigger:** If a Thoth answer needs data not lawfully disclosed by an
existing source, return a bounded omission. Do not add broad graph disclosure.

---

## Task 5 — Commit one real semantic template case immutably  (journey 4–6 · P1)

**Goal:** A user selects one real local domain model, versioned template,
environment, and command executor; validates semantics; inspects immutable
criteria/provenance/authority boundaries; preflights; and commits exactly one
recoverable case.

**Why this shape:** Product completion requires one honest source-to-case path,
not every alternative case-entry family. Existing empty catalogs and preview
screens must become source-backed only for that path.

### Steps

1. Expose one real source-owned template/environment/executor and its exact
   versions/digests through the existing asset projections. Do not fabricate a
   built-in if the kernel has no source-owned asset.
2. Wire DomainForge's existing library adapter for syntax/import/semantic
   validation, model inspection, semantic-reference checks, plan pinning,
   drift refusal/revalidation, and the one in-scope governed projection path.
3. Expand case preview to show purpose, outcome, model, criteria, origins,
   environment, executor, sandbox, authority boundary, approvals, and close
   rule from validated contracts.
4. Preserve preflight/commit separation, template-digest staleness, stable
   request ID, ambiguous-response recovery, and exactly-once case minting.
5. Add real-stack cases for success, invalid semantic source, drift, stale
   preflight, lost response, denial without effect, and incompatible asset.

### Gate

```bash
just crate-test sea-forge-server case
just crate-test sea-forge-planner template
just workbench-e2e-real "domain|template|preflight|commit|drift"
just workbench-contracts-gate
just proof
```

**Done when:** In-scope Journey 4–6 stories pass; one real browser commit creates
one case and can be recovered by request ID after a lost response. Changing the
template after preflight or reusing the request ID with changed input rejects
with no second case.

**Redesign trigger:** If satisfying the representative path requires a new
general editor, graph library, or alternate case source, stop and shrink back
to the versioned-template path.

---

## Task 6 — Run one canonical governed command episode to settlement  (execution · P0)

**Goal:** The committed case launches a bounded local command only after exact
authority, uses the granted environment and Linux Landlock sandbox, records the
complete canonical lifecycle, and displays execution separately from settlement.

**Why this shape:** SF-FE-004 and G2/G10/G13 cannot close around a preview-only
delegation route. The case path must reuse the kernel's one episode pipeline,
not add Workbench-specific execution.

### Steps

1. Route case command episodes through the existing canonical criteria →
   authority → sandbox/runtime → trace → evidence → settlement → declaration
   pipeline. Workspace creation must occur after authority.
2. Make one run ID resolve consistently through case, restart, CLI/SFWP, and UI;
   keep legacy flat run fixtures readable without re-keying.
3. Add the protected execute command and bounded monitor through the closed
   bridge. Show actor, action, environment, sandbox, limits, secret references,
   consequence, immutable criteria, progress, outputs, termination, evidence,
   and settlement as separate domains.
4. Implement scoped cancellation and retry-as-new-episode; never edit or replay
   the prior episode and never infer cancellation/settlement optimistically.
5. Prove allow, deny/escalate with no effect, timeout, nonzero exit, cancellation,
   sandbox violation, bounded output, secret-sentinel absence, and exit-zero
   false-success rejection.

### Gate

```bash
just crate-test sea-forge-server episode
just crate-test sea-forge-case-runner lifecycle
just proof
just no-async-kernel
just workbench-e2e-real "command|authority|settlement|false success|cancel"
```

**Done when:** In-scope `9.1`–`9.3`, `9.8`, `9.9`, G1, G2, G10, and G13 pass on
real records. Replacing settlement with exit-status inference or moving
workspace creation before authority makes a conformance test fail.

**Redesign trigger:** If the case runner cannot reuse the canonical pipeline,
stop and reconcile the service boundary; do not duplicate lifecycle logic.

---

## Task 7 — Make the case horizon, human approval, and recovery spendable  (journey 7–8, 11 · P1)

**Goal:** The live case explains structure and transitions, completes one human
approval/task with server-enforced separation of duty, and supports append-only
retry, replan, terminate, park/resume, disconnect, restart, and orphan recovery.

**Why this shape:** SF-FE-006 is one lifecycle gap crossing horizon, inbox,
operations, and request recovery. Adding isolated buttons without canonical
events would create editable history.

### Steps

1. Extend case/horizon projections with purpose, outcome, owner, close rule,
   stages/items/milestones/sentries, lanes, triggering event, transition cause,
   authority, evidence, settlement, and episode lineage from committed records.
2. Implement protected append-only replan, terminate, park, resume, and
   retry-new-episode commands. Reopen only where the current case lifecycle
   defines it; do not mutate prior events.
3. Complete approval and human-task inbox rows with full typed decision context,
   eligibility, expiry, precondition, evidence/note, downstream effect, and
   lawful next step. Enforce approver identity/SoD server-side.
4. Complete request ambiguity, disconnect/cursor-gap, server-kill, in-flight,
   orphan, and unsettled recovery. Never blind-resubmit or silently auto-retry.
5. Add real fixtures for active/waiting/blocked/parked/future/completed/rejected,
   expired/stale/unauthorized approval, cancellation with sibling history,
   disconnect, restart, orphan recovery, and failed recovery.
6. Run a fresh independent evaluator checkpoint and feed its in-scope
   `next_fix_set` back through this task before advancing.

### Gate

```bash
just crate-test sea-forge-server approval
just crate-test sea-forge-server recovery
just workbench-e2e-real "horizon|approval|human|replan|park|resume|restart|orphan"
just check
just test
```

**Done when:** In-scope Journey 7, 8, and 11 stories pass; expired or
self-approval denies with no effect; retry creates a distinct linked run; a
server kill leaves an explicit recoverable/orphan state. Editing history or
blindly resubmitting makes named tests fail.

**Redesign trigger:** If a requested lifecycle action has no canonical reducer
or spec-defined event, render it unsupported and stop for a public-contract
decision rather than inventing frontend state.

---

## Task 8 — Make evidence, audit, replay, and integrity independently inspectable  (journey 12 · P1)

**Goal:** An auditor can traverse a run's authority, trace, artifacts,
declarations, criteria, evidence, settlement, ordered case replay, disclosure
history, and ledger integrity through machine-readable source links.

**Why this shape:** A rich run page still fails G4/G9/G12/G14 if its rows are
dead-end summaries or if provenance, inclusion, and disclosure cannot be
verified independently.

### Steps

1. Complete source-linked run/evidence/settlement projections with origin and
   semantic envelope, artifact ownership/license identity, digests/events,
   declaration policy/SoD/reliability, and machine-readable endpoints.
2. Add ordered append-only case replay derived from canonical events. Never
   cache a competing sequence as truth.
3. Add disclosure-constrained authority/Thoth/audit search. Decide disclosure
   before retrieval and record influence/access where specified.
4. Surface chain/MMR inclusion and consistency verification. Witness/IFL
   integrations remain excluded, but local integrity verification is required.
5. Add invalid provenance, self-declaration, tampered-copy, missing record,
   stale projection, disclosure denial, and false-success fixtures.

### Gate

```bash
just crate-test sea-forge-ledger integrity
just crate-test sea-forge-server run
just workbench-e2e-real "evidence|audit|replay|integrity|provenance"
just proof
```

**Done when:** In-scope Journey 12 and G4/G9/G12/G14 pass; every important row
opens a typed source or honestly reports it unavailable. Tampering with a copied
ledger produces `integrity_failed`, never a stale-success view.

**Redesign trigger:** If a view cannot be rebuilt from canonical records, do not
ship it as audit truth; expose the canonical records directly or mark it
unavailable.

---

## Task 9 — Reuse one accepted result without overstating capability  (journey 13–14 · P1)

**Goal:** One accepted result becomes governed memory, conservative capability,
one verified projection, and a lineage-linked artifact; stale indexes fall back
to authoritative records and insufficient evidence cannot promote capability.

**Why this shape:** Product completion requires one end-to-end reuse path.
SF-FE-005 should not trigger broad federation, IP-productization, or catalog
work that the owner excluded.

### Steps

1. Ground additive views/actions in existing memory, capability, evidence,
   self-model, projection, and artifact services. Reuse their promotion and
   disclosure rules; never let the UI manually promote canonical standing.
2. Implement disclosure-scoped recall with actor boundary before retrieval,
   authoritative fallback for stale/unavailable indexes, freshness/rebuild
   labels, and influence links into subsequent work.
3. Implement capability detail for attempted/demonstrated/proven/degraded/
   quarantined/contracted standing, qualifying settlements, variation,
   recovery, burden, promotion/contraction policy, exclusions, and next proof.
4. Complete one governed deterministic projection with source/adapter/version/
   parameters/limitations, validated hash-linked output, quarantine on invalid
   output, and explicit generated-output versus runtime-readiness standing.
5. Complete artifact identity/lineage and the accepted-result reuse action.
   Excluded federation and productization routes remain unsupported.
6. Prove disclosure denial before retrieval, stale fallback, insufficient
   evidence, regression/contraction, invalid projection quarantine, rebuild
   equivalence, and accepted reuse.

### Gate

```bash
just crate-test sea-forge-capability recall
just crate-test sea-forge-capability capability
just crate-test sea-forge-artifact-ip artifact
just workbench-e2e-real "memory|capability|projection|artifact|reuse|quarantine"
just workbench-contracts-gate
```

**Done when:** In-scope Journey 13 plus `14.1`, `14.3`–`14.5` pass. Deleting or
staling the index still produces correct authoritative results with a visible
freshness warning; one weak settlement cannot promote capability; an invalid
projection cannot be reused.

**Redesign trigger:** If accepted reuse requires a broad marketplace,
federation, or manual promotion action, stop; those are excluded and cannot
become shortcuts.

---

## Task 10 — Make operational health, reload, and rebuild honest  (journey 16 · P1)

**Goal:** Operators can validate/rebuild self-model and derived stores, inspect
version skew and operational debt, survive failed rebuild/reload with the last
verifiable snapshot, and protect in-flight work from configuration changes.

**Why this shape:** Readiness cannot be a static badge. Product completion
requires repair and last-known-good behavior without silent fallback.

### Steps

1. Implement validate/rebuild actions for self-model, memory, capability, and
   other in-scope derived stores through governed server commands.
2. Add atomic last-known-good config reload for future dispatches only. Invalid
   reload preserves the prior snapshot, emits a visible event, and cannot alter
   in-flight authority/runtime context.
3. Surface server/desktop/CLI/SFWP/schema/model/adapter compatibility and skew
   with source records and corrective actions.
4. Provide one source-backed operational debt/readiness view linking every
   blocked/stale/degraded/unknown condition to affected capability and lawful
   repair.
5. Test successful/failed rebuild, stale-but-verifiable snapshot, independent
   derived-store rebuild equivalence, incompatible version, valid/invalid
   reload, and in-flight isolation.
6. Run a fresh evaluator checkpoint. Resolve all reproducible in-scope
   `FAIL`, `PARTIAL`, and failed hard gates before advancing; excluded stories
   must be reported `OUT_OF_SCOPE`, not `FAIL` or fabricated `PASS`.

### Gate

```bash
just crate-test sea-forge-server reload
just crate-test sea-forge-self-model rebuild
just workbench-e2e-real "health|rebuild|reload|stale|compatibility|debt"
just check
just test
just workbench-check
```

**Done when:** In-scope Journey 16 stories pass; invalid reload/rebuild retains
the last verifiable snapshot and marks it stale; a future dispatch uses the new
valid config while an in-flight episode retains its original authority hash.

**Redesign trigger:** If reload requires mutable global policy observed by an
in-flight run, stop and introduce snapshot binding at the existing authority
boundary rather than accepting the race.

---

## Task 11 — Prove the packaged Linux product journey  (distribution · P0)

**Goal:** A clean Linux installation completes the full in-scope representative
journey through the packaged Tauri app, supervised real sidecar, Unix socket,
Landlock, and persistent records with accessibility and secret-safety evidence.

**Why this shape:** Developer Vite and host integration tests establish slices,
not distribution verification. The Product Completion Definition requires the
installed artifact and restartable retained cell.

### Steps

1. Build and install/test the supported Linux package in a clean disposable
   environment without a repository checkout or Bun runtime. Verify checksums,
   sidecar, versions, root/socket agreement, `0600` socket, conflicting-server
   refusal, and package inventory.
2. Run the complete 12-step representative journey against one temporary cell,
   substituting the accepted non-agent command path and excluding only the exact
   stories above. Retain record IDs and traces for every step.
3. Run allow, deny/escalate, false-success, lost response, disconnect, restart,
   orphan recovery, audit, integrity, and accepted reuse as one ordered suite.
4. Run keyboard-only navigation, focus order/return, Escape behavior, 200% zoom,
   390px narrow viewport, reduced motion, axe on every in-scope route, console
   error capture, and horizontal-overflow checks.
5. Scan source records, logs, errors, rendered UI, test artifacts, transcripts,
   bundles, and copied/exported in-scope content for secret sentinels. Verify CSP
   and Tauri capability scopes permit only required local actions.
6. Document macOS and every other accepted platform/integration exclusion in
   release-facing support text without implying verification.

### Gate

```bash
just workbench-package
just workbench-package-inventory
just workbench-e2e-real
just ci
just proof
just workbench-check
```

**Done when:** The installed Linux package completes the journey twice—fresh
cell and retained-cell restart—with zero mocks, console errors, axe violations,
horizontal overflow, secret leakage, or unexplained skips. Removing the sidecar,
Landlock availability, or an authority record makes the suite fail closed.

**Redesign trigger:** A host missing required Linux facilities is a skipped
platform environment with a stated reason, not a pass. Fix the release runner;
do not weaken sandbox or package claims.

---

## Task 12 — Gate release artifacts and obtain independent PASS_STOP  (release · P0)

**Goal:** Required CI aggregates every product gate, Linux release artifacts are
retained and smoke-tested, documentation matches verified support, and a fresh
read-only evaluator returns integrated `PASS_STOP` with every non-excluded story
and all 15 hard gates passing.

**Why this shape:** This is the only terminal condition. The implementation
agent's own tests and narrative cannot independently qualify the frontend or
the product.

### Steps

1. Under the owner's Task 12 authorization, wire CI to run the root kernel gate,
   minimum proofs, Workbench contracts/host/frontend gates, Linux package and
   inventory, real packaged E2E, secret scan, and artifact retention. Do not add
   publishing or unverified macOS targets.
2. Add release notes/support documentation naming supported Linux artifacts,
   hashes, migrations/compatibility, exact exclusions, and bounded limitations.
   Remove or correct claims contradicted by current evidence.
3. Run all local equivalents from a clean checkout/worktree or CI revision.
   Generated artifacts must reproduce with zero diff; package install smoke and
   retained-cell restart must pass.
4. Start a fresh evaluator session using the entire canonical evaluator spec and
   Task 0's exact input packet. The evaluator is read-only and receives no
   implementer self-assessment. Store both timestamped Markdown and JSON output.
5. If the evaluator returns `FAIL_CONTINUE`, validate each defect against the
   accepted scope. Route reproducible in-scope defects back to the earliest
   owning task and repeat. Correct evaluator inputs for accepted exclusions;
   never implement excluded breadth merely to improve score. Treat challenged
   exclusions or a newly necessary forbidden contract change as an owner
   decision.
6. After `PASS_STOP`, run `just context-check`, update
   `.agents/CURRENT_STATUS.md` with exact final commands/results and supported
   claims, and stop. Do not continue optional polish.

### Gate

```bash
just ci
just proof
just workbench-check
just workbench-package
just workbench-package-inventory
just workbench-e2e-real
git diff --check
just context-check
jq -e '
  .verdict == "PASS_STOP" and
  .qualification == "FRONTEND_COMPLETE_INTEGRATED" and
  ([.hard_gates[] | select(.result != "PASS")] | length == 0) and
  ([.story_coverage[] | select((.result == "FAIL") or (.result == "PARTIAL") or (.result == "NOT_TESTED"))] | length == 0)
' "$(ls -1t .agents/reports/frontend-eval-*.json | head -n 1)"
```

**Done when:** The exact gate above exits 0 on the same evaluated revision; CI
retains the verified Linux package and evidence; every non-excluded story is
`PASS`, every excluded story is `OUT_OF_SCOPE` with the accepted reason, all 15
hard gates pass, and no required journey uses mocks, previews, or silent
fallback. Changing the latest JSON verdict or any hard gate/story result makes
the terminal gate fail.

**Redesign trigger:** None. A failed terminal gate means continue at the
earliest owning task; a forbidden change or challenged scope decision means ask
the owner.

---

## Final acceptance gates (whole plan)

- **Task 0 gate:** Current dirty work is preserved, classified, and reproducibly baselined; evaluator inputs contain no placeholders.
- **Task 1 gate:** Unresolved identity blocks protected work and both readiness actions are spendable; guard/handler mutants fail.
- **Task 2 gate:** Typed denial/approval context and committed source-record links survive server → host → validated renderer; citation/free-text mutants fail.
- **Task 3 gate:** One command proves real Tauri/sidecar/server integration, reconnect, request recovery, and no mock fallback.
- **Task 4 gate:** Fresh/existing/migrated cells and bounded Thoth affordance pass against real records with disclosure-first retrieval.
- **Task 5 gate:** One real semantic template/environment/executor case preflights and commits exactly once with drift and ambiguity protection.
- **Task 6 gate:** One governed Linux command episode proves allow, denial-without-effect, timeout, cancellation, Landlock, and false-success settlement.
- **Task 7 gate:** Horizon, human approval/task, SoD, append-only lifecycle, restart, and orphan recovery are spendable and source-backed.
- **Task 8 gate:** Evidence, provenance, audit, replay, machine-readable records, and local ledger integrity are independently inspectable.
- **Task 9 gate:** One accepted result reuses through governed memory/capability/projection/artifact without false promotion or stale-index deception.
- **Task 10 gate:** Rebuild, last-known-good reload, version skew, in-flight protection, and operational debt are honest and actionable.
- **Task 11 gate:** The installed Linux package completes fresh and retained-cell journeys with accessibility, secret, CSP, and capability-scope evidence.
- **Task 12 terminal gate:** CI and release artifacts are gated, and the latest independent evaluation is integrated `PASS_STOP` with all 15 hard gates and every non-excluded story passing.
- **Cross-cutting gate:** `just ci`, `just proof`, `just workbench-check`, `just workbench-package-inventory`, `just workbench-e2e-real`, `git diff --check`, and `just context-check` all exit 0 on the evaluated revision.
- **Reproducibility gate:** Generated Rust schema, JSON Schema, TS/AJV, and UI token projections reproduce with zero diff.
- **Claim gate:** Linux is the only verified package claim; all accepted exclusions are explicit and no excluded surface pretends to be spendable.

The plan is complete only when every gate above is `PASSING` on the same
evaluated tree. No checkbox, file presence, prior status line, or agent claim
can override a failing, blocked, absent, or stale gate result.

## Guardrails (do not violate)

- Authorize every side effect before execution, then trace, evidence, settle,
  and record it. A process exit is never success unless settlement accepts the
  declared outcome.
- Keep authority and isolation independent. Never weaken either by fallback.
- Use one authority mediator and one canonical episode pipeline. The renderer,
  Tauri host, case runner, sandbox, and tests may not duplicate policy or
  settlement logic.
- Normalize intent into typed operations. Never interpolate intent into shell
  commands or paths. Use argv execution, a minimal explicit environment,
  safe-join path validation, enforced timeouts, and bounded output.
- The renderer never accesses the Unix socket, `.sea-forge/`, SQL, filesystem
  truth, or a generic backend invoke. The Tauri host owns the closed typed
  bridge.
- TanStack Query owns inspection views only; protected mutations round-trip
  through SFWP and committed records. XState owns interaction state only.
- Preserve append-only history. Retry creates a new episode; replan, reopen,
  termination, approval, cancellation, and recovery append attributable events.
- Keep execution, settlement, dialogue, integrity, and capability vocabularies
  distinct. Unknown never maps to success.
- Decide disclosure before retrieval. Keep secrets and sensitive payloads out
  of source, fixtures, logs, errors, traces, evidence, UI, and instructions.
- Never fabricate authority, evidence, criteria, sources, identity, confidence,
  fixtures presented as real, or evaluator results.
- Keep kernel crates synchronous; only `sea-forge-agent` and
  `sea-forge-server` may gain async/HTTP dependencies. No `unsafe`.
- Never hand-edit generated zones or runtime output. Change the canonical
  producer and regenerate.
- Never delete or weaken conformance/evaluator tests to pass a gate. A skipped
  platform/integration states its reason and cannot count as passed.
- Do not expand into excluded agents, manager loops, federation, productization,
  alternate case-entry families, remote/multi-user infrastructure, or macOS.
- Do not add or upgrade dependencies, change persisted schema/layout, ID
  grammar, policy precedence, breaking public behavior, deployment, or secrets
  without the required owner approval.
- Preserve unrelated dirty-tree work. Keep each atomic fix set narrow and leave
  the repository green and resumable at every handoff.
