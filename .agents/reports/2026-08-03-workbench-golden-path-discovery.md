# Workbench Golden-Path Discovery

**Date:** 2026-08-03  
**Status:** Active implementation record — not a completion claim

## Product decision

The smallest truthful SEA Forge journey is the representative local Linux path already defined in `docs/execution/PRODUCT_COMPLETION_DEFINITION.md:77-102`.

```text
open or initialize cell
  → inspect readiness and resolved identity
  → ask bounded, disclosure-controlled Thoth question
  → select materialized template and preflight
  → commit one idempotent case
  → execute one governed command or approval path
  → inspect execution, settlement, evidence, and recovery separately
  → reuse only an accepted result with its evidence-backed assurance
```

This is the product's golden path because it crosses governed truth boundaries without claiming catalog breadth: initialization, authority, source records, protected commitment, execution, settlement, evidence, and reuse. The current Workbench is not yet demonstrably complete for that path. The active Ralph plan keeps its required order and defines Task 4 as the first unfinished entry slice (`.agents/plans/2026-08-02-workbench-product-completion-ralph-loop.md:451-496`).

## Intended user and job

The intended user is a local operator who needs to establish a governed cell, understand whether it can safely perform an operation, create and run one bounded piece of work, and inspect the resulting evidence without reading source or calling an API directly. This is inferred from the product completion contract's observable journey, not from the prior static UI.

## Capability map at discovery

| Capability | Standing | Evidence | Product consequence |
| --- | --- | --- | --- |
| Tauri-supervised local sidecar / adopt an existing server | EXISTS, but fresh-root startup was unsafe | `workbench/apps/desktop/src-tauri/src/supervisor.rs:216-358` | A supplied cell can be supervised or adopted, but a missing root previously caused implicit directory creation. |
| Explicit fresh-cell initialization | PARTIAL — implementation in progress | `supervisor.rs:231-295`; renderer action at `ReadinessPage.tsx:98-118` | The first atomic Task 4 slice is making initialization an explicit operator action and refusing a root that gains history while confirmation is open. |
| Cell selection, known-history classification, migration, and incompatible-history refusal | MISSING | No current typed host flow; Task 4 step 1 requires it at plan lines 464-467 | Do not claim open/select or migration complete from environment-variable resolution alone. |
| Identity-safe protected affordances | EXISTS | `workbench/apps/desktop/src/hooks/useIdentity.ts`, `src-tauri/src/bridge.rs:378-404` | The renderer forwards only a server-advertised actor; it does not author a role or identity. |
| Source-backed readiness | EXISTS / PARTIAL | `crates/sea-forge-server/src/sfwp/readiness.rs:197-240`; `ReadinessPage.tsx` | The server projects committed self-model state, but Task 4 still needs full startup/config coverage and real-cell state scenarios. |
| Disclosure-controlled Thoth inspection | EXISTS / PARTIAL | `crates/sea-forge-server/src/sfwp/thoth.rs`; `workbench/apps/desktop/src/pages/ThothPage.tsx:25-190` | A real typed answer can be displayed with omissions and an authority notice; restricted-disclosure E2E evidence remains required. |
| Materialized template entry, authoritative preflight, digest-pinned commit | EXISTS / PARTIAL | `crates/sea-forge-server/src/sfwp/case.rs:1-20, 58-177` | The vertical mechanics exist, but the golden path still lacks a demonstrated real domain/template/environment scenario. |
| Case/run inspect views and separation of execution from settlement | EXISTS | `crates/sea-forge-server/src/sfwp/run_views.rs:1-64` | The product can avoid reporting an exit-zero process as settled success; the execution console and complete lifecycle controls remain later work. |
| Real native Tauri/server proof plant | EXISTS | `.agents/CURRENT_STATUS.md` Task 3 evidence | The integrated path may be tested with `just workbench-e2e-real`; mocked Playwright remains speed evidence only. |

## Feature classification

