# sea-forge-ledger/src

## Overview
Source for the `sea-forge-ledger` crate: SEA Forge's integrity ledger — append-only, hash-chained, MMR-committed, signed (spec-full §7.0c). `types.rs` holds the entry/checkpoint/MMR types, domain-separated hash functions, the `LedgerStream` append/read/verify/checkpoint machinery, the bounded reader, crash-repair, and `LedgerManager` for multi-stream global checkpoints; `signing.rs` holds Ed25519 key loading and signing; `exposure.rs` holds the typed exposure registry (SEC-09/10/11) folded from the `exposure` ledger stream.

## Interface
- Modules: `types`, `signing`, `exposure`; root re-exports the main types plus `canonical_json`, `global_root`, `SECRET_SENTINELS` [lib.rs:pub mod], [lib.rs:pub use]
- Constants: `RECORD_VERSION = "0.2"`, `CANONICALIZATION = "jcs-nfc-v1"`, `HASH_ALGORITHM = "sha256-v1"` [types.rs:RECORD_VERSION], [types.rs:CANONICALIZATION], [types.rs:HASH_ALGORITHM]
- Hash/canonicalization: `canonical_json` (shared `sea_forge_core::canonical` primitive, SUP-09d), `hash_canonical`, `payload_hash`, `entry_hash`, `stream_root`, `global_root` [types.rs:canonical_json], [types.rs:entry_hash], [types.rs:global_root]
- `ulid()` — 26-character Crockford Base32 ULID from the CSPRNG, monotonic under clock regression [types.rs:ulid]
- Types: `MmrState`, `LedgerEntry`, `LedgerCheckpoint`, `MerkleProof`, `WitnessReceipt`, `GlobalCheckpoint`, `AssuranceLevel`, `CommittedRecordRef`, `PreActionAssurance`, `ViewStatus`, `CompatibilityViewState`, `CrashResidue`, `TailRepair` [types.rs]
- `verify_proof(entry_hash, root, proof) -> bool` [types.rs:verify_proof]
- `check_redaction(payload) -> Result<(), ForgeError>` and `SECRET_SENTINELS` — redaction policy rejecting plaintext secrets, exposed so the transcript redactor in `sea-forge-agent` redacts against the same corpus [types.rs:check_redaction], [types.rs:SECRET_SENTINELS]
- `LedgerStream` — one append-only stream: `open`, `commit_typed` / `commit_typed_once` / `commit_typed_new`, `append`, `verify`, `bounded_reader` / `resume_bounded_reader` (`LedgerReadSession`/`LedgerReadPosition`/`LedgerReadResume`/`LedgerReadRow`/`LedgerReadStep`), `create_checkpoint`, `verify_checkpoints`, `prove_entry`, `read_entries`, witness receipts (`read_witness_receipts`, `append_witness_receipt`, `verify_witness_receipts`), crash recovery (`crash_residue`, `repair_tail`, `quarantine_incomplete_tail`), `materialize_view` / `materialize_aggregate_view` / `rebuild_record_view` [types.rs:LedgerStream]
- `LedgerManager` — manages multiple streams and creates/verifies global checkpoints, including `create_pre_action_assurance` with independent witnesses [types.rs:LedgerManager]
- Signing: `load_or_create_signing_key`, `load_verifying_key`, `read_verifying_key`, `sign_bytes`, `verify_signature`; re-exports `ed25519_dalek::{SigningKey, VerifyingKey}` [signing.rs]
- Exposure: `ExposureRegistry` — `open`, `record`, `derive`, `revoke`, `labels`, `inherits`, `revoked` over the `exposure` ledger stream [exposure.rs:ExposureRegistry]

## Invariants
- Crate is `#![forbid(unsafe_code)]` [lib.rs:forbid attribute]
- The ledger is append-only and hash-chained: `entry_hash` is domain-separated (`sea-forge/ledger-entry/v1`) over the canonical entry excluding the `entry_hash` field itself; payload, MMR root, and global root hashes use their own domains (`sea-forge/payload/v1`, `sea-forge/mmr-root/v1`, `sea-forge/global-root/v1`) [types.rs:entry_hash], [types.rs:payload_hash], [types.rs:stream_root], [types.rs:global_root]
- Canonicalization profile is `jcs-nfc-v1` (not RFC 8785 JCS): object keys sorted, string values NFC-normalized while keys are not (documented asymmetry), arrays retain order [types.rs:canonical_json]
- Redaction policy: payloads containing any `SECRET_SENTINELS` string are rejected; a payload explicitly carrying `redacted_commitment` passes as an approved ciphertext commitment [types.rs:check_redaction]
- Verify operations must never mint keys: `load_verifying_key` returns a typed error when the key file is absent rather than creating a signing key (F-22) [signing.rs:load_verifying_key]
- Private key files are created with mode 0600 from the start on Unix (no world-readable window between `write` and `chmod`, F-22); Windows uses the owner-only DACL from `sea_forge_os::write_private` [signing.rs:load_or_create_signing_key]
- Keys live outside `.sea-forge/` by passing an absolute key directory [signing.rs:load_or_create_signing_key]
- Bounded-reader advances use checked arithmetic; ordinal or byte-offset overflow is a `ledger_integrity_error`, and row linkage validates the next ordinal before accepting it [types.rs:checked_ledger_offset_add], [types.rs:checked_ledger_ordinal_add]
- A `LedgerReadPosition` is a candidate row after integrity validation; it is not by itself an acknowledgement or proof of event validity [types.rs:LedgerReadPosition]
- Exposure labels come only from trusted observations of plaintext delivery (e.g. the MCP broker); they never come from the agent and are never removed; a child context inherits every ancestor's labels and revocation; brokered-credential use alone is not exposure [exposure.rs:module doc]
- A context's parent can be set exactly once; delegation that would form a cycle is refused; ancestor walks stop on a cycle or unknown parent [exposure.rs:derive], [exposure.rs:lineage]
- Context/label identifiers are validated: non-empty, ≤128 bytes, ASCII alphanumeric plus `._:-` [exposure.rs:valid]

## Side Effects
- Filesystem: appends JSONL entries, writes checkpoints, MMR state, and witness receipts under the stream root; creates ledgers directories (`LedgerManager::new`); crash repair and tail quarantine mutate the stream files [types.rs:LedgerStream], [types.rs:LedgerManager]
- Filesystem: creates key directories and key files (mode 0600 / Windows owner-only DACL) when a signing key is first created [signing.rs:load_or_create_signing_key]
- Global CSPRNG and monotonic-clock use for ULID generation [types.rs:ulid]

## Verification
- Run: `cargo test -p sea-forge-ledger` (repo-wide: `just test`)
