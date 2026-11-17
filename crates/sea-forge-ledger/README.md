# sea-forge-ledger

## Overview
SEA Forge crate for the integrity ledger: append-only, hash-chained, MMR-committed, signed (`Cargo.toml` description; spec-full §7.0c). Owns the stream/entry/checkpoint types, domain-separated hashing, the `LedgerStream` append/verify/checkpoint/crash-recovery machinery with a bounded reader, `LedgerManager` global checkpoints with independent witnesses, Ed25519 signing key handling, and the typed exposure registry (SEC-09/10/11). A Windows-only optional dependency on `sea-forge-os` supplies owner-only private-file writes.

## Interface
- Source contract: [./src/README.md] — `LedgerStream`, `LedgerManager`, `LedgerEntry`/`LedgerCheckpoint`/`MerkleProof`/`WitnessReceipt`, `check_redaction`/`SECRET_SENTINELS`, signing (`load_or_create_signing_key`, `load_verifying_key`), `ExposureRegistry` [src/lib.rs:pub use], [src/types.rs:LedgerStream], [src/signing.rs], [src/exposure.rs:ExposureRegistry]
- Workspace manifest: `Cargo.toml` (package `sea-forge-ledger`; depends on `sea-forge-core` 0.3.0, `ed25519-dalek`; `[target.'cfg(windows)'.dependencies]` on `sea-forge-os`)

## Invariants
- Crate is `#![forbid(unsafe_code)]` [src/lib.rs:forbid attribute]
- The ledger is append-only and hash-chained with domain-separated hashes (`sea-forge/ledger-entry/v1`, `sea-forge/payload/v1`, `sea-forge/mmr-root/v1`, `sea-forge/global-root/v1`) under the `jcs-nfc-v1` canonicalization profile (not RFC 8785 JCS) [src/types.rs:entry_hash], [src/types.rs:canonical_json]
- Redaction policy rejects payloads containing plaintext secret sentinels; only explicit `redacted_commitment` payloads pass [src/types.rs:check_redaction]
- Verify operations never mint keys (F-22); signing keys are created 0600 from the start (Unix) or with an owner-only DACL (Windows) [src/signing.rs:load_verifying_key], [src/signing.rs:load_or_create_signing_key]
- Bounded-reader advances use checked arithmetic and validate row linkage ordinals; violations are `ledger_integrity_error` [src/types.rs:checked_ledger_offset_add]
- Exposure labels come only from trusted observations of plaintext delivery, never from the agent, and are never removed; children inherit all ancestor labels and revocation; parent is set once; delegation cycles are refused (SEC-09/10/11) [src/exposure.rs:module doc], [src/exposure.rs:derive]

## Side Effects
- Filesystem: appends JSONL entries, writes checkpoints/MMR/witness receipts, creates ledgers roots, performs crash repair and tail quarantine [src/types.rs:LedgerStream], [src/types.rs:LedgerManager]
- Filesystem: creates signing-key files and directories on first use [src/signing.rs:load_or_create_signing_key]
- CSPRNG and monotonic-clock use for ULID generation [src/types.rs:ulid]

## Verification
- Run: `cargo test -p sea-forge-ledger` (repo-wide: `just test`)
