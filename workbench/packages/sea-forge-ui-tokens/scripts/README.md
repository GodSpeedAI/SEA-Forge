# sea-forge-ui-tokens/scripts

## Overview

Tooling for the `@sea-forge/ui-tokens` package. The single script, `check-drift.mjs`, enforces that the published token stylesheet `../sea-forge.tokens.css` is an exact byte-for-byte copy of the canonical design-token spec source at `/.agents/specs/frontend/colors_and_type.css` (repo root). It is intentionally a copy-projection, not a transform — the token file must never be hand-edited or divergently transformed [check-drift.mjs:header-comment].

## Interface

- CLI script run via the package script `check-drift` (`node scripts/check-drift.mjs`, per `../package.json`); also invoked directly by the repo gate `just workbench-contracts-gate` [justfile:check-drift-invocation].
- No exported symbols; the script's contract is its exit status.

## Invariants

- `sea-forge.tokens.css` must equal `.agents/specs/frontend/colors_and_type.css` byte-for-byte; any difference is drift and fails the check [check-drift.mjs:comparison].

## Side Effects

- Reads the two files and writes an error line to stderr on drift; exits with status 1 on drift and 0 when the files match [check-drift.mjs:main].
- No file writes, network calls, or other mutations.

## Verification

- Run: `bun run check-drift` (from `workbench/packages/sea-forge-ui-tokens/`), or repo gate `just workbench-contracts-gate`, which also runs this script [justfile:check-drift-invocation].
