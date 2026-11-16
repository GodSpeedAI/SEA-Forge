# @sea-forge/ui-tokens (sea-forge-ui-tokens)

## Overview

Bun workspace package (`@sea-forge/ui-tokens`, private, `"type": "module"`) that publishes SEA Forge's canonical design tokens as CSS custom properties. Its sole artifact is `sea-forge.tokens.css` — the dark color-scheme token set (`--surface-*`, `--fg-*`, `--color-authority-*`, `--color-execution-*`, `--color-dialogue-*`, `--color-settlement-*`, `--color-evidence-*`, focus/motion primitives) — which is a **byte-for-byte copy** of the normative spec source `/.agents/specs/frontend/colors_and_type.css`, not a transform of it [package.json:exports, check-drift.mjs:header-comment, sea-forge.tokens.css].

The package exposes the stylesheet via `exports["./tokens.css"] = "./sea-forge.tokens.css"` and a single `check-drift` script [package.json:scripts]. Consumers import it as `@sea-forge/ui-tokens/tokens.css` (see `../sea-forge-ui-components/package.json` dependencies).

## Interface

- `./tokens.css` → `sea-forge.tokens.css`: `:root`-level CSS custom properties defining SEA Forge's color, typography, focus, and reduced-motion primitives [sea-forge.tokens.css].
- [./scripts/README.md] — `check-drift.mjs`, the drift gate enforcing the copy-projection invariant.
- Package script: `check-drift` → `node scripts/check-drift.mjs` [package.json:scripts].

## Invariants

- `sea-forge.tokens.css` must remain a byte-identical copy of `/.agents/specs/frontend/colors_and_type.css`; editing either one without re-copying fails `check-drift` [check-drift.mjs:comparison].
- Token values are changed in the spec source (`.agents/specs/frontend/colors_and_type.css`), then copied here — never hand-edited in place [check-drift.mjs:header-comment].

## Side Effects

- `check-drift.mjs` reads the spec source and the projected file, writes an error to stderr, and exits 1 on drift [check-drift.mjs:main]. The stylesheet itself affects rendering only when loaded by a consumer; the package performs no other I/O.

## Verification

- Run: `bun run check-drift` (from this directory), or the repo gate `just workbench-contracts-gate`, which invokes this script [justfile:check-drift-invocation]. No test script exists for the package.
