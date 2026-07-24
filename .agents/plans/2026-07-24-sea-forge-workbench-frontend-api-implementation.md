# Implementation Plan — SEA Forge Workbench: Frontend, Desktop Host, API, Events, and Thoth Interaction

**Created:** 2026-07-24
**Source of truth:** `.agents/specs/frontend/` (API spec + method catalog + frontend architecture contract + view-flow + wireframes + DESIGN, precedence per skill), executed through `.agents/skills/building-sea-forge-workbench/`
**Originating context:** Repository-grounding task on branch `full-spec`: the frontend specification package is complete but `status: target-unmapped`; no frontend, Tauri host, Bun workspace, event subscription, or schema generation exists in the repo. This plan closes that gap without weakening kernel invariants.
**Status of the work today:** Backend kernel + Unix-socket NDJSON server EXIST (verbs `submit`/`status`/`approve`/`reject`/`agent_list`/`agent_probe`/`delegate`/`cancel_delegation`/`ask`). Everything renderer-side is greenfield. SFWP envelopes, request recovery, and events are additive server work. The frontend spec package's stale references were corrected 2026-07-24 (see `.agents/skills/building-sea-forge-workbench/reference/source-map.md`).

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in the order given. Dependencies: Tasks 1→2→3→4→5 are strictly sequential (each consumes the previous task's artifacts); Tasks 6–8 depend on 3+4; Task 9 depends on 3; Task 10 depends on 9 and is **optional and skippable**; Task 11 depends on 8+9; Tasks 12–13 depend on 8; Task 14 depends on all prior except 10.
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **One governance path, one contract producer.** Server handlers stay thin adapters over the shared services the CLI already uses (`crates/sea-forge-cli/src/commands/` pattern, `Request::Ask` → `sea_forge_thoth::service::ask`); generated schemas are the only source of TypeScript domain types — never handwrite a canonical enum twice; never fork governance logic into a Workbench-only variant.
- Match surrounding code style; the idioms to mirror are in `crates/sea-forge-server/src/lib.rs` (verb handlers), `crates/sea-forge-server/tests/conformance_m12.rs` (conformance tests), and `crates/sea-forge-cli/src/commands/ask.rs` (thin adapter).
- The frontend spec package is the spec for **semantics**; the repository is the spec for **substrate**. When they disagree, repository invariants win and the conflict is recorded in the task report (skill source hierarchy).
- Before each task, re-read the matching reference file in `.agents/skills/building-sea-forge-workbench/reference/` — this plan states *what*; the skill states *how*.
- New JS/Rust dependencies each need an ADR entry per `docs/decisions/ADR-002-audit-remediation-dependencies.md` (one ADR may cover the locked stack set introduced in Task 2).
- Do not renumber existing task IDs in later revisions; insert hierarchically (Task 5A, 5B…).

### Dependency graph

```text
Task 1 grounding + capability map
  └─ Task 2 workspace + stack proof (Tauri2+Bun+React19+Vite+Astryx+Router+Query+XState)
       └─ Task 3 SFWP transport: envelopes, hello, request-status, events, schema gen
            └─ Task 4 app shell + tokens + semantic components + guards
                 └─ Task 5 Readiness vertical slice  ◄ first user-visible settlement
                      ├─ Task 6 case authoring: draft → preflight → commit
                      │    └─ Task 7 case overview + horizon
                      │         └─ Task 8 execution console + settlement handoff
                      │              ├─ Task 11 agent delegation + ACP permissions + transcripts (also Task 9)
                      │              ├─ Task 12 settlement + recovery surfaces
                      │              └─ Task 13 capability, memory, evidence, integrity
                      └─ Task 9 ThothInteractionPort + DirectAgUiAdapter
                           └─ Task 10 (optional) OSS CopilotKit adapter
Tasks 1-13 (10 optional) ─ Task 14 packaging, accessibility, end-to-end proof
```

### Global verification gates (must stay green after EVERY task)

```bash
devbox run -- just check          # Rust workspace: fmt, lint, typecheck, no-async-kernel
devbox run -- just test           # Rust workspace tests incl. server conformance
devbox run -- just context-check  # .agents/ status coupling (CI-enforced)
cd workbench && bun run check     # frontend lint + typecheck (exists from Task 2 on)
cd workbench && bun run test      # frontend unit/component tests (Task 2 on)
```

Rebuild steps if you touched canonical Rust types (generated contracts) before running the gates:

```bash
cd workbench && bun run generate:contracts   # Rust → JSON Schema → TS (exists from Task 3 on)
git diff --exit-code workbench/packages/contracts/generated || echo "regenerate committed deliberately"
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
|---|---|
| NDJSON Unix-socket server; `Request` enum tagged `verb`, snake_case; unknown verbs rejected cleanly (ADR-003 additive seam) | `crates/sea-forge-server/src/lib.rs:396-401` |
| Listener bind, 0600 perms, line-delimited responses | `crates/sea-forge-server/src/lib.rs:515-568` |
| Server config (`server.yaml`, `socket_path`) | `crates/sea-forge-server/src/config.rs` |
| Canonical domain types (cases, criteria, settlement, approvals, authority, trace), serde snake_case | `crates/sea-forge-core/src/types.rs` |
| Typed error classes (`ForgeError::class()`) → SFWP error taxonomy input | `crates/sea-forge-core/src/errors.rs` |
| Durable cursor substrate: ledger `entry_ulid` + MMR proofs | `crates/sea-forge-ledger/src/types.rs:206`, `:915` |
| Thoth one-path governance seam: `service::ask` shared by CLI + server | `crates/sea-forge-thoth/src/service.rs:258`; protocol types `crates/sea-forge-thoth/src/protocol.rs:41-246` |
| ACP permission mediation trait + fail-closed default | `crates/sea-forge-agent/src/acp.rs:49-93` (`DenyAllMediator`), termination `:129` |
| Delegation, transcript evidence, SWE_SEED declaration ingress | `crates/sea-forge-server/src/delegation.rs`, `swe_seed_reconciliation.rs`, `transcript_seal.rs` |
| SoD authorship check used by settlement + promotion | `crates/sea-forge-core/src/types.rs:1492` |
| Conformance-test idiom to mirror for new verbs | `crates/sea-forge-server/tests/conformance_m12.rs` (and `_m13`, `_m16`) |
| Kernel crates must stay sync; tokio only in server/isolated adapters | root `AGENTS.md` §Project Map |
| **No** JS workspace, package manager, Tauri, schema gen (`schemars`/`ts-rs`/`typeshare` absent from every `Cargo.toml`) | verified 2026-07-24; see `reference/repository-integration.md` |
| Canonical design tokens (authority/execution/dialogue families) | `.agents/specs/frontend/colors_and_type.css` |
| Target route table + guards G1–G9 | `.agents/specs/frontend/sea-forge-gui-view-flow-transition-spec-v0.1.md` §7.3 |
| SFWP framing, preconditions, stale rejection, request recovery | `.agents/specs/frontend/sea-forge-workbench-api-spec-v0.1.md` §4, §8, §12 |
| Full target method catalog (`status: target-unmapped`) | `.agents/specs/frontend/sea-forge-workbench-api-method-catalog-v0.1.yaml` |

### Capability-delta classification (inspection result, 2026-07-24)

| Capability | Status | Evidence / note |
|---|---|---|
| Tauri host | MISSING | no host crate anywhere |
| Bun workspace | MISSING | no package.json/bun.lock outside `.opencode/` |
| React/Vite renderer | MISSING | greenfield |
| Astryx compatibility | UNKNOWN | proven in Task 2 spike |
| SEA Forge theme | MISSING | tokens EXIST (`colors_and_type.css`); theme projection absent |
| Application shell / routing | MISSING | route table specified, nothing built |
| Server transport (socket NDJSON) | EXISTS | `lib.rs:515` |
| Protocol negotiation (`system.hello`) | MISSING | additive verb |
| Request IDs / request-status recovery / idempotency store | MISSING | no correlation state server-side |
| Query contracts (view-shaped: readiness/overview/horizon/run/settlement/capability) | MISSING–PARTIAL | records EXIST; view shaping absent (`Request::Status` is case-status only) |
| Protected commands | PARTIAL | submit/approve/reject/delegate/cancel EXIST; envelopes/preconditions/digests MISSING |
| Event subscriptions / durable cursor / gap recovery | MISSING | ledger `entry_ulid` is the substrate |
| Generated frontend types | MISSING | needs ADR + generator |
| Readiness query | MISSING | no readiness verb |
| Case entry/draft validation/preflight/atomic commit | PARTIAL | `Submit` commits; draft/preflight split MISSING |
| Case overview / horizon / run detail | PARTIAL | records + status EXIST; views MISSING |
| Cancellation | EXISTS (delegation) / PARTIAL (runs) | `CancelDelegation`; run-cancel recovery `lib.rs:148` |
| Retry lineage | PARTIAL | retry-as-new-episode semantics in kernel; no UI surface |
| Agent-run detail / ACP permissions | PARTIAL | ACP types + deny-all mediator EXIST; interactive mediation MISSING |
| Thoth interaction port / AG-UI / CopilotKit | MISSING | `ask` verb EXISTS (sync only) |
| Transcript access | PARTIAL | sealing + retention EXIST; streamed disclosure-gated access MISSING |
| SWE_SEED proof | EXISTS | `swe_seed_reconciliation.rs` + conformance tests |
| Settlement detail / declarations | PARTIAL | records + declarations EXIST; views MISSING |
| Capability detail | PARTIAL | promotion EXISTS; views MISSING |
| Evidence access / integrity inspection | PARTIAL | evidence + ledger proofs EXIST; disclosure-gated UI access MISSING |
| Storybook fixtures / a11y testing / desktop e2e / packaging | MISSING | greenfield |

---

## Task 1 — Repository grounding and compatibility map  (grounding · P0)

**Goal:** The capability-delta table above is re-verified on the executing agent's branch, and every SFWP catalog method has a written grounding verdict (`reuse/adapt/merge/add/reject`) with `file:line` evidence.

**Why this shape:** The API catalog is self-declared `target-unmapped`; implementing any method before grounding it invites speculative endpoints, which the source specs and skill prohibit.

### Steps

1. Re-verify each row of the capability-delta table against the current tree; correct drift — file: `.agents/skills/building-sea-forge-workbench/reference/repository-integration.md` (update in place).
2. Produce the method-grounding table for all catalog families (`system`, `request/operation`, `events`, `cell`, `readiness`, `case`, `run`, `agent_run`, `approval`, `thoth`, `settlement`, `capability`, `evidence`, `integrity`, `memory`, `artifact`, `self_model`) — file: `.agents/reports/2026-XX-XX-sfwp-grounding.md` (new; follow `.agents/reports/` conventions).
3. Record any method rejected for invariant conflict with the exact invariant cited.
4. File discovered out-of-scope debt in `.agents/OBSERVED_DEBT.md`.

### Gate

```bash
test -f .agents/reports/*sfwp-grounding*.md && grep -cE '\|\s*(reuse|adapt|merge|add|reject)\s*\|' .agents/reports/*sfwp-grounding*.md | awk '{exit ($1>=40)?0:1}'
```

**Done when:** every catalog method has a verdict row (≥40 methods covered); deleting a verdict row makes the gate fail.

**Redesign trigger:** If grounding finds an existing subsystem this plan classified MISSING (e.g. a schema generator hiding in a branch), reclassify, shrink the affected task, and note the correction in the report.

---

## Task 2 — Workspace and stack proof: Tauri 2 + Bun + React 19 + Vite + Astryx  (foundation · P0)

**Goal:** A `workbench/` Bun workspace with a Tauri 2 host crate builds, launches a window rendering an Astryx-themed React 19 page using SEA Forge tokens, TanStack Router, TanStack Query, and XState — proving the locked stack coexists before any feature work.

**Why this shape:** Astryx is beta and React 19 peer ranges are unproven; the plan must fail here, cheaply, if the stack cannot coexist — not in Task 5. CopilotKit is deliberately **excluded** from this proof (Task 10 proves it separately; the Workbench must never depend on that proof succeeding).

**Repository evidence:** no existing JS tooling to migrate (Task 1). **Missing capability:** all of it. **Existing substrate reused:** `colors_and_type.css` tokens; devbox/justfile for command wiring.

### Steps

1. Create `workbench/` Bun workspace: `package.json` (`"packageManager": "bun@<exact>"`, workspaces `apps/*`, `packages/*`), `bunfig.toml`, `bun.lock` — files: `workbench/package.json`, `workbench/bunfig.toml`.
2. Scaffold renderer `workbench/apps/desktop/` (Vite + React 19 + TS strict) and Tauri 2 host crate `workbench/apps/desktop/src-tauri/` (own crate, **not** a member of the root Cargo workspace kernel boundary — add a separate `[workspace]` or exclude entry so `just no-async-kernel` semantics are untouched) — record the choice in the ADR.
3. Create token/theme packages: `workbench/packages/sea-forge-ui-tokens/` (copy-projection of `colors_and_type.css` → `sea-forge.tokens.css`, with a drift check against the spec file) and `workbench/packages/sea-forge-astryx-theme/` (`sea-forge.astryx-theme.css` over `@astryxdesign/theme-neutral`).
4. Install pinned deps: `@astryxdesign/core`, `@astryxdesign/theme-neutral`, `@astryxdesign/cli`, `@stylexjs/stylex` (peer), `@tanstack/react-router`, `@tanstack/react-query`, `xstate`, `@xstate/react`, `react-hook-form`, `ajv` — exact versions, no ranges for Astryx.
5. Render a proof page: Astryx button + table themed by SEA Forge tokens, one typed route with validated search param, one query, one trivial machine.
6. Wire scripts: `bun run dev`, `bun run build`, `bun run check` (tsc + eslint or biome per Bun compat), `bun run test` (Vitest); add `just workbench-check` delegating to Bun — file: `justfile`.
7. Write the stack ADR — file: `docs/decisions/ADR-004-workbench-stack.md` (locked stack, exclusions, Bun-not-shipped rule, any Node tooling exception found, with removal path).
8. Add `workbench/AGENTS.md` describing local commands, boundaries (renderer never touches socket/FS), and generated-zone rules.

### Gate

```bash
cd workbench && bun install --frozen-lockfile && bun run check && bun run build && cargo build --manifest-path apps/desktop/src-tauri/Cargo.toml && bun run test
```

**Done when:** the gate exits 0 and `bun run tauri dev` shows the themed proof page; removing the Astryx theme import visibly breaks the page (teeth: tokens are actually flowing). devbox `just check`/`just test` still green (kernel untouched).

**Redesign trigger:** Astryx cannot theme under React 19 or requires StyleX authoring → fall back to Astryx precompiled CSS + CSS Modules only, document the reduced substrate in the ADR, and continue (do not switch to an excluded framework).

---

## Task 3 — SFWP transport: envelopes, negotiation, request recovery, events, generated contracts  (protocol · P0)

**Goal:** The server speaks additive SFWP: `system.hello` negotiation, enveloped requests with `request_id`, `request.get_status` recovery, `events.subscribe`/`events.get_range` over a ledger-`entry_ulid` cursor, and precondition digests on protected verbs — with Rust→JSON Schema→TS generation making the contracts typed end to end, and the Tauri host owning socket, correlation, and reconnect.

**Why this shape:** ADR-003 requires additive evolution (old clients/servers fail cleanly on unknown verbs/frames); the ledger's `entry_ulid` is already the durable ordered identity, so the event cursor grounds on it instead of inventing a parallel sequence. The host owns transport so the renderer stays inside the typed bridge.

**Repository evidence:** `lib.rs:396` enum seam; `ledger/src/types.rs:206`; no correlation store (Task 1). **Missing capability:** everything in the goal. **Existing substrate reused:** existing verb handlers stay untouched; new SFWP methods delegate to the same shared services.

### Steps

1. Add SFWP frame/envelope types alongside the existing enum (new `sfwp` module; `Request` enum gains an `Sfwp(SfwpFrame)`-style additive verb or a parallel first-line `hello` detection — choose in-code with a comment citing ADR-003) — file: `crates/sea-forge-server/src/sfwp/mod.rs` (new).
2. Implement `system.hello`/`system.describe`/`system.get_schema` returning protocol version, method catalog subset actually implemented, and schema refs.
3. Implement a bounded request-correlation store (durable enough to answer `request.get_status` across reconnect; persisted under the run root, never in `.sea-forge/` source zones) and thread `request_id` through protected verb handling.
4. Implement `events.subscribe` (opaque cursor = sealed `entry_ulid` position), `events.unsubscribe`, `events.get_range` for gap recovery; stream frames as `event` lines on the subscribed connection.
5. Add precondition support (`policy_bundle_digest`, per-record `{ref, expected_digest}`) with `rejected_as_stale` structured rejection on protected methods that Task 6+ will use.
6. Add the schema-generation pipeline: `schemars` (or justified equivalent — ADR update) derives on the SFWP surface types + relevant `sea-forge-core` types; a generator binary/xtask emits JSON Schema; `bun run generate:contracts` emits TS + AJV validators into `workbench/packages/contracts/generated/` (checked in, drift-tested).
7. Implement the Tauri host socket client: connect, hello, typed commands (`sfwp_query`, `sfwp_command`, `sfwp_request_status`), event channel with cursor persistence and reconnect+gap-recovery loop — files: `workbench/apps/desktop/src-tauri/src/{socket.rs,bridge.rs,events.rs}`.
8. Server conformance tests for each new verb incl. no-side-effect-on-deny and stale rejection; host integration tests for correlation recovery across a killed connection — files: `crates/sea-forge-server/tests/conformance_sfwp.rs`, `workbench/apps/desktop/src-tauri/tests/bridge.rs`.

### Gate

```bash
devbox run -- just test && cargo test --manifest-path workbench/apps/desktop/src-tauri/Cargo.toml && cd workbench && bun run generate:contracts && git diff --exit-code packages/contracts/generated && bun run test
```

**Done when:** a scripted client can hello→subscribe→kill connection mid-command→reconnect→`request.get_status`→resume events from cursor with a deliberately induced gap recovered via `events.get_range` (this scenario is a named test); changing a Rust contract type without regenerating fails the drift test.

**Redesign trigger:** If the single-connection NDJSON model cannot carry interleaved responses+events cleanly, split into a command connection and a subscription connection per client (both host-owned) rather than inventing multiplexing framing beyond the spec.

---

## Task 4 — Application shell and semantic design foundations  (shell · P1)

**Goal:** The desktop app renders the governed shell — sidebar (13 top-level surfaces), governed-focus main region, right evidence drawer skeleton, keyboard map — plus the SEA Forge semantic component set and route guards G1–G9 wired to real hello/readiness data where it exists.

**Why this shape:** Every later slice mounts into this shell; semantic components (`GovernedStatusPill`, `DualStateIndicator`, `SourceFreshnessBadge`, `IntegrityIndicator`, `ProtectedActionButton`, `WhyStatePanel`, `EvidenceDrawer`, `AuthorityBoundaryPanel`, `AvailabilityLadder`) must exist before organisms, or generic Astryx patterns will leak domain meaning.

**Existing substrate reused:** Task 2 theme; Task 3 bridge; wireframe spec regions; `ui_kits/app/components/*.html` contracts. **Missing capability:** all shell/components.

### Steps

1. Implement shell layout + navigation routes (TanStack Router file routes per view-flow module names) — files: `workbench/apps/desktop/src/routes/`.
2. Implement the nine semantic components with CSS Modules over Astryx primitives — package: `workbench/packages/sea-forge-ui-components/`.
3. Implement guards G1–G9 as router context guards fed by hello/session state; denial renders a governed denial surface, never a blank route.
4. Storybook: stories for every semantic component state (each authority/execution/dialogue/settlement/integrity variant) — `workbench/packages/sea-forge-ui-components/stories/`.
5. axe-core checks in component tests; keyboard traversal test for the shell.

### Gate

```bash
cd workbench && bun run check && bun run test && bun run build-storybook
```

**Done when:** shell navigates by keyboard alone; every semantic component has stories for all states and axe passes; rendering `GovernedStatusPill` with an unknown variant renders `unknown` (not success) — a test proves it.

**Redesign trigger:** none plausible (pure composition over proven stack).

---

## Task 5 — Readiness vertical slice  (first settlement · P1)

**Goal:** An operator opens the app and sees operation-sensitive readiness with source freshness, integrity, a named limitation when degraded, and an honestly disabled case-creation action with reason — backed by a real `readiness.get` server view, live-invalidated by events.

**Why this shape:** Readiness is the specified first primary-path surface (`context-provenance.md` derivation) and exercises every foundation piece (query, events, semantic components, guards) with no mutation risk.

**Existing substrate reused:** kernel readiness inputs (policy load, self-model validation, endpoint probes — grounded in Task 1 report). **Missing capability:** `readiness.get` view + UI.

### Steps

1. Implement `readiness.get` as an SFWP inspect method shaping existing kernel facts into an operation-sensitive view (no new truth, projection only) — files: `crates/sea-forge-server/src/sfwp/readiness.rs` + conformance test.
2. Regenerate contracts; add `ReadinessConsole` organism + route with TanStack Query + event invalidation — files: `workbench/apps/desktop/src/routes/readiness/`, `workbench/packages/sea-forge-ui-patterns/`.
3. Fidelity pass against `ui_kits/app/index.html` structure and the wireframe acceptance criteria; a11y pass.
4. Playwright journey: launch → degraded readiness → inspect Why/Evidence → disabled case creation shows reason.

### Gate

```bash
devbox run -- just test && cd workbench && bun run test && bun run e2e -- --grep "readiness"
```

**Done when:** the journey passes against a real server on a temp root; switching intended operation changes readiness (operation-sensitivity has a test); killing the server mid-view marks the view stale rather than blank.

**Redesign trigger:** If readiness facts prove too scattered for one view, ship the subset with explicit `unknown` sections (unknown ≠ unavailable) and record the gap — do not fabricate completeness.

---

## Task 6 — Case authoring: draft, validation, preflight, atomic commit  (authoring · P1)

**Goal:** Draft → validate → preflight → commit works end to end with local reversible drafts, AJV hints, authoritative server preflight, precondition digests, stale rejection with repair path, and ambiguity recovery — no duplicate commit reachable.

**Why this shape:** This is the first protected command path; it establishes the XState preflight/commit machine and the digest discipline every later mutation reuses. Draft storage decision (Tauri store vs SQLite) triggers here — run the spike from the skill's deferred-decisions table before choosing.

**Existing substrate reused:** `Request::Submit`/`SubmitPayload` commit path; planner validation + criteria provenance (`crates/sea-forge-planner/`); hash conventions (`compute_record_hash`). **Missing capability:** `case.entry_options`, `case.preflight`, draft store, preflight/commit split.

### Steps

1. Ground and implement `case.entry_options`, `case.preflight` (evaluate; audit-only records), and envelope `case.commit` over the existing submit path with preconditions — files: `crates/sea-forge-server/src/sfwp/case.rs` + conformance tests (stale rejection, no side effect on reject).
2. Draft store per spike outcome (local, versioned, reversible, never `.sea-forge/`) behind Tauri commands.
3. `CaseCreationWorkbench` + `PlanPreflightPanel` organisms; react-hook-form + generated AJV schemas; XState machine `draft → validating → preflight_ok → committing → committed | rejected_as_stale | ambiguous → status_recovery`.
4. Machine tests: duplicate commit unreachable from `ambiguous`; stale rejection routes to re-preflight; Playwright: commit succeeds; policy mutated between preflight and commit yields the repair path.

### Gate

```bash
devbox run -- just test && cd workbench && bun run test && bun run e2e -- --grep "case-authoring"
```

**Done when:** proof scenarios 6 (ambiguous commit, no duplication) and 7 (stale preflight) pass as named e2e tests; deliberately re-enabling commit from `ambiguous` in the machine makes the model test fail.

**Redesign trigger:** If existing `SubmitPayload` cannot carry preflight digests without breaking CLI compat, add a parallel enveloped commit verb and leave `submit` untouched (additive), noting the eventual convergence in OBSERVED_DEBT.

---

## Task 7 — Case overview and horizon  (operations · P2)

**Goal:** Committed cases render an Overview (summary, sentries, criteria provenance) and a live Horizon board (items, states, lawful actions) driven by the case event stream with cursor continuity.

**Existing substrate reused:** case records/reducers, `Request::Status`, Task 3 events. **Missing capability:** `case.get_summary`/`case.get_overview`/`case.get_horizon` views + UI.

**Why this shape:** Overview/horizon are pure projections over records that already exist; this task proves live event reduction at scale before execution surfaces depend on it.

### Steps

1. Implement the three inspect views over existing records + conformance tests — file: `crates/sea-forge-server/src/sfwp/case_views.rs`.
2. `CaseHorizonBoard` organism (Astryx Table default; virtualization only if row counts demand it), case-scoped nav (Overview/Horizon/Timeline/Plan/Runs/Agent Tasks/Settlement/Evidence).
3. Event-reduced live view with authoritative refetch on gap; URL-addressable filters via validated search state.
4. Playwright: run a case; watch horizon transition without refresh; kill/restart server; horizon recovers via cursor + refetch.

### Gate

```bash
devbox run -- just test && cd workbench && bun run test && bun run e2e -- --grep "case-horizon"
```

**Done when:** proof scenario 8 (event gap → authoritative recovery, no silent loss) passes as a named test; an injected out-of-order event is not rendered ahead of refetch.

**Redesign trigger:** none plausible (projection-only).

---

## Task 8 — Command execution console and settlement handoff  (execution · P2)

**Goal:** Run Detail shows execution state, output/log streams, cancellation, and retry lineage — always rendering execution and settlement as separate domains, with the settlement handoff visible when execution completes.

**Existing substrate reused:** run records, trace events, cancellation recovery (`lib.rs:148`), retry-as-new-episode kernel semantics. **Missing capability:** `run.get`, log/content streams, `ExecutionConsole`.

**Why this shape:** Execution truth is kernel-owned; the console only renders governed records and streams — `execution state ≠ settlement state` is enforced structurally by `DualStateIndicator`.

### Steps

1. Implement `run.get` + bounded log/content streaming (disclosure-gated, host channel) + conformance tests.
2. `ExecutionConsole` organism with dual-state display, `run.cancel` protected command (no optimistic completion — cancelled only when the record says so), retry linking to the new episode.
3. Playwright: happy path (scenario 1); execution succeeds/settlement rejects rendered separately (scenario 2).

### Gate

```bash
devbox run -- just test && cd workbench && bun run test && bun run e2e -- --grep "execution"
```

**Done when:** scenarios 1 and 2 pass as named tests; a UI that renders "success" from process exit alone is impossible — the component test asserts settlement state comes only from settlement records.

**Redesign trigger:** none plausible.

---

## Task 9 — Thoth interaction port and direct AG-UI adapter  (thoth · P2)

**Goal:** Ask Thoth works through `ThothInteractionPort` + `DirectAgUiAdapter`: typed question kinds, streamed answer assembly over a host channel, clarification round-trips, governed denials rendered as denials, and every generative element resolving to the registered `ThothRenderable` union.

**Existing substrate reused:** `Request::Ask` → `service::ask` (`service.rs:258`), protocol types, disclosure engine. **Missing capability:** streaming presentation, port, adapter, renderable registry.

**Why this shape:** The port isolates all agent-interaction tech churn; the direct adapter is canonical so CopilotKit (Task 10) can never become load-bearing. Contract: `reference/thoth-interaction-contract.md`.

### Steps

1. Ground `thoth.list_question_kinds`/`thoth.get_answer` (inspect) additively; keep `ask` the single governance path; stream assembly happens presentation-side over the answer/evidence content.
2. Implement the port, adapter, renderable registry (`grounded-answer`, `evidence-list`, `action-paths`, `approval-proposal`, `clarification`) with an unsupported-kind fallback.
3. Ask Thoth route: composer (typed kinds), answer history, disclosure-denial rendering (named claim classes, not error toast).
4. Port-level test suite (this same suite must pass for any future adapter); Playwright: ask → grounded answer with evidence refs; ask restricted → governed denial (scenario 9: no content retrieval before disclosure decision — assert no content frames on the wire).

### Gate

```bash
devbox run -- just test && cd workbench && bun run test && bun run e2e -- --grep "thoth"
```

**Done when:** scenario 9 passes; the port test suite is adapter-parameterized (teeth for Task 10/eval 4); no component imports the adapter directly (lint rule).

**Redesign trigger:** If AG-UI protocol mapping onto NDJSON events proves awkward, the adapter may speak a dedicated host channel format — the port contract, not the wire, is the stable boundary.

---

## Task 10 — Optional OSS CopilotKit adapter  (thoth · P3, skippable)

**Goal:** `CopilotKitAdapter` passes the same port-level suite using only open-source, locally runnable CopilotKit packages — and removing it changes nothing outside the adapter.

**Why this shape:** Development convenience only. The Workbench must never depend on this task succeeding; it may be skipped entirely without renumbering.

### Steps

1. Implement the adapter behind the port; register the same renderables; no CopilotKit type escapes the adapter directory (lint rule + grep test).
2. Run the adapter-parameterized port suite against it; document the OSS-only dependency set in the ADR.
3. Removal drill (eval 4): disable the adapter, run all Thoth e2e against the direct adapter, restore.

### Gate

```bash
cd workbench && bun run test -- --grep "port-suite" && bun run e2e -- --grep "thoth" && ! grep -rn "@copilotkit" apps packages --include="*.ts*" | grep -v adapters/copilotkit
```

**Done when:** both adapters pass the same suite; the removal drill (scenario 11) is scripted and green.

**Redesign trigger:** Any CopilotKit functionality requiring Copilot Cloud or enterprise features → drop that functionality, not the boundary.

---

## Task 11 — Agent delegation, ACP permissions, and transcript evidence  (delegation · P2)

**Goal:** Agent Task console shows delegated runs (dialogue, permission requests, termination, settlement — four separate domains), lets an operator decide permissions interactively (deny ≠ cancel), and exposes sealed transcript evidence with disclosure gating.

**Existing substrate reused:** `Delegate`/`CancelDelegation` verbs, `delegation.rs`, ACP mediation trait (`acp.rs:77` — implement an interactive mediator where `DenyAllMediator` is the fail-closed default), transcript sealing/retention, SWE_SEED declaration ingress. **Missing capability:** `agent_run.get`, permission event surfacing, interactive mediator, transcript streaming UI.

**Why this shape:** Permission mediation already has a trait seam; the Workbench supplies a mediator implementation routed through SFWP — no parallel permission model.

### Steps

1. Surface permission requests as events + `agent_run.permission.decide` SFWP method mapping to `PermissionDecision`; unavailable mediator stays deny (fail-closed conformance test).
2. `AgentTaskConsole` organism: dialogue stream, permission queue, termination state, settlement state — visually separate; permission machine (deny leaves run running — scenario 4 test).
3. Transcript evidence drawer over sealed transcripts (disclosure before retrieval).
4. Playwright: delegation with a permission denial that does not cancel (scenario 4); transcript access allowed/denied paths.

### Gate

```bash
devbox run -- just test && cd workbench && bun run test && bun run e2e -- --grep "agent-task"
```

**Done when:** scenario 4 passes; the run demonstrably continues post-denial; mediator-unavailable test proves deny with a governed record.

**Redesign trigger:** If interactive mediation cannot reach the running ACP session without kernel changes, park the decision UI on the approval surface (governed parking already exists) instead of adding a side channel.

---

## Task 12 — Settlement and recovery surfaces  (settlement · P2)

**Goal:** Settlement Detail renders criterion-by-criterion evaluation (`CriterionSettlementMatrix`), declarations with SoD context, approval escalation (`ApprovalDecisionPanel` on the Inbox), and recovery paths (retry lineage as new episodes).

**Existing substrate reused:** settlement records/declarations, approvals + SoD checks, `Approve`/`Reject` verbs. **Missing capability:** `settlement.get`, `approval.list/get/decide` envelope, matrix/panel organisms.

### Steps

1. Ground `settlement.get`, `approval.list`/`approval.get`, merge approve/reject into enveloped `approval.decide` (existing verbs stay for CLI compat) + conformance tests (expiry, SoD deny, no side effect).
2. Organisms + routes; approval decisions always via `approval.decide` with preconditions (unchanged, unexpired).
3. Playwright: escalation path (scenario 3); settlement rejection detail (pairs with scenario 2).

### Gate

```bash
devbox run -- just test && cd workbench && bun run test && bun run e2e -- --grep "settlement|approval"
```

**Done when:** scenario 3 passes; deciding an expired approval renders the typed expiry outcome (test with teeth: fast-forwarded TTL).

**Redesign trigger:** none plausible.

---

## Task 13 — Capability, memory, evidence, and integrity  (memory · P3)

**Goal:** Capability Detail (conservative states, promotion evidence, next spendable proof path), governed memory recall, Evidence browser, and Integrity Inspector (ledger proof verification) are inspectable surfaces.

**Existing substrate reused:** capability promotion, memory recall (`crates/sea-forge-cli/src/commands/recall.rs` pattern), evidence crate, ledger MMR proofs (`prove_entry`). **Missing capability:** the four view families + `integrity.verify` surfacing.

### Steps

1. Ground and implement `capability.get`, `memory.recall` (protected retrieval), `evidence.get` (disclosure-gated), `integrity.verify` views + conformance tests.
2. `CapabilityPromotionPanel`, `AvailabilityLadder`, `IntegrityInspector`, evidence browser routes.
3. Playwright: capability stays conservative after insufficient evidence (scenario 10); integrity halt renders as integrity domain (scenario 5).

### Gate

```bash
devbox run -- just test && cd workbench && bun run test && bun run e2e -- --grep "capability|integrity|evidence"
```

**Done when:** scenarios 5 and 10 pass as named tests; a tampered ledger entry makes the Integrity Inspector show `integrity_failed` (teeth: test mutates a copy of the ledger).

**Redesign trigger:** none plausible.

---

## Task 14 — Packaging, accessibility, and end-to-end proof  (release · P1)

**Goal:** A packaged desktop bundle (Tauri binary + OS WebView + compiled assets, **no Bun**) passes the full proof-scenario suite, the accessibility contract, and repository release checks.

### Steps

1. Tauri bundling config, desktop permission scopes (minimal FS/socket), icons per asset boundary (none fabricated — placeholder documented).
2. Bundle inspection test: assert no `bun` binary/runtime in the artifact (scenario 12).
3. Full a11y pass: keyboard-only primary path, 200% zoom, reduced motion, axe across routes (Playwright).
4. Run all 12 proof scenarios as one suite against a packaged build + real server; wire into `just workbench-e2e`.
5. Update `.agents/CURRENT_STATUS.md`, close plan checkboxes, record residual debt.

### Gate

```bash
cd workbench && bun run build && bun run tauri build && bun run e2e:packaged && ! (tar tf target/**/bundle/* 2>/dev/null | grep -i "bun") && devbox run -- just check && devbox run -- just test
```

**Done when:** all 12 proof scenarios green against the packaged app; scenario 12 (no Bun shipped) verified by artifact inspection.

**Redesign trigger:** Platform-specific packaging failures are reported as skipped-with-reason per AGENTS.md — never as passed.

---

## Final acceptance checklist (whole plan)

- [ ] Every SFWP catalog method has a grounding verdict; rejected methods cite invariants *(Task 1)*
- [ ] Locked stack proven coexisting; Astryx-theme removal visibly breaks the proof page *(Task 2)*
- [ ] hello→kill→reconnect→status→cursor-resume→gap-recovery scenario green; contract drift test has teeth *(Task 3)*
- [ ] Shell + semantic components; unknown enum variant renders `unknown`, never success *(Task 4)*
- [ ] Readiness slice: operation-sensitive, degraded-but-usable, real server, stale-on-disconnect *(Task 5)*
- [ ] Scenarios 6+7: no duplicate commit from ambiguity; stale preflight repair path *(Task 6)*
- [ ] Scenario 8: event gap recovered authoritatively, no silent loss *(Task 7)*
- [ ] Scenarios 1+2: happy path; execution-success/settlement-reject rendered separately *(Task 8)*
- [ ] Scenario 9: disclosure denial before retrieval; port suite adapter-parameterized *(Task 9)*
- [ ] Scenario 11: CopilotKit removal drill green (or task skipped entirely) *(Task 10)*
- [ ] Scenario 4: permission denial without cancellation; fail-closed mediator *(Task 11)*
- [ ] Scenario 3: approval escalation; expired approval renders typed expiry *(Task 12)*
- [ ] Scenarios 5+10: integrity halt as integrity; capability stays conservative *(Task 13)*
- [ ] Scenario 12: packaged bundle contains no Bun; full suite green on packaged build *(Task 14)*
- [ ] `devbox run -- just check`, `just test`, `just context-check` exit 0 throughout.
- [ ] `workbench/packages/contracts/generated` rebuilt and committed with every contract change.
- [ ] Spec-minimum proofs P1–P4b and the active full-spec milestone gate remain green.

## Guardrails (do not violate)

- **Every kernel invariant in the skill's list** (`navigation ≠ mutation` … `retry ≠ replay`) survives every task; a slice that weakens one fails review regardless of its gate.
- The renderer never touches the Unix socket, `.sea-forge/` files, or SQL; no generic `invoke_backend(method, arbitrary_json)` command.
- Server additions are additive (ADR-003): existing verbs keep their behavior; old clients fail cleanly.
- No new dependency without its ADR (ADR-002 discipline); Astryx pinned exact; excluded list (Tailwind, shadcn, Redux, Next.js/SSR, GraphQL-primary, Copilot Cloud, CopilotKit-owned state/persistence/auth) is absolute.
- CopilotKit work never blocks or reorders Tasks 1–9 and 11–14.
- The static kit (`ui_kits/app/`) is visual reference; its JS is never imported; the generated `DESIGN-MANIFEST.json` never defines routes.
- Do not fabricate authority, evidence, scores, criteria, confidence, or options in fixtures presented as real data.
- Keep the frontend spec package read-only during implementation; corrections go through `reference/source-map.md`.
- Commit hygiene: server (Rust) changes and frontend changes land in separate commits with their own gates; do not push or open PRs unless asked.
