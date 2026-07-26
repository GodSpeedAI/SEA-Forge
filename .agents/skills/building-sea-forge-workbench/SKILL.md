---
name: building-sea-forge-workbench
description: Builds and wires repository-grounded SEA Forge Workbench vertical slices using Tauri, Bun, React, Astryx, SFWP, AG-UI, and governed event synchronization. Use when implementing or reviewing Workbench routes, components, view models, desktop bridges, API queries and commands, Thoth interaction adapters, live events, evidence, settlement, or frontend/backend integration.
user-invocable: true
---

# Building the SEA Forge Workbench

Implement the SEA Forge Workbench: a Tauri 2 desktop application whose React
renderer talks to `sea-forge-server` only through a typed Rust bridge over the
existing Unix-socket NDJSON surface. Every slice preserves the kernel's
governance invariants: authority before side effects, evidence before
settlement, settlement before capability.

## Purpose

Turn the frontend specification package under `.agents/specs/frontend/` into
working, tested product code — incrementally, one vertical slice at a time,
grounded in the actual repository rather than in specification examples.

## Trigger conditions

Use this skill when a task involves any of: Workbench routes or screens,
semantic or domain components, view models, XState machines, the Tauri host or
bridge, SFWP queries/commands/events, generated frontend contracts, Thoth
interaction adapters (AG-UI or CopilotKit), event reconnect/cursor logic,
frontend evidence/settlement/capability views, Bun workspace tooling, or
desktop packaging.

Do not use it for: pure kernel/crate work with no Workbench surface, marketing
sites, or design-token authoring questions (route those to the design-system
skill at `.agents/specs/frontend/SKILL.md`, which this skill treats as a
source document, not a competitor).

## Non-goals

- Re-designing the visual system (owned by `.agents/specs/frontend/DESIGN.md`).
- Replacing SFWP with GraphQL, generic REST, or CopilotKit-owned state.
- Implementing backend capabilities the kernel already owns.
- Weakening any authority, evidence, settlement, or integrity boundary.

## Locked stack

| Layer | Decision |
|---|---|
| Desktop shell | Tauri 2 (owns socket access, correlation, channels, reconnect, dialogs, drafts) |
| Frontend runtime/pkg | Bun (`bun.lock`, `bunfig.toml`, `packageManager: bun@<exact>`); Node never the product runtime |
| Renderer | React 19 + TypeScript + Vite |
| Design substrate | `@astryxdesign/core` + `@astryxdesign/theme-neutral` + `@astryxdesign/cli` (+ `@stylexjs/stylex` peer), precompiled CSS + SEA Forge theme + CSS Modules |
| Routing | `@tanstack/react-router` (typed routes, validated search state) |
| Server-state | `@tanstack/react-query` (inspection views only, never canonical mutation) |
| Workflow state | `xstate` + `@xstate/react` (preflight/commit, reconnect, approval, settlement flows) |
| Forms | `react-hook-form` + `ajv` (+ `@hookform/resolvers`); schemas generated from Rust, never handwritten Zod domain schemas |
| Dense data | Astryx Table by default; `@tanstack/react-table` / `@tanstack/react-virtual` only where virtualization/compound filtering is needed |
| Thoth interaction | `ThothInteractionPort` with `DirectAgUiAdapter` (canonical) and optional OSS-only `CopilotKitAdapter` |
| Testing | Vitest, React Testing Library, Storybook, Playwright, axe-core, plus Rust integration tests for the Tauri host |

Excluded: Tailwind, shadcn, Redux, Next.js/SSR, GraphQL-as-primary, Copilot
Cloud and all proprietary CopilotKit services, global Zustand (unless a
concrete gap survives Router+Query+XState+local state). New dependencies
require an ADR per `docs/decisions/ADR-002-audit-remediation-dependencies.md`.

Deferred decisions (editor, graph lib, draft storage, AHP, OTel, charting)
live in `reference/stack-and-dependencies.md` with their triggers.

## Source hierarchy

When sources disagree, in order:

