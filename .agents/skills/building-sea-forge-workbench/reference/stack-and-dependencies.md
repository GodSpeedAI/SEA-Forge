# Stack and dependencies

Locked decisions, exact responsibilities, exclusions, and deferred choices.
Deviating from a locked decision requires a direct, documented repository
conflict plus an ADR in `docs/decisions/` (ADR-002 governs dependency
additions). "It was easier" is not a conflict.

## Locked stack and responsibility boundaries

### Tauri 2 — desktop shell
Owns: native packaging, OS integration, Unix-socket access, file dialogs,
local draft persistence APIs, request correlation, event-channel bridging,
reconnect behavior, content streaming. The WebView renderer **never** touches
the Unix socket, the filesystem under `.sea-forge/`, or SQL directly.

### Bun — frontend runtime and package manager
Canonical package manager, lockfile owner (`bun.lock`), script runner, dev
runtime, and test runner where compatible. Enforce with `bunfig.toml` and
`"packageManager": "bun@<exact-version>"`. Bun is a **development** tool: the
shipped app is Tauri Rust binary + OS WebView + compiled assets — prove this
with the packaging gate ("Bun absent from shipped bundle"). If an upstream
tool genuinely requires Node: prove the incompatibility, isolate the command,
document it as a temporary tooling exception with a removal path. The repo has
no other JS workspace, so no migration question exists — the Workbench Bun
workspace is self-contained.

### React 19 + TypeScript + Vite — renderer
Vite builds the renderer. No Next.js, no SSR, no second renderer framework.

### Astryx — design substrate (not semantic owner)
`@astryxdesign/core`, `@astryxdesign/theme-neutral`, `@astryxdesign/cli`,
`@stylexjs/stylex` (required peer). **Pin exact versions — Astryx is beta.**
Astryx supplies accessible primitives, interaction components, layout
patterns, theming infrastructure, agent-readable CLI docs. SEA Forge owns:
semantic state vocabulary, domain state machines, navigation, governed
actions, settlement/capability meaning, canonical tokens
(`.agents/specs/frontend/colors_and_type.css`). Styling: Astryx precompiled
CSS + SEA Forge custom theme + CSS Modules for bespoke domain components. No
StyleX authoring for SEA Forge components unless a task proves an advantage.

### TanStack Router — navigation state
Typed routes, params, validated search state, URL-addressable projection and
filter state. Guards G1–G9 from the view-flow spec implement as route guards.

### TanStack Query — source-backed inspection
Owns caching/refresh for inspection views (`readiness.get`,
`case.get_overview`, `case.get_horizon`, `run.get`, `settlement.get`,
`capability.get`, …). **Never** canonical mutation semantics — protected
commands go through the SFWP command client (Tauri command → host → socket),
and views update from source-backed state, not from optimistic writes.

### XState — workflow state
Machines for flows with explicit lifecycle/recovery semantics: preflight and
commit, stale preflight, submission ambiguity, cancellation, retry, event
reconnect, agent permission, settlement declaration, evidence verification,
artifact transitions. Machines are frontend interaction state only — they
never duplicate canonical backend reducers.

### react-hook-form + ajv — forms
Contract flow: canonical Rust types → generated JSON Schema → generated
TypeScript → AJV client hints → **server validation and preflight remain
authoritative**. No handwritten canonical domain schema in Zod. Zod narrowly
permitted for route search state / local presentation prefs when justified.

### Dense data
Astryx Table by default. `@tanstack/react-table` + `@tanstack/react-virtual`
only for: virtualized rows, compound filters, nested rows, pinned columns,
criterion/evidence matrices, capability coverage, large audit/timeline views.

### Thoth interaction
`ThothInteractionPort` with `DirectAgUiAdapter` (canonical, open protocol)
and optional `CopilotKitAdapter` (OSS packages only). Full contract:
`thoth-interaction-contract.md`.

### Testing
Vitest, React Testing Library, Storybook, Playwright, axe-core; Rust
integration tests for Tauri commands/channels/reconnect/recovery/packaging/
permissions. Full contract: `testing-and-settlement.md`.

## Excluded (do not add, do not recommend)

Tailwind; shadcn; a second universal component primitive system; generic
dashboard templates as architecture; Redux; global Zustand (unless a concrete
state problem survives Router+Query+XState+local state — document it first);
Next.js/SSR; GraphQL as primary API; generic REST mutation; Copilot Cloud;
enterprise-only CopilotKit features; proprietary hosted runtimes or component
catalogs; CopilotKit-owned persistence/auth/canonical state.

## Tauri bridge policy

Three mechanisms, chosen by payload semantics:

| Mechanism | Use for |
|---|---|
| **Commands** (typed, request/response) | SFWP inspection queries, protected command submission, `request.get_status` lookup, file selection, local draft ops, bounded content requests |
| **Channels** (ordered streams) | SFWP event streams, log streams, agent output, transcript streams, evidence content, long-running operation progress |
| **Lightweight events** | Low-volume presentation notifications only — never where ordering/replay is authoritative |

The Rust host owns: socket connection, protocol negotiation, request
correlation, request-status recovery, event cursor, event-gap recovery,
stream integrity, local file boundaries, desktop permissions. The renderer
receives a narrow typed bridge. **Prohibited:** a generic
`invoke_backend(method, arbitrary_json)` escape hatch when typed generated
commands are feasible.

## Compatibility risks (watch at proof time)

- Astryx beta churn → pin exact versions; record every upgrade against the
  swizzle log in `design-and-ux-contract.md`.
- React 19 + Astryx peer ranges → verify at the Milestone 2 proof, not later.
- Bun + Vite/Tauri CLI interop → Tauri CLI is a Rust binary (fine); if any
  plugin requires Node, apply the documented-exception procedure above.
- StyleX peer of Astryx → consumed as precompiled CSS; do not let it leak
  into SEA Forge authoring.

## Deferred decisions

Do not decide these without repository evidence and a task-specific spike at
the milestone where they become necessary:

| Decision | Trigger (milestone) | Required evidence | Candidates | Acceptance criteria | Cost of delay | Reversibility |
|---|---|---|---|---|---|---|
| Code editor component | Case authoring needs `.sea` editing (M6) | Spike: bundle size, a11y, tokenizer effort, read-only diff views | CodeMirror 6; Monaco | Keyboard a11y, theme fits tokens, <500KB added, `.sea` highlighting feasible | Low — textarea suffices for drafts | High (isolated component) |
| Graph projection library | Horizon/topology visualization beyond tables (M7+) | Spike: dagre-style layouts, a11y list alternative, virtualization | `@xyflow/react`; custom SVG | List alternative preserved; 200-node graph <16ms frame | Low — tables/boards first | High (view-level) |
| Draft storage | Drafts must survive crash/restart (M6) | Spike: Tauri store plugin vs SQLite; multi-draft, versioned, reversible | Versioned local files via Tauri FS scope; SQLite | Survives kill -9; never touches `.sea-forge/`; exportable | Medium — in-memory drafts lose work | Medium (migration path needed) |
| AHP host integration | A second client (CLI/web) needs shared sessions (post-M14) | Real second-client requirement | AHP adapter behind existing port | No SFWP change; port unchanged | None today | High (adapter) |
| OpenTelemetry renderer instrumentation | Perf/diagnosis need beyond devtools (post-M14) | Concrete diagnosis gap | OTel JS SDK, local collector only | No network export by default; no payload capture | None today | High |
| Advanced charting | A view needs real charts, not tables (M13+) | Concrete chart requirement from a settlement/capability view | Astryx charts if available; visx; custom SVG | Tokens respected; a11y table fallback | None today | High |
