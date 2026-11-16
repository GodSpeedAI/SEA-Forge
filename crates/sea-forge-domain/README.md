# sea-forge-domain

## Overview
SEA Forge crate for the intent vocabulary and domain planning primitives (`Cargo.toml` description). It classifies intent summaries into a closed set of `IntentPattern` values that `sea-forge-planner` maps to concrete case plans. Depends only on `sea-forge-core`.

## Interface
- Source contract: [./src/README.md] — `IntentPattern` and `interpret` [src/lib.rs:IntentPattern], [src/lib.rs:interpret]
- Workspace manifest: `Cargo.toml` (package `sea-forge-domain`, dependency on `sea-forge-core` 0.3.0)

## Invariants
- The intent vocabulary is deliberately closed: only exact `TEST_ONLY: ...` sentinels and the "generate a .sea model" demo phrasing are recognized; anything else is `ForgeError::UnknownIntent` [src/lib.rs:interpret]
- Intent summaries are bounded to non-empty and at most 500 characters [src/lib.rs:interpret]

## Side Effects
- None. Pure string classification.

## Verification
- Run: `cargo test -p sea-forge-domain` (repo-wide: `just test`)