1. Actual repository invariants and implementation (`crates/`, root `AGENTS.md`, `docs/decisions/`).
2. Canonical kernel specs (`.agents/specs/spec-minimum.md`, `spec-full.md`).
3. `sea-forge-workbench-api-spec-v0.1.md` (target semantics, not proof of implementation).
4. `sea-forge-workbench-frontend-architecture-component-contract-v0.1.md`.
5. View-flow and wireframe specs.
6. UX epic and atomic design breakdown.
7. `DESIGN.md`, `DESIGN-spec-mapping.md`, `MOCKUP-BRIEF.md`.
8. OpenDesign static kit (`ui_kits/`) and generated handoff metadata — visual reference only, never architecture. Known staleness is catalogued in `reference/source-map.md`.

## Task-class routing

| Task smells like | Read first |
|---|---|
| "Where is X in the spec package / which file is authoritative?" | `reference/source-map.md` |
| "Where does this live in the repo / what exists already?" | `reference/repository-integration.md` |
| "Which library / may I add a dependency?" | `reference/stack-and-dependencies.md` |
| "Implement screen/route/feature Y" | `reference/implementation-workflow.md`, then the slice's contracts |
| "Query/command/event/digest/cursor semantics" | `reference/api-and-event-contracts.md` |
| "Component, token, layout, accessibility" | `reference/design-and-ux-contract.md` (+ `.agents/specs/frontend/DESIGN.md`) |
| "Thoth chat/ask/clarification/CopilotKit" | `reference/thoth-interaction-contract.md` |
| "What tests prove this done?" | `reference/testing-and-settlement.md` |

Load only what the task needs. Never load the whole specification package for
a single-slice task.

## Mandatory repository inspection

Before writing code for any slice:

1. Read the nearest `AGENTS.md` (root today; frontend-local one once created).
2. Confirm current substrate against `reference/repository-integration.md`;
   if reality has moved, update that file in the same change.
3. Locate the exact server surface: `crates/sea-forge-server/src/lib.rs`
   (`Request` enum at ~line 396, Unix listener at ~line 523) and the types in
   `crates/sea-forge-core/src/types.rs`.
4. Classify each capability the slice needs: `EXISTS`, `PARTIAL`, `MISSING`,
   `CONFLICTS`, `UNKNOWN`, `NOT REQUIRED` — with `file:line` evidence.
5. Read one nearby pattern plus its tests before adding code in that area.

## Vertical-slice workflow

Follow the seventeen-step loop in `reference/implementation-workflow.md`:
identify the user-visible settlement → inspect → load contracts → map concepts
to real types → classify → pick the smallest coherent slice → extend server
only where genuinely absent → generate typed contracts → Tauri bridge → view
model/XState → route + components → source-backed queries and events →
failure/recovery handling → tests → visual fidelity + a11y → repository checks
→ evidence report.

Definition of done for every slice: the settlement is demonstrable, its tests
fail when the behavior is broken, `devbox run -- just check` and
`devbox run -- just test` stay green, and no invariant in
`reference/implementation-workflow.md` §Invariants is weakened.

## API grounding workflow

The API spec is a target, not an inventory. For every method a slice touches:

```text
target method → existing request/service/type (file:line)
             → existing event/read model
             → verdict: reuse / adapt / merge / add / reject
```

Today's server speaks a `verb`-tagged enum (`submit`, `status`, `approve`,
`reject`, `agent_list`, `agent_probe`, `delegate`, `cancel_delegation`,
`ask`); SFWP envelopes, request IDs, negotiation, and event subscription are
additive work governed by `reference/api-and-event-contracts.md`. Never
invent a speculative endpoint; never weaken preconditions, digests, or
disclosure gates to simplify the UI.

## Design fidelity workflow

Tokens come from `.agents/specs/frontend/colors_and_type.css` (semantic
`--color-authority-*`, `--color-execution-*`, `--color-dialogue-*` families —
never collapsed to generic success/warning/error). Astryx is substrate;
SEA Forge owns semantics. Component tiers, the swizzling ladder
(compose → theme → supported override → swizzle-with-record), and the
accessibility contract are in `reference/design-and-ux-contract.md`.

For any surface with a checked-in mockup, treat visual fidelity as part of the
settlement:

1. Render the reference and implementation at the same viewport.
2. Capture a reference screenshot and an implementation screenshot.
3. Compare computed geometry for shell tracks, major regions, and fixed bars;
   inspect typography, density, colors, borders, responsive transitions, and
   confirm intended rules are active in normal and reduced-motion media.
