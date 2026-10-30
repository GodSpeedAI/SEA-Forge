# sea-forge-core

## Overview
SEA Forge governed capability-execution kernel types and lifecycle
(Cargo.toml). The synchronous foundation crate every other kernel crate
depends on: domain type vocabulary, `ForgeError` taxonomy, ID grammar, ID
prefix constants, workspace path validation, the single canonical-JSON hashing
primitive, the approvals journal, and the strict YAML parsing boundary.
`#![forbid(unsafe_code)]` [src/lib.rs].

## Interface
- Modules: `approvals`, `canonical`, `errors`, `ids`, `path`, `pipe_name`, `types`, `workload_image`, `yaml` [src/lib.rs:pub mod].
- Root re-exports: `pipe_name_for`, `ForgeError`, `RECORD_VERSION = "0.1"` [src/lib.rs].
- Domain types: `Case`, `CasePlan`, `PlanItem`, `Operation`, `AuthorityAction`, `AuthorityDecision`, `TraceEvent`, `EvidenceRecord`, `SettlementDeclaration`, `ApprovalRequest`, `CapabilityRecord`, `SemanticEnvelope`, `MemoryItem`, `ProjectionRecord`, `BundleManifest`, and the full kernel vocabulary [src/types.rs].
- Shared primitives: `canonical_json`/`canonical_bytes` [src/canonical.rs]; `validate_relative_path`, `normalize_relative_path`, `valid_id_segment` [src/path.rs]; `cell_id`/`run_id`/`case_id`/`random_id` [src/ids.rs]; `from_str`/`from_slice`/`check_syntax`/`to_string` [src/yaml.rs].
- Source tree contract: [src/README.md]; strict YAML adapter: [src/yaml/README.md]

## Invariants
- `#![forbid(unsafe_code)]` [src/lib.rs:forbid(unsafe_code)].
- `RECORD_VERSION = "0.1"` is the kernel-wide record schema version [src/lib.rs:RECORD_VERSION].
- One canonical-JSON primitive for every hash producer (SUP-09d) [src/canonical.rs:module docs].
- The approvals journal fold has one owner so CLI and server cannot disagree about open approvals [src/approvals.rs:module docs].
- Strict YAML: no typeless parsing, duplicate keys/merge-key expansion/YAML 1.1 booleans/BOM/non-finite floats rejected, inputs capped at 4 MiB [src/yaml.rs:module docs].

## Side Effects
- Reads and appends `<root>/approvals.jsonl` [src/approvals.rs:append].
- ["None"] otherwise — the crate is otherwise synchronous and side-effect-free by design (kernel crate; no network, no async).

## Verification
- Run: `cargo test -p sea-forge-core` (repo-wide: `just test`)
