# sea-forge-core — src

## Overview
Source tree for the synchronous governed capability-execution kernel types and
lifecycle (Cargo.toml). `#![forbid(unsafe_code)]` [lib.rs]. Modules: the
domain type vocabulary ([types.rs]), the approvals journal ([approvals.rs]),
the single canonical-JSON primitive ([canonical.rs]), error taxonomy
([errors.rs]), ID grammar ([ids.rs]), workspace path validation ([path.rs]),
Windows named-pipe naming ([pipe_name.rs]), sandbox workload image identity
([workload_image.rs]), and the strict YAML boundary ([yaml.rs] with
[yaml/README.md]).

## Interface
- Module tree, declared in [lib.rs]: `approvals`, `canonical`, `errors`, `ids`, `path`, `pipe_name`, `types`, `workload_image`, `yaml`.
- Root re-exports ([lib.rs]): `pipe_name_for` [pipe_name.rs], `RECORD_VERSION = "0.1"` [lib.rs:RECORD_VERSION], `ForgeError` [errors.rs:ForgeError].
- Domain types ([types.rs], ~99 public structs/enums): `Intent`, `Case`, `CasePlan`, `PlanItem`, `Operation`, `AuthorityAction`, `Actor`, `ActorRole`, `AuthorityDecision`, `Verdict`, `NormalizedDisposition`, `ExecutionRequest`, `ExecutionResult`, `TraceEvent`, `EvidenceRecord`, `SettlementCriteria(Record)`, `SettlementDeclaration`, `ApprovalRequest`, `CapabilityRecord`, `CapabilityPromotionPolicy`, `SemanticEnvelope`, `MemoryItem`, `ExtensionDescriptor`, `ProjectionRecord`, `SpecPipelineRun`, `BundleManifest`, `TranscriptSummary`, and the full case/evidence/settlement/capability/projection vocabulary.
- Approvals journal ([approvals.rs]): `approvals_path`, `append`, `load_all`, `latest_by_id`, `latest_status`, `pending`, `pending_for_case`, `check_expiry`.
- Canonical JSON ([canonical.rs]): `canonical_json`, `canonical_bytes` — one primitive for every hash producer (SUP-09d).
- IDs ([ids.rs]): `cell_id`, `bundle_id`, `run_id`, `case_id`, `snapshot_id`, `random_id`, `seq_id`, `valid_run_id`.
- Paths ([path.rs]): `validate_relative_path`, `normalize_relative_path`, `path_prefix_matches`, `valid_id_segment`.
- Pipe name ([pipe_name.rs]): `pipe_name_for`.
- Workload image ([workload_image.rs]): `WorkloadImage`, `InvalidWorkloadImage`.
- YAML ([yaml.rs]): `from_str`, `from_slice`, `check_syntax`, `to_string`, `YamlError`, `MAX_INPUT_BYTES = 4 MiB`; strict adapter in [yaml/README.md].

## Invariants
- `#![forbid(unsafe_code)]` [lib.rs:forbid(unsafe_code)].
- `RECORD_VERSION = "0.1"` pins the record schema version re-used across kernel crates [lib.rs:RECORD_VERSION].
- One canonical-JSON primitive for every hash producer; four crates previously carried byte-equivalent copies (SUP-09d) [canonical.rs:module docs].
- Typeless YAML parsing is deliberately not offered — it re-types scalars, so every caller parses into a concrete type [yaml.rs:module docs].
- YAML boundary rejects duplicate keys, expands no merge keys (`<<`), rejects YAML 1.1 boolean words, explicit tags, UTF-8 BOM, non-finite floats, and inputs over 4 MiB [yaml.rs:module docs].
- The approvals journal's format and its "latest record per `approval_id` wins" fold live here so CLI and server cannot disagree about open approvals [approvals.rs:module docs].
- Path/id validation primitives are the single shared defense for the raw-string-path vs normalized-filesystem defect class [path.rs:module docs].

## Side Effects
- Approvals journal reads and appends `<root>/approvals.jsonl` (append-only `OpenOptions`, parent dirs created) [approvals.rs:append].
- ["None"] otherwise — remaining modules are pure: canonical hashing, ID generation, path validation, pipe-name derivation, YAML parsing, and type definitions perform no I/O.

## Verification
- Run: `cargo test -p sea-forge-core` (repo-wide: `just test`)
