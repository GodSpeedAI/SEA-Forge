# sea-forge-realitytrace

## Overview
SEA Forge crate providing ledger-derived security evidence for RealityTrace and independent auditors (`Cargo.toml` description; `sea-forge.security-trace/v1`). Per ADR-012, SEA Forge owns authority, enforcement, trusted observation, and settlement in its append-only ledger; RealityTrace owns structured expected-versus-observed comparison; the auditor owns the verdict. This crate is an evidence consumer only — no execution crate depends on it.

## Interface
- Source contract: [./src/README.md] — `TRACE_SCHEMA`, `EXPECTATION_KIND`, `PROOF_PRODUCER`, `FrozenSuite`, `expectation_record`, `project`, `proof_completed`, `record_evidence`, `record_category`, `project_attempts` [src/lib.rs:TRACE_SCHEMA], [src/lib.rs:record_evidence], [src/lib.rs:project_attempts]
- Workspace manifest: `Cargo.toml` (package `sea-forge-realitytrace`; depends on `sea-forge-ledger` 0.3.0 and the pinned `sxr-core` git revision `42879a8`)
- Contract schemas: `contracts/evidence-recorded.v1.schema.json` and `contracts/evidence-recorded.record-kind.psv` (embedded in tests; not a documented subdirectory contract in this assignment)

## Invariants
- Nothing decides, settles, or fills in missing evidence: absent records are reported as `unresolved`, never inferred [src/lib.rs:module doc]
- The E7 proof producer must be `sea-forge-security-acceptance` (the frozen suite), never SEA Forge itself; `sxr_core` refuses any other stamp [src/lib.rs:PROOF_PRODUCER]
- Provenance is fail-closed: 40-hex commit, 64-hex manifest digest, non-empty identity fields [src/lib.rs:expectation_record]
- Evidence categories never merge: claim, decision, enforcement, observation, and settlement stay distinct [src/lib.rs:record_category]
- Duplicate proof delivery and any `sxr_core` refusal are errors, never silently accepted [src/lib.rs:record_evidence]

## Side Effects
- None. Pure projection over committed `LedgerEntry` values and in-memory `sxr_core` ingest/emit; no daemon, no second authority store [src/lib.rs:module doc]

## Verification
- Run: `cargo test -p sea-forge-realitytrace` (repo-wide: `just test`)
