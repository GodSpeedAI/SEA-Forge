# ADR-005: SFWP schema-generation dependencies (schemars, json-schema-to-typescript)

## Status

Accepted

## Date

2026-07-24

## Context

`.agents/plans/2026-07-24-sea-forge-workbench-frontend-api-implementation.md`
Task 3 requires the SFWP transport-layer contracts to be generated end to end:
Rust types → JSON Schema → TypeScript types + AJV validators, so the frontend
never hand-writes a second copy of a domain shape the server already owns.
Per ADR-002, new dependencies need an approval entry; this ADR covers the two
packages Task 3 introduces for that pipeline.

## Decision

### `schemars` (Rust, `crates/sea-forge-server` only)

Added to `[workspace.dependencies]` as `schemars = "1"` (resolved `1.2.1`),
referenced as `schemars.workspace = true` from `crates/sea-forge-server/Cargo.toml`
only — **not** added to `sea-forge-core` or any other kernel crate. `JsonSchema`
is derived on exactly the eleven new SFWP transport types listed in
`crates/sea-forge-server/src/sfwp/mod.rs`'s `SCHEMA_TYPES` constant
(`EventFrame`, `Precondition`, `RecordDigest`, `ChangedRecord`,
`RejectedAsStale`, `RequestRecord`, `HelloResult`, `DescribeResult`,
`GetSchemaResult`, `UnsupportedVersion`, `MethodDescriptor`) — the protocol
surface Task 3 actually produces, not the full domain model. Domain
view-shaped contracts (readiness, case overview, settlement, …) are Task 5+
territory and will extend this list additively when those types exist; this
ADR does not pre-approve schemars on any crate beyond `sea-forge-server`.

A new binary, `crates/sea-forge-server/src/bin/gen_sfwp_schema.rs`, calls
`schemars::schema_for!()` for each of those types and writes
`<TypeName>.schema.json` into `workbench/packages/contracts/schema/` — the
Rust-authored intermediate the TypeScript generation step consumes.
`crates/sea-forge-server/tests/conformance_sfwp.rs`'s
`generated_schemas_are_committed_and_current` test regenerates into a temp
directory and diffs against the committed files, so a Rust contract type that
changes without regenerating fails the gate.

Adding `schemars` shifted the workspace-unified resolution of the unrelated
transitive dependency `xxhash-rust` (pulled in via `domainforge-core`) from
`0.8.16` to `0.8.18`, both within the crate's existing `^0.8` requirement.
`deny.toml`'s exact-version BSL-1.0 license exception for that crate was
updated to match (`crates/sea-forge-server` and its dependents were otherwise
unaffected); see the `deny.toml` comment for the full note.

### `json-schema-to-typescript` (JS/TS, `workbench/packages/contracts` only)

Added as an exact-pinned dependency (`15.0.4`) of the new Bun workspace
package `@sea-forge/contracts` (`workbench/packages/contracts/package.json`),
alongside `ajv` (`8.20.0`, matching the version already locked for
`apps/desktop` in ADR-004). `workbench/packages/contracts/scripts/generate.ts`
reads every `../schema/*.schema.json`, emits a TypeScript interface per type
via `json-schema-to-typescript`'s `compile()`, and a paired `<Name>.validator.ts`
using Ajv's standalone-code generator. The generator compiles the raw schema
once and writes the resulting static validation function deterministically into
`workbench/packages/contracts/generated/`; it does not call `ajv.compile()` in
the browser. This is required by the packaged Tauri renderer's strict CSP,
which must not allow runtime `Function(...)`/`unsafe-eval`. The generated
validator keeps the same typed `validate` export, so frontend contract
consumers do not change. The pipeline is wired as
`bun run generate:contracts` from the workbench root. No other JS package
gained this dependency; `apps/desktop` consumes the generated output as a
plain workspace package (`@sea-forge/contracts`), not the generator itself.

### Why generation, not hand-written TS domain types

The alternative — hand-writing TypeScript interfaces for the SFWP protocol
surface — was rejected for the same reason ADR-004 rejected forking Astryx's
generated CSS: the Rust types are the single source of truth, and a
hand-maintained TS copy would silently drift the first time a server-side
field changed without a matching frontend PR. Generation plus a committed,
diffed output (rather than generating at build time and never committing) is
what makes drift visible in code review instead of discovered at runtime.

## Consequences

- `just workbench-check` and the plan's Task 3 gate
  (`bun run generate:contracts && git diff --exit-code packages/contracts/generated`)
  both fail loudly the moment a Rust SFWP type changes shape without
  regeneration — this is deliberate, not a flake to silence.
- `cargo deny check licenses` now carries an updated exact-version exception
  for `xxhash-rust 0.8.18`; the next schemars/domainforge-core update that
  shifts this transitive version again will need the same one-line bump.
- Any future SFWP transport type must be added to both `SCHEMA_TYPES` (Rust)
  and picked up automatically by the generator (it globs `schema/*.schema.json`,
  so no TypeScript-side registration is needed beyond re-running the script).
- The generated validators are static JavaScript with a typed export. They may
  use `// @ts-nocheck` for their internal generated implementation; handwritten
  frontend code remains type-checked and consumes `ValidateFunction<T>`.

## References

- `.agents/plans/2026-07-24-sea-forge-workbench-frontend-api-implementation.md` Task 3
- `crates/sea-forge-server/src/sfwp/mod.rs` (`SCHEMA_TYPES`), `src/bin/gen_sfwp_schema.rs`
- `workbench/packages/contracts/` (`package.json`, `scripts/generate.ts`, `schema/`, `generated/`)
- `crates/sea-forge-server/tests/conformance_sfwp.rs` (`generated_schemas_are_committed_and_current`)
- `deny.toml` (`xxhash-rust` exception)
- `docs/decisions/ADR-002-audit-remediation-dependencies.md` (dependency approval convention)
- `docs/decisions/ADR-004-workbench-stack.md` (precedent: generation over hand-maintained duplicates)