4. Add a regression check for stable structure/styles instead of relying on a
   one-time visual judgment.
5. Verify zero console errors and run the required axe/keyboard checks.

Copy layout and interaction intent, never prototype truth: live labels, states,
sources, freshness, authority, and evidence still come from validated
contracts. Do not claim fidelity from source inspection alone.

## Thoth adapter workflow

All Thoth UI goes through `ThothInteractionPort`
(`reference/thoth-interaction-contract.md`). `DirectAgUiAdapter` must always
remain viable; `CopilotKitAdapter` is optional, OSS-only, and removable
without touching canonical types, routes, SFWP contracts, or
approval/settlement behavior. Generative UI resolves only to the registered
`ThothRenderable` union. Approvals always round-trip through
`approval.decide` — a CopilotKit callback is never an approval.

## Validation loop

After every change set:

```sh
cargo fmt --all -- --check          # Rust touched?
devbox run -- just check            # workspace gate
devbox run -- just test             # workspace tests
bun run check                       # frontend lint+types (once workspace exists)
bun test / bun run test:unit        # frontend tests (once workspace exists)
python3 .agents/skills/building-sea-forge-workbench/scripts/validate-skill.py  # when editing this skill
```

Never claim a skipped platform test passed; report it skipped with the reason.

## Completion report contract

Every task using this skill ends with: the settlement demonstrated (command +
output), capability classifications used, repository paths touched, tests
added/updated and their results, invariants checked, limitations, and the next
spendable slice. Reports follow `.agents/` memory rules — update
`CURRENT_STATUS.md` at handoff; log out-of-scope findings in
`OBSERVED_DEBT.md`, not in code comments.

## Examples

**Example 1 — "Show live run status in the Case Horizon."**
Route: `implementation-workflow.md` + `api-and-event-contracts.md`. Findings:
run records EXIST (`crates/sea-forge-core/src/types.rs`), event subscription
MISSING (no `subscribe` verb in `Request`). Slice: additive `events.subscribe`
verb + cursor from ledger `entry_ulid`, a Tauri channel, an XState reconnect
machine, and a horizon board consuming the typed stream. Not in slice:
generic pub/sub, HTTP transport.

**Example 2 — "Add an approval decision panel."**
Route: `implementation-workflow.md` + `design-and-ux-contract.md`. Findings:
`Approve`/`Reject` verbs EXIST (`sea-forge-server/src/lib.rs`), approval
listing PARTIAL (status only). Slice: `approval.list`/`approval.get` mapped
onto existing records, `ApprovalDecisionPanel` organism using
`ProtectedActionButton`, stale-precondition rejection surfaced as a repair
path — never optimistic approval.

**Example 3 — "Wire Thoth ask with clarification."**
Route: `thoth-interaction-contract.md`. Findings: `ask` verb and
`sea_forge_thoth::service::ask` EXIST (`crates/sea-forge-thoth/src/service.rs:258`);
AG-UI stream MISSING. Slice: `ThothInteractionPort` + `DirectAgUiAdapter`
over a Tauri channel, rendering only registered `ThothRenderable` components;
denial renders as governed disclosure denial, not as an error toast.

## Reference files

- [source-map.md](reference/source-map.md) — every spec-package file, status, staleness.
- [repository-integration.md](reference/repository-integration.md) — actual substrate with `file:line` evidence.
- [stack-and-dependencies.md](reference/stack-and-dependencies.md) — locked stack, exclusions, deferred decisions.
- [implementation-workflow.md](reference/implementation-workflow.md) — the slice loop, invariants, definition of done.
- [api-and-event-contracts.md](reference/api-and-event-contracts.md) — transport, envelopes, digests, cursors, errors.
- [design-and-ux-contract.md](reference/design-and-ux-contract.md) — tokens, component tiers, a11y, fidelity.
- [thoth-interaction-contract.md](reference/thoth-interaction-contract.md) — port, adapters, prohibitions.
- [testing-and-settlement.md](reference/testing-and-settlement.md) — test layers and completion evidence.

Implementation plan:
`.agents/plans/2026-07-24-sea-forge-workbench-frontend-api-implementation.md`.
