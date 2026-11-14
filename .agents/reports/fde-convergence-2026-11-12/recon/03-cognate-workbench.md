# 03 Recon: Cognate, Cognate Workbench, Horizon boundary

Read-only. Tags: OBSERVED (saw code/doc/git output) or INFERRED. `C:` = `/home/sprime01/projects/`.

## A. Cognate (GodSpeedAI/Cognate)

### A1. Identity
| Worktree | Branch | HEAD | vs origin/main (ahead/behind) | Dirty | Purpose |
|---|---|---|---|---|---|
| C:cognate | security/repo-hygiene-2026-10-31 | 87333bc | 0 / 2 | clean | Merged PR #74 (hygiene, history rewrite, status rev 78) |
| C:cognate-horizon | feat/realitytrace-real-snapshot | ead573a | 1 / 0 | untracked node_modules only | Runs begin on created RealityTrace snapshot, genesis fallback |
| C:cognate-identity-wt | horizon/shared-identity | f4d4db6 | 2 / 3 | untracked node_modules only | New `packages/identity` (operator, pairing, WebAuthn) |
| C:gsr-heads/cognate | detached | bcef663 | n/a | n/a | Pinned readiness head (release #72) |

origin/main ~ bcef663 (INFERRED). Both feature branches have remote-tracking refs (OBSERVED).

### A2. Role and ownership
Runtime SDK: "define meaning once, project it everywhere" (`README.md`). Six planes in `docs/architecture.md`.
- **Owns:** run lifecycle, durable event log (store-sqlite/postgres/memory), continuations (interrupt/resume, `human_approval` kind), capability eligibility and invocation policy, shared thread state with `expectedVersion`, content-hashed artifacts, cognition/recall, lease-bound remote offers (delegation), observation outbox.
- **Does not own:** UI (renderers only: `react`, `ui-astryx`), authority decisions (SEA-Forge decides), cost ledger, skill authoring (agent-discovery is read-only), first-class "session" (Workbench maps sessions onto `correlationId`), planning.

### A3. Implemented capabilities
| Capability | Evidence | Tests | Status |
|---|---|---|---|
| RuntimeService (startRun, getRun, cancelRun, events, invoke, readProjection, listRuns, shared state, observe, remote offers) | `packages/runtime-api/src/types.ts:229-300` | runtime-api `conformance.ts`; runtime-bun tests | IMPLEMENTED (OBSERVED) |
| Resume and cancel of continuations, idempotent outcomes | `types.ts:130-140`, `packages/continuations` | continuations tests | IMPLEMENTED |
| Event stream resumable by position | `types.ts` events() doc | `packages/protocol-connect/test/service.test.ts` | IMPLEMENTED |
| SEA-Forge authority client (NDJSON socket, `godspeed.authority_request` v1.0.0; unknown refused) | `runtime-bun/src/sea-forge-authority.ts:1-30`, `governed-operation.ts:189` | not checked | IMPLEMENTED in code; live path needs SEA-Forge |
| Remote offers (publish/list/revoke) | `runtime-bun/src/remote-offers.ts` | not checked | IMPLEMENTED |
| Recall (`run.recall`) | `runtime-bun/src/cognition.ts`, `packages/cognitive-*` | cognitive tests | IMPLEMENTED |
| Slots and pre-claim budget | `runtime-bun/src/runtime.ts:45`, `execution-slots.ts` | not checked | PARTIAL (concurrency, not money) |
| Capability `cost?` field | `packages/capabilities/src/types.ts:47` | none | SPEC-ONLY (metadata) |
| RealityTrace observation outbox | `packages/realitytrace` | `test/sxr-cli.test.ts` (branch) | PARTIAL |
| Identity package | `packages/identity` on horizon/shared-identity only | 4 test files on branch, not run | UNVERIFIED; not on main |

Tests: 102 test files under `packages/*/test` (OBSERVED). `NOT_RUN` in docs: 0 hits.

### A4. Interfaces
- TS contract `runtime-api/src/types.ts:229`. Connect RPC `packages/protocol-connect` (+`-bun`). Protobuf `proto/cognate/v1/*.proto` with generated `packages/proto/src/gen/cognate/v1/*_pb.ts`. AG-UI SSE `packages/ag-ui`. MCP `packages/mcp`. A2A `packages/a2a` (continuation maps to `input-required`). CLI `packages/cli`.
- SEA-Forge env: `COGNATE_GOVERNANCE`, `COGNATE_SEA_FORGE_SOCKET`, `COGNATE_WORLD_REF`, `COGNATE_SEA_FORGE_ACTOR` (`governance-env.ts:6-9`).
- Run status: `running|waiting|completed|failed|cancelled`. Continuation: `OPEN|RESUMED|CANCELLED|EXPIRED|FAILED|BLOCKED`.
- Events (OBSERVED): `run.*` (start, step, waiting, resume, cancel, completed, failed, invocation, recall, remote_offers, governance.escalated), `state.changed`, `observation.*`, `capability.provided|withdrawn`, `continuation.*`.
- IDs: `runId` (generated), `eventId` randomUUID, `worker-<uuid>` owner, `recoveryEpoch` UUID, caller `idempotencyKey`, `correlationId` (Workbench uses `session:<id>`, `agui-thread:<id>`), `threadId`, `continuationId`, `offerId`, `leaseId`, `artifactId`.
- `observe()` is in-process only, not on Connect (`types.ts` doc).

### A5. Integrations (Cognate side)
| System | Mechanism | Evidence |
|---|---|---|
| DomainForge | `.sea` source, binding 0.21.1 | `docs/semantics.md` |
| SEA-Forge | authority socket; settlement stays in SEA-Forge | `sea-forge-authority.ts`, `docs/` |
| RealityTrace/SXR | SXR CLI adapter, snapshot branch | `packages/realitytrace/src/sxr-cli.ts:111` |
| Gauntlet, Context Kernel, Casework | 0 hits | grep |
| SWE_SEED | 0 hits | grep |
| Horizon | 0 real hits | grep |
Cost accounting is NOT in Cognate; spend ceilings live in Workbench (`src/workbench/economics.ts`, `docs/known-gaps.md`).

A6. UX: `ui-astryx` reference surfaces only; no product UI (OBSERVED).

### A7. Unfinished
- `feat/realitytrace-real-snapshot`: one commit, touches `sxr-cli.ts` (+23) and test (+90). No run evidence.
- `horizon/shared-identity`: `ed6c52a` adds identity (15 files, +1454); `f4d4db6` fixes SQLite collection names. No consumer in repo (grep empty). README says it is "one definition of the operator for Horizon and Workbench" (OBSERVED). Package scope is `@godspeed/identity`, unlike `@godspeedai/cognate-*` (OBSERVED).

## B. Cognate Workbench (GodSpeedAI/Cognate-Workbench)

### B1. Identity
| Worktree | Branch | HEAD | vs origin/main | vs origin desktop | Dirty | Purpose |
|---|---|---|---|---|---|---|
| C:cognate-workbench | feat/desktop-terminal-browser | e45e97d | 140 / 8 | 19 / 0 | 37 modified PNGs (0 text lines) | Tauri desktop, PTY terminal, managed browser, capability runtime |
| C:cognate-workbench-horizon | feat/horizon-connect | c4096b9 | 75 / 8 | 11 / 57 | clean | Horizon connect, case-step proposal, RealityTrace observations, desktop B1-B4 |
| C:gsr-heads/cognate-workbench | detached | 1b6fcb4 | n/a | n/a | n/a | Pinned head, includes passkey mobile remote (#20) |
| /tmp/wb-baseline | detached | 448a095 | n/a | n/a | n/a | Stale, prunable; untouched |

- horizon-connect has no origin upstream. Of its 11 unique commits, 7 are not patch-equivalent to origin desktop (`git cherry`): 35ee1cf, 78c4bb2, fb7104f, a25f895, a9bf1f7, 9695cad, c4096b9. Origin desktop has 57 commits horizon-connect lacks (Windows fix a600e32, E1 forensics e6de37a).
- origin/main is not an ancestor of the desktop branch.

### B2. Role and ownership
"Open-source coding agent harness for work that somebody is accountable for" (`README.md`). Browser UI over local directories on Cognate SDK.
- **Owns:** operator cockpit (tasks, contracts, review gate, workflows, capabilities, automations, experts/teams, schedules, spend display), human terminal and browser sessions, mobile remote, Horizon connect action, `.sea` domain model (`domain/`).
- **Projects from Cognate:** `workbench` projection and `workbench.*` events (27 families). Terminal/browser metadata is durable; stream bytes never persisted (`src/terminal/README.md`).
- **Does not own:** providers, execution contexts, authority decisions (case keeper and SEA-Forge).

### B3. Implemented capabilities
| Capability | Evidence | Tests | Status |
|---|---|---|---|
| Tasks, contracts, review gate | `src/ui/workbench/Workbench.tsx`, `ReviewCard.tsx` | `test/J-WB-task-contract`, `J-WB-review-gate` | IMPLEMENTED (names only; not run) |
| Steering | `src/workbench/steering.ts` | `J-WB-steering` | IMPLEMENTED |
| Economics, fail-closed ceilings | `src/workbench/economics.ts` | `J-WB-economics`, `ui-observability` | PARTIAL (planning economics informational) |
| Workflows/graph | `src/workbench/workflow.ts` | `J-WB-workflow` | PARTIAL (parallel plans not started) |
| Experts/Teams, schedules | `src/ui/experts/*`, `src/schedules/` | experts reports | IMPLEMENTED (`docs/experts-and-teams.md`) |
| Terminal (PTY local/WSL/SSH) | `src/terminal/*`, `TerminalPanel.tsx` | not enumerated | PARTIAL (device pass pending) |
| Managed browser, inspect/annotate | `src/browser/*`, `BrowserPanel.tsx` | `E2E-browser.test.ts` | PARTIAL |
| Previews | `src/preview/` (2 files) | none listed | PARTIAL (T12 core) |
| Mobile remote QR + passkey | `src/remote/*` (9 files), `server.ts:1021-1028` | `E2E-mobile-remote`, `remote-webauthn`, `remote-pairing` | IMPLEMENTED on main; not run |
| Connect Horizon | `src/capabilities/connectHorizon.ts:35-61` | `J-AUTH-CONNECT-HORIZON` | IMPLEMENTED, horizon-connect only |
| Agents propose case steps | `src/capabilities/caseProposeStep.ts:67-90` | `J-CASE-propose-step` | IMPLEMENTED, horizon-connect only; refuses without governance |
| RealityTrace observations | `src/observations.ts` | `J-RT-realitytrace`, `J-COG-realitytrace` | IMPLEMENTED on branch; pins SXR v0.3.0 |
| Capability runtime | `.agents/plans/capability-bundles/HANDOFF.md`, `src/capabilities/capabilityRuntime.ts` | `J-CAP-*` | PARTIAL (tranches P, M, T1 committed) |
| Tauri shell | `src-tauri/src/{backend,lib,main,menu,title}.rs` | `scripts/qualify-tauri.sh` | PARTIAL; no `#[tauri::command]` (OBSERVED: none) |

Tests: 203 files under `test/`. Rev 37 recorded 1564-test gate green. Rev 42 marks menu and dialog rows BLOCKED on xdotool. No `NOT_RUN` strings. horizon-connect commit notes 3 failures when pg0 is missing.

### B4. Interfaces
- Cognate packages pinned `0.8.0` on desktop branch (`package.json`); origin/main adopted Cognate 0.9.0 (`39700cb`, PR #29).
- Named capabilities validated server-side (`src/ui/workbench/README.md`); 38 mounted components; 18 public projections.
- Session deep link `#session/<id>[/<eventId>]` (`src/ui/sessionLink.ts`). Mobile URL `/m#p=<43-char base64url>` (`E2E-mobile-remote.test.ts:95`).
- Remote HTTP: `/remote/status|start|refresh|stop` (`server.ts:1025-1028`). Browser websocket `/browser/stream`. Remote state thread `auth:remote:v1` (`src/remote/state.ts:10`).
- Horizon key: `workbench.key` (0600) and `workbench.key.json` `{sessionId, expiresAt}` in secretsDir (`connectHorizon.ts:56-58`); TTL 1h to 90d.
- SEA-Forge case frame: `case_add_item`, `agent_task` operation (`caseProposeStep.ts:56-76`).

### B5. Integrations (Workbench side)
| System | Mechanism | Direction | Evidence |
|---|---|---|---|
| Cognate runtime | Connect RPC and packages | consumes | `package.json` |
| SEA-Forge | governance socket; settlement | asks | `governance.ts`, `src/settlement/` |
| Case keeper | `case_add_item` over socket | proposes | `caseProposeStep.ts` |
| DomainForge | `domain/index.ts` | consumes | `domain/README.md` |
| RealityTrace | `observations.ts:22` | emits | J-RT tests |
| SWE_SEED | `sweSeed.ts` proof source `any` or `swe_seed` | reads | `COGNATE_SWE_SEED_BIN` |
| Gauntlet, Context Kernel | Gauntlet 12 hits, not traced; Context Kernel 0 | unclear | grep |

### B6. UX surfaces
- Views are state, not URL routes: `App.tsx` `view === "split"`, `view: "conversation"`. Hash deep link `#session/`. Mobile at `/m`.
- Cockpit `Workbench.tsx` tabs: tasks, workflows, capabilities, automations. Other panels: GovernancePane, RemotePanel, ConnectHorizon, Experts/Teams, schedules, spending, cognitive console, inspector, TerminalPanel, BrowserPanel.
- Design: `DESIGN.md` ("Cognate Console"); tokens `var(--cg-*)` in `src/ui/app.css`.
- Interaction grammar: `src/ui/workbench/UX.md` 13 hard rules. Presentational only; tokens only; one filled button per surface; state as words; unknown shows `unknown`/`n/a`, never zero; 360px; reduced motion. Enforced in part by `ui-observability.test.tsx` and `native-select-guard`.

### B7. Unfinished
- **Desktop branch 37 dirty files:** all binary PNG re-captures (`.agents/reports/ui-production-readiness/evidence/after/{t2,t4,u5}` and `screenshots/{concurrency,experts-debt,experts,red-journeys,workbench}`). Intent: refreshed UI evidence, uncommitted. Owner decides commit or discard as one batch.
- Desktop: PR #28 five-platform matrix (status says running, unverified). Menu and dialog rows BLOCKED on xdotool.
- horizon-connect: clean, unpushed, behind origin desktop E1 fixes. Needs rebase or merge decision before push.
- Known gaps (`docs/known-gaps.md`): capability substitution, parallelism plans, `routeExecutor` (nothing calls it), `BACKLOG`/`BLOCKED` states (no UI), artifact/preview fidelity, worktree isolation, global Board, Evidence/Reviews nav. `subagent.start` fire-and-forget.
- TODO/FIXME in src: 0.

## C. Cognate <-> Workbench <-> Horizon

### C1. Horizon to Workbench (Horizon side read at `C:horizon-connector`, branch feat/github-connector)
1. Operator clicks Connect Horizon. `identity.connectHorizon` is operator-only; `runId` refused; tenant and actor pinned (`connectHorizon.ts:42-43`). OBSERVED.
2. Issues a session for identity `horizon`; writes key and companion json (`:50-58`). Key never reaches browser. OBSERVED.
3. Horizon reads key at call time, never logs it, checks mode and expiry (`horizon-connector/src/adapters/workshop.ts:2-6, 43-130`). OBSERVED.
4. Horizon calls Workbench via `connectTransport` to `createRemoteRuntimeService` (Connect RPC) (`workshop.ts:4`). OBSERVED.
5. Horizon `assign`s work to a workshop agent (`horizon-connector/src/app.ts:86`). OBSERVED.
6. Workbench to Horizon back-link: `#session/<id>` deep link. OBSERVED.
Direction: Horizon is client; Workbench server is the service. Only one credential (bearer, 90 days max, on disk).

### C2. Mobile remote and identity
- On main: `src/remote/*`, `/remote/*` routes, `/m` page.
- `packages/identity` (horizon/shared-identity) claims to be the shared operator definition, but nothing imports it. Workbench keeps its own `src/remote/webauthn.ts` (284 lines) and `cbor.ts` (99 lines). `cbor.ts` is byte-identical to identity's; `webauthn.ts` differs by about 58 lines. Two WebAuthn implementations; the shared one is not wired.

### C3. RealityTrace snapshot
Workbench emits via `ObservationDelivery` to SXR v0.3.0 (`observations.ts:69-74`). Cognate branch changes begin-run to `snapshot:genesis` with fallback (`sxr-cli.ts:111`). Workbench has no test of the snapshot path (INFERRED). Drift risk.

### C4. Duplicated responsibilities
| Duplicate | Risk |
|---|---|
| Two WebAuthn/CBOR stacks (Workbench vs identity) | Security drift; identity unused |
| Two operator-identity definitions (Workbench LOCAL_OPERATOR and ensureIdentity vs identity package) | Needs migration |
| Two cockpits (Horizon UI vs Workbench) | Horizon UI not read; overlap likely (UNVERIFIED) |
| Workbench vs "SEA Forge Workbench" | No reference in Workbench or Cognate docs (grep). Skill `building-sea-forge-workbench` describes the same stack (Tauri, Bun, React, Astryx) (INFERRED) |
| Sessions | Workbench `session:<id>` vs Cognate runs |

### C5. Contract risks
1. Version skew: desktop branch pins Cognate 0.8.0; origin/main uses 0.9.0.
2. Identity scope `@godspeed/identity` vs `@godspeedai/*`.
3. horizon-connect diverged from desktop origin (7 unique commits, missing E1 fixes).
4. Unmerged work: identity (3 behind main), snapshot (1 ahead), horizon-connect (no upstream).
5. Governance off by default; case and governed ops refuse. Fail-closed is good, but "end to end" needs live SEA-Forge.
6. Horizon has no RPC path for `observe()` (in-process only).

## Unknowns
- Whether horizon-connect builds and passes its 6 new tests on a clean host (not run).
- Horizon UI and case views (`C:Horizon`, `C:Horizon-core`, `C:horizon-desk-v2`): not read in depth.
- Full HANDOFF.md tranches after T1.
- Whether Workbench Gauntlet and SWE_SEED hits are live or docs-only.
- Identity package tests on shared-identity branch (not run).