**Required for the representative path:** explicit cell initialization/open, startup/config/readiness truth, identity resolution, bounded Thoth inspection, one real template preflight/commit, one allow and deny/escalate path, command episode, approval path, run/evidence/settlement inspection, request recovery, and conservative reuse.

**Supportive:** the shell, evidence drawer, event invalidation, generated contracts, package inventory, and the seed/reset demonstration cell. They make the required path observable but are not independent user goals.

**Deferred by owner decision:** the exclusions in `.agents/reports/workbench-completion-eval-exclusions.json`, including remote and multi-user product claims, general agent/ACP behavior, broad federation, and optional CopilotKit. Excluded surfaces must remain visibly unsupported; they may not imitate a live operation.

**Misleading or obsolete if presented as live:** any route backed only by `UnbackedSurface`, static mockup content, a mocked Tauri bridge, a code citation instead of a committed source record, or a generic success status.

## Current Task 4 implementation map

```text
ReadinessPage
  → useIdentity (validated cell/supervision state)
  → sfwp_initialize_cell (closed Tauri host action)
  → CellSupervisor::initialize
  → explicit root recheck
  → supervised or adopted sea-forge-server
  → SFWP readiness / identity / Thoth queries
```

The authority boundary is unchanged: initialization only starts the selected local server after confirmation; protected work still enters the server's authority mediator. The renderer never accesses the filesystem or socket.

## Evidence and current limitations

- `just context-check` and `git diff --check` passed at discovery.
- `just check-fast` passed after host-runtime temp-directory approval.
- `just workbench-contracts-gate` currently fails because pre-existing, uncommitted Task 2 generated contracts differ from `HEAD`; it must not be bypassed by staging only generated files.
- The added fresh-root tests are designed to prove both no write before confirmation and refusal when history appears before the action. Full Task 4 remains unverified until its required real-cell matrix and gate pass.

## Next evidence-bearing work

Finish and verify the explicit fresh-cell initialization slice, then extend the same host-owned entry flow with safe selection, existing-history compatibility, and migration/refusal handling before treating the startup journey as usable.

## 2026-08-03 progress update: explicit fresh-root confirmation

The active slice now keeps a missing or empty root in `initialization_required` until the operator explicitly confirms it. `CellSupervisor::initialize` rechecks that the root is still blank while holding the supervisor lifecycle lock; if history appeared in the meantime, it returns a structured `cell_initialization_refused` envelope with `no_side_effect: true`. The renderer gives that state one dominant Initialize action, hides premature case creation, and retains the action after a refusal while showing the host-provided next lawful action.

**Verified:** the host gate's clippy and all 34 isolated host tests passed, including the supervisor tests that prove no write on missing-root open and refusal without a socket when history appears before confirmation. Its final host-wide formatter check remains blocked by pre-existing `bridge.rs` formatting drift; `just fmt-check`, `just context-check`, and `git diff --check` passed.  
**Not verified:** `just workbench-check` stops in `workbench-contracts-gate` before renderer typecheck, build, or component tests because the pre-existing Task 2 generated contracts differ from `HEAD`. No generated files were staged or altered to hide that failure. The required real-cell selection/history/migration matrix and the Task 4 real E2E gate remain outstanding.

### Boundary review

**Architecture conformance — conformant for this slice.** The Workbench contract places server lifecycle and the Unix socket in the Tauri host (`workbench/apps/desktop/src-tauri/src/lib.rs:67-105`). The new renderer call remains a registered closed host action (`bridge.rs:338-362`), while the state decision and filesystem recheck stay in `supervisor.rs:231-295`; `useIdentity.ts:224-235` only invokes and refreshes. No renderer filesystem or socket path was introduced.

**NeatCode review:** correctness 3/5, repository fit 4/5, semantic integrity 4/5, restraint 4/5, operational credibility 3/5, evidence 3/5. The two 3s are deliberate: renderer verification is blocked at the existing contract gate, and selecting/migrating existing cells is still absent. No new dependency, generated artifact, silent fallback, or duplicate authority path was added. The one named completion finding is the unverified renderer/real-cell portion above; it must be resolved before any Task 4 completion claim.

