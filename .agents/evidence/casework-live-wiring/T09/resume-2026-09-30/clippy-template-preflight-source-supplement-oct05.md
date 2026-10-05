# Clippy template preflight source supplement — 2026-10-05

This supplements, and does not alter, `clippy-template-independent-approval-oct05.md`.
The two preflight excerpts below were extracted from the original command logs
and copied via native patch; each copied excerpt was byte-compared with its
`/tmp` source using `cmp`.

| Gate | Immutable preflight excerpt | Original full raw log | Captured RAM / swap | Compiler process scan |
|---|---|---|---|---|
| Focused Clippy | `clippy-template-clippy-preflight-oct05.raw` | `/tmp/sea-clippy-template-critic-oct05.raw` (mtime 2026-10-05 13:24:25.249746599 -0400) | MemAvailable 2,556,304 kB; SwapTotal 12,582,912 kB; SwapFree 2,328,988 kB | `ps -eo comm= | rg '^(cargo|rustc|rustdoc|clippy-driver|rustfmt)$'` produced no matches |
| Package test | `clippy-template-test-preflight-oct05.raw` | `/tmp/sea-clippy-template-test-critic-oct05.raw` (mtime 2026-10-05 13:27:30.353723358 -0400) | MemAvailable 2,737,552 kB; SwapTotal 12,582,912 kB; SwapFree 2,140,576 kB | Same comm-only scan produced no matches |

The shell did not print a wall-clock timestamp at the instant of preflight.
Approximate gate-start times are 13:23:06 -0400 for Clippy and 13:24:47 -0400
for the package test, inferred from each full-log mtime minus Cargo's reported
build duration (1m19s and 2m43s, respectively). Treat the raw log mtimes and
captured memory/process output as direct evidence; the derived start times are
estimates, not exact timestamps.

## Existing manifest warning anchor

Both Rust commands emitted Cargo's warning that each affected manifest sets
both `license` and `license-file`. The specific warned manifest paths in the
two raw logs are:

`crates/sea-forge-authority/Cargo.toml`, `sea-forge-capability/Cargo.toml`,
`sea-forge-cli/Cargo.toml`, `sea-forge-core/Cargo.toml`,
`sea-forge-domain/Cargo.toml`, `sea-forge-domainforge/Cargo.toml`,
`sea-forge-evidence/Cargo.toml`, `sea-forge-extension/Cargo.toml`,
`sea-forge-ledger/Cargo.toml`, `sea-forge-planner/Cargo.toml`,
`sea-forge-runtime/Cargo.toml`, `sea-forge-sandbox/Cargo.toml`,
`sea-forge-settlement/Cargo.toml`, and `sea-forge-trace/Cargo.toml`.

This is already tracked as `.agents/DEBT.md` M-20 “Release hygiene” (the
recorded statement is that many crates declare both fields and Cargo warns on
every invocation). No new debt claim or edit is needed for this repair. For
example, `crates/sea-forge-core/Cargo.toml:6-7` sets both fields via workspace
values; the other listed paths are the exact package-level Cargo warning
anchors in the captured output.
