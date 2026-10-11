# ADR-004: SEA Forge Workbench frontend/desktop stack

## Status

Accepted

## Date

2026-07-24

## Context

`.agents/plans/2026-07-24-sea-forge-workbench-frontend-api-implementation.md`
Task 2 requires a `workbench/` Bun workspace with a Tauri 2 host crate that
builds, launches a window, and renders an Astryx-themed React 19 page using
SEA Forge canonical tokens, TanStack Router, TanStack Query, and XState —
proving the locked stack in `.claude/skills/building-sea-forge-workbench/reference/stack-and-dependencies.md`
actually coexists before any feature work lands on top of it. Per ADR-002,
every new dependency needs an approval entry; this ADR covers the whole
locked-stack set introduced by Task 2 in one entry rather than one per
package, since they were chosen together as a single proven substrate.

Repository evidence before this change: no JS/TS workspace existed anywhere
in the repo (no `package.json`, no `bun.lock`, no `node_modules` outside
`.opencode/` editor tooling). `crates/` is a separate Rust workspace with
`unsafe_code = "deny"` and a synchronous-kernel boundary enforced by
`just no-async-kernel`.

## Decision

### Workspace boundary

`workbench/` is a Bun workspace at the repository root
(`workspaces: ["apps/*", "packages/*"]`, `package.json` `packageManager`
pinned, `bunfig.toml` with `install.exact = true`). It is **not** part of the
root Cargo workspace: `workbench/apps/desktop/src-tauri/Cargo.toml` declares
its own empty `[workspace]` table, making it a standalone Cargo workspace
root rather than an implicit member of the kernel workspace. This keeps
`just no-async-kernel` (which walks a fixed list of kernel crate names) and
`just check`/`just test` unaffected by anything under `workbench/`, and keeps
the Tauri host's `tokio` dependency (required by the `tauri` crate) out of
the kernel's synchronous boundary entirely — it lives in its own workspace,
not merely an excluded member of the shared one.

### Locked packages (exact-pinned, no ranges)

| Package | Version | Role |
|---|---|---|
| `@astryxdesign/core` | 0.1.8 | Component primitives (beta — exact pin required) |
| `@astryxdesign/theme-neutral` | 0.1.8 | Base theme this pack projects SEA Forge tokens onto |
| `@astryxdesign/cli` | 0.1.8 | Agent-readable docs / scaffolding, dev-time only |
| `@stylexjs/stylex` | 0.19.0 | Required Astryx peer; consumed as precompiled CSS only |
| `@tanstack/react-router` | 1.170.18 | Typed routes, validated search state |
| `@tanstack/react-query` | 5.101.4 | Source-backed inspection views, never canonical mutation |
| `xstate` / `@xstate/react` | 5.32.5 / 6.1.0 | Frontend workflow/lifecycle state |
| `react-hook-form` | 7.82.0 | Forms (paired with generated AJV schemas from Task 3 on) |
| `ajv` | 8.20.0 | Client-side hint validation; server preflight remains authoritative |
| `@tauri-apps/cli` | 2.11.4 | Tauri 2 project tooling |
| `vitest` | 4.1.10 | Frontend test runner |

React 19.2.x, Vite 8.1.x, and TypeScript 6.0.x (strict) came from the
`bun create vite --template react-ts` scaffold rather than being separately
pinned; they are already exact/caret-pinned by that template's own
`package.json` and are not part of the locked-substrate set above.

### Astryx theming approach: token projection via `defineTheme`, not raw CSS override

`@astryxdesign/core/theme` exports `defineTheme({ name, extends, tokens })`,
which layers explicit token overrides on top of a base theme
(`@astryxdesign/theme-neutral`'s `neutralTheme`) and applies them as runtime
CSS custom properties via a `data-astryx-theme` attribute. `theme-neutral`'s
precompiled `theme.css` scopes its component rules to
`@scope([data-astryx-theme="neutral"])`, so `workbench/packages/sea-forge-astryx-theme`
keeps `name: "neutral"` (the structural component CSS stays wired) and
overrides only token *values* — `--color-background-body`,
`--color-text-primary`, `--color-accent`, `--color-success/error/warning`,
etc. — to `var(--surface-workspace)`, `var(--fg-primary)`,
`var(--color-focus-primary)`, `var(--color-authority-allowed)`, and so on,
referencing the canonical SEA Forge tokens copy-projected into
`workbench/packages/sea-forge-ui-tokens/sea-forge.tokens.css` from
`.agents/specs/frontend/colors_and_type.css`. This is the "theme" tier of the
swizzle ladder in `reference/design-and-ux-contract.md` — compose over
Astryx, never fork its component internals. A drift-check script
(`packages/sea-forge-ui-tokens/scripts/check-drift.mjs`) byte-compares the
projected CSS against the spec source so the two cannot silently diverge.

### Bun-not-shipped rule

Bun is a **development-only** tool here: package manager, lockfile owner,
script runner, and (via Vitest) test runner. The shipped artifact is the
Tauri Rust binary plus the OS WebView plus compiled Vite assets — Task 14's
packaging gate asserts no `bun` binary appears in the bundle. No exception to
this was needed during Task 2; `@tauri-apps/cli` and all Tauri build steps
run as ordinary Rust/Cargo processes, not Node/Bun-dependent tooling.

### Bun canary (Rust rewrite) as the pinned package manager

`bun upgrade --canary` was run on 2026-07-24 to move the machine's
mise-managed `bun` from stable 1.3.5 to the canary 1.4.0 build, which is
Bun's in-progress Zig→Rust rewrite (confirmed via the upstream `bun-in-rust`
announcement and canary release notes — Claude Code itself has shipped this
build since v2.1.181). `workbench/package.json`'s `packageManager` field is
pinned to `bun@1.4.0` accordingly. This was an explicit, scoped choice: the
canary build is pre-release, so `bun upgrade --stable` is the documented
rollback path if it regresses; `bun install --frozen-lockfile` against the
existing lockfile succeeded under 1.4.0 with no changes, and the full Task 2
gate (install, check, build, test, Tauri `cargo build`) passed under it.

