# sea-forge-domain/src

## Overview
Source for the `sea-forge-domain` crate: SEA Forge's intent vocabulary and domain planning primitives. It classifies a natural-language intent summary into one of a small closed set of `IntentPattern` values that the planner maps to concrete case plans. The single module `lib.rs` depends only on `sea-forge-core`'s `ForgeError`.

## Interface
- `IntentPattern` — closed set of recognized intent shapes: `Demo`, `GeneratedZone`, `FalseSuccess`, `Nonzero`, `Timeout` [lib.rs:IntentPattern]
- `interpret(summary: &str) -> Result<IntentPattern, ForgeError>` — maps an intent summary to an `IntentPattern`, or returns `ForgeError::Input` / `ForgeError::UnknownIntent` [lib.rs:interpret]

## Invariants
- An intent summary must be non-empty and at most 500 characters (counted over `char`s after trimming) or `interpret` fails with `ForgeError::Input` [lib.rs:interpret]
- Only the exact `TEST_ONLY: ...` sentinels and the "generate a .sea model" demo phrasing are recognized; anything else is `ForgeError::UnknownIntent` — the vocabulary is deliberately closed, not heuristic [lib.rs:interpret]

## Side Effects
- None. Pure string classification; no filesystem, network, or clock use.

## Verification
- Run: `cargo test -p sea-forge-domain` (repo-wide: `just test`)
