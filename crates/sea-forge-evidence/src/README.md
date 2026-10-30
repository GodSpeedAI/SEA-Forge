# sea-forge-evidence/src

## Overview
Source for the `sea-forge-evidence` crate: SEA Forge's evidence and artifact capture. It provides hashing and canonicalization primitives, a strictly sequence-checked JSONL evidence writer, and artifact capture that copies a source file into the run's artifacts directory and records an `EvidenceRecord` (optionally minting an `ArtifactDescriptor` for governed outputs).

## Interface
- `sha256_bytes(bytes: &[u8]) -> String` [lib.rs:sha256_bytes]
- `sha256_file(path: &Path) -> Result<String, ForgeError>` [lib.rs:sha256_file]
- `canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ForgeError>` — delegates to the single shared canonical primitive in `sea-forge-core` (SUP-09d) [lib.rs:canonical_json]
- `hash_canonical<T: Serialize>(value: &T) -> Result<String, ForgeError>` — `sha256:` prefixed digest of the canonical form [lib.rs:hash_canonical]
- `JsonlEvidenceWriter` — JSONL evidence journal with `create`, `append`, `prepare`, `append_prepared`, `refs` [lib.rs:JsonlEvidenceWriter]
- `capture_file(source, artifacts, name, writer, source_event_id, descriptor) -> Result<(EvidenceRecord, Option<ArtifactDescriptor>), ForgeError>` — copy a source file into the artifacts directory and record it as evidence [lib.rs:capture_file]

## Invariants
- `JsonlEvidenceWriter::create` opens with `create_new(true)`: an evidence journal is created exclusively, never truncated or appended to an existing file [lib.rs:create]
- `append_prepared` rejects a prepared record whose `run_id` or `evidence_id` does not match the writer's sequence (`seq_id("evi", 4, sequence + 1)`), keeping the journal gap-free [lib.rs:append_prepared]
- Durability discipline (F-25.r): each append flushes and then calls `sync_data` — flush alone is kill-9-safe, not power-loss-safe; `sync_data` is sufficient for an append-only record [lib.rs:append_prepared]
- Artifact capture reads the source and writes through `sea_forge_sandbox::safe_write_within` (no-follow) so a symlink swapped in at the artifact path cannot redirect the capture (F-25.i); paths are joined via `safe_join` [lib.rs:capture_file]
- Evidence IDs come from `sea-forge-core`'s `seq_id` and the record's `version` from the shared `RECORD_VERSION` [lib.rs:prepare]
- The minted `ArtifactDescriptor` uses fixed identity values (`artifact_type: sea_model`, `stage: intellectual`, `license: internal-unlicensed`, `review_status: draft`) with the content digest [lib.rs:capture_file]

## Side Effects
- Filesystem: creates the artifacts directory, copies artifact bytes into it, creates the evidence JSONL file exclusively, appends records, and `sync_data`s the journal per append [lib.rs:capture_file], [lib.rs:JsonlEvidenceWriter]
- Reads the source file to hash and copy it [lib.rs:capture_file], [lib.rs:sha256_file]

## Verification
- Run: `cargo test -p sea-forge-evidence` (repo-wide: `just test`)