## 2026-08-03 progress update: history compatibility preflight

The packaged-sidecar proof has been corrected to match the product state
machine: fresh cell open is non-mutating, then an explicit initialize action
starts the real sidecar and serves SFWP. `CellSupervisor` now also reads the
existing, fixed-path self-model manifest header before spawning. A malformed or
unknown `schema_version` is refused as `cell_history_incompatible`, with no
socket created; a compatible or legacy history remains on the existing start /
adopt path. This is a **preflight**, not a migration: the current server does
not wire its append-only self-model `ensure_init` upgrade primitive into startup,
so compatible migration remains incomplete rather than being claimed.

**TDD evidence:** the new incompatible-history test first failed with
`server_start_refused`, proving that the former supervisor attempted the
sidecar. The next host-gate run passed all 34 lib, bridge, and packaged-stack
tests, including the incompatible-history refusal. Its final host-wide format
check remains blocked by pre-existing formatting drift in `bridge.rs`, which is
outside this slice and was not rewritten. `just fmt-check`, `just
context-check`, and `git diff --check` passed. Cargo emitted only the existing
`license`/`license-file` manifest warnings.

## 2026-08-03 progress update: real packaged initialization proof

The native runner now has a dedicated `initialization` scenario. It deliberately
does **not** seed a fixture cell: the configured root is absent when the compiled
Tauri application opens. The WebKit proof requires the rendered `Initialize this
cell` control, asserts that no sidecar socket exists before its click, clicks the
control through the real Tauri bridge, then requires the newly published socket
to serve `system_hello`. This proves the full local edge from visible consent to
the bundled sidecar, rather than only the host unit boundary.

**Verified:** `just workbench-e2e-real initialization` exited 0 on Linux. It
packaged the renderer and native application, launched it under the real WebKit
driver/Xvfb path, and reported `[real-e2e] native Tauri smoke passed`.

**Runner correction:** the Task 4 plan's old `cell|readiness|Thoth` filter did
not select any native-smoke scenario; the runner previously supported only
`hello`, `identity`, `reconnect`, and `request recovery`. That command is not
valid evidence. The dedicated `initialization` name is now the repeatable fresh
cell proof. The generic existing-history smoke remains separate and seed-based.

**Still incomplete:** this verifies initial creation only. It does not select a
different root, migrate compatible history, negotiate a version with the server,
or prove the planned readiness/Thoth history matrix. Those require the owner
authorization for selected-cell persistence and public readiness vocabulary.

## 2026-08-03 discovery update: selected-cell state ownership

The required selection workflow cannot safely be added as a renderer-only root
picker. At present, `CellSupervisor` owns an immutable `Cell`, `SocketHandle`
owns one immutable socket path, and `EventCursor` persists one global cursor.
Selecting another cell therefore needs a host-owned atomic switch of all three
lifecycles, with a cursor namespaced by the selected cell identity. Otherwise
one cell's cursor could be replayed against another cell's events and the
renderer could briefly issue through the old socket after reporting the new
root. Server startup also does not call `sea_forge_self_model::store::ensure_init`,
so the host may refuse unknown schemas but cannot honestly claim migration.

The pending owner decision is recorded in `OPEN_QUESTIONS.md`. No selected-root
file, dialog dependency, schema, public host command, or server lifecycle was
added while that choice remains unapproved.

## 2026-08-03 regression evidence: existing-history packaged path

`just workbench-e2e-real "hello|identity|reconnect|request recovery"` exited
0 after the fresh-root and manifest preflight changes. It packages the current
renderer and native app, uses the seeded existing cell, and drives the real
WebKit/Tauri path through SFWP hello, identity, reconnect, and request-status
recovery. This is regression evidence that established history still starts and
serves the supervised sidecar; it is not evidence of an operator-visible
open/select or migration workflow.