## Rust workspace boundary risk

Because `src-tauri/Cargo.toml` opts out via its own `[workspace]` table, a
future contributor could accidentally delete that table (e.g. while cleaning
up boilerplate) and silently pull the Tauri host — and its `tokio`
dependency — into the root kernel workspace, or make it un-buildable if the
root workspace's `members` list doesn't include it. `just no-async-kernel`
would not catch this (it only enumerates the fixed kernel crate list), so
this is filed as an explicit watch item rather than covered by an existing
gate; see `.agents/OBSERVED_DEBT.md`.

## Alternatives Considered

### Fork Astryx's theme-neutral CSS instead of composing via `defineTheme`

Rejected. `theme-neutral`'s CSS is `@generated ... do not edit manually`
output from `astryx theme build`; hand-editing it would silently diverge on
the next Astryx version bump. `defineTheme`'s token-override mechanism is
the tool's own supported extension point and matches the skill's swizzle
ladder ("compose" before "swizzle-with-record").

### Give the Tauri host crate a real `exclude` entry in the root `Cargo.toml` instead of its own `[workspace]` table

Considered. An `exclude` entry on the kernel's `[workspace]` table would work
equally well and keeps the exclusion visible from the kernel side. Chosen the
crate-local empty `[workspace]` instead so the workbench directory is fully
self-describing (a contributor working only in `workbench/` never needs to
also touch the root `Cargo.toml`), consistent with the plan's instruction to
add "a separate `[workspace]` or exclude entry."

### Stay on stable Bun 1.3.5 instead of the canary Rust rewrite

Considered, and would have been the default absent an explicit request — a
pre-release build carries real regression risk for a project-wide package
manager. Adopted the canary build because it was an explicit, informed
choice made after confirming (via the upstream announcement and this
project's own passing gate under 1.4.0) that it is real, currently-shipping
software, not a hypothetical. `bun upgrade --stable` is the documented
reversal.

## Consequences

- `devbox run -- just check` / `just test` remain scoped to the Rust kernel
  workspace and stay green with zero awareness of `workbench/`.
- `just workbench-check` is the frontend's own gate (`bun install
  --frozen-lockfile && bun run check && bun run build && bun run test`),
  wired in `justfile`, run separately from the kernel gates per this plan's
  commit-hygiene guardrail (Rust and frontend changes land in separate
  commits with their own gates).
- Any future Astryx version bump must be recorded against the swizzle log in
  `reference/design-and-ux-contract.md` per the stack doc's compatibility
  risk note.
- The machine's default `bun` is now a pre-release canary build; if it
  regresses, `bun upgrade --stable` restores 1.3.x and
  `workbench/package.json`'s `packageManager` field must be repinned to
  match in the same change.

## References

- `.agents/plans/2026-07-24-sea-forge-workbench-frontend-api-implementation.md` Task 2
- `.claude/skills/building-sea-forge-workbench/reference/stack-and-dependencies.md`
- `workbench/package.json`, `workbench/bunfig.toml`
- `workbench/apps/desktop/src-tauri/Cargo.toml` (`[workspace]` boundary)
- `workbench/packages/sea-forge-astryx-theme/src/theme.ts`,
  `workbench/packages/sea-forge-ui-tokens/`
- `justfile` `workbench-check` recipe
- `docs/decisions/ADR-002-audit-remediation-dependencies.md` (dependency
  approval convention this ADR follows)
- Bun-in-Rust: `bun.com/blog/bun-in-rust`; canary release
  `github.com/oven-sh/bun/releases/tag/canary`
