# Subsystem: Integrity Ledger & Cryptography

> **Append-only integrity ledgers, ULID ordering, Merkle Mountain Ranges, and multi-witness assurance.**  
> Governed crate: `sea-forge-ledger`

---

## 1. Purpose

The **Integrity Ledger** subsystem provides cryptographic, non-repudiable audit trails for all governance events in SEA Forge. It guarantees that history is strictly append-only (Invariant **DATA-01**), that records cannot be altered or dropped undetected, and that client systems can prove the inclusion of any decision or trace event using Merkle Mountain Range (MMR) proofs.

---

## 2. Responsibilities

* **Append-Only Ledger Streams (`LedgerStream`):** Manages discrete named journals under `.sea-forge/ledgers/<stream_id>/entries.jsonl`.
* **Monotonic Ordering & ULID Generation:** Assigns Universally Unique Lexicographically Sortable Identifiers (ULIDs) to each committed entry, establishing causal event sequence.
* **Hash-Chained Audit Trails:** Links each entry to the SHA-256 hash of the preceding entry in the stream:
  $$\text{entry\_hash}_n = \text{SHA-256}(\text{canonical\_payload}_n \parallel \text{entry\_hash}_{n-1})$$
* **Merkle Mountain Range (MMR) Proofs:** Builds incremental MMR trees over stream entries, generating compact, verifiable `MerkleProof` structures.
* **Cryptographic Signatures & Witness Quorums:** Signs ledger checkpoints using Ed25519 and collects `WitnessReceipt`s to satisfy `PreActionAssurance` requirements.
* **Materialized View Derivation:** Materializes derived views (`authority/active-policy.json`, `authority/decisions.jsonl`) from committed stream records.

---

## 3. Non-Responsibilities

* **Policy Evaluation:** Does not inspect record contents to determine permissions (owned by `sea-forge-authority`).
* **Relational Querying:** Does not provide SQL search over historical events (owned by `sea-forge-capability` via SQLite FTS).

---

## 4. Position in the System

```mermaid
flowchart TB
    KERNEL["Kernel Pipeline / Server"] -->|commit_typed(kind, refs, payload)| STREAM["LedgerStream"]
    STREAM --> ULID["Generate Monotonic ULID"]
    ULID --> CANON["Compute Canonical Payload Hash (jcs-nfc-v1)"]
    CANON --> CHAIN["Link to Prior Entry SHA-256"]
    CHAIN --> MMR["Append to Merkle Mountain Range"]
    MMR --> FLOCK["File-Locked Append (flock) to entries.jsonl"]
    FLOCK --> FLUSH["sync_data() to Disk"]
    FLUSH --> REF["Return CommittedRecordRef"]
    REF --> VIEW["Materialize Derived View"]
```

---

## 5. Core Abstractions

### `LedgerEntry` (`sea-forge-ledger::types::LedgerEntry`)
The atomic record of truth committed to disk:
```rust
pub struct LedgerEntry {
    pub ledger_id: String,
    pub entry_ulid: String,
    pub append_ordinal: u64,
    pub record_kind: String,
    pub correlation_refs: Vec<String>,
    pub payload_hash: String,
    pub prior_entry_hash: String,
    pub entry_hash: String,
    pub committed_at: String,
    pub payload: serde_json::Value,
    pub parent_entry_ulids: Vec<String>,
}
```

### `CommittedRecordRef` (`sea-forge-ledger::CommittedRecordRef`)
An immutable reference returned immediately after an entry is committed:
* `ledger_id`: Name of the owning ledger stream.
* `entry_ulid`: The monotonic identifier.
* `append_ordinal`: 1-based index in the stream.
* `payload_hash`: SHA-256 hash of canonical `jcs-nfc-v1` payload bytes.
* `entry_hash`: Cryptographic leaf hash.

### `MerkleProof` (`sea-forge-ledger::types::MerkleProof`)
Cryptographic proof that a specific `entry_ulid` exists at a specific ordinal within an MMR root:
* `leaf_ordinal`: Position of the entry in the tree.
* `leaf_hash`: Entry hash.
* `peaks`: Current peak hashes of the Merkle Mountain Range.
* `audit_path`: Sibling hashes needed to reconstruct the peak.

---

## 6. Internal Operation

### File-Level Concurrency & Locking
Ledger streams are accessed concurrently by blocking worker threads across the server and CLI. To prevent race conditions and interleaving:
* Each `LedgerStream::commit_entry()` acquires an exclusive `flock(2)` on `.sea-forge/ledgers/<stream>/stream.lock`.
* The payload and newline are written in a single call to `write_all()`.
* `file.sync_data()` is called before releasing the lock, ensuring durability across sudden power loss or process termination.

### Multi-Witness Pre-Action Assurance
When a high-risk policy rule requires pre-action assurance:
1. The kernel commits `authority_decision` entries to the ledger.
2. `LedgerManager::create_pre_action_assurance()` signs the checkpoint using the local node's Ed25519 private key.
3. The checkpoint is dispatched to configured independent witness daemons.
4. Each witness validates the hash chain, signs a `WitnessReceipt`, and returns it.
5. Execution proceeds only when the minimum required quorum (`min_witnesses`) is reached.

---

## 7. State & Persistence

Each ledger stream lives in a private directory within the cell:
```text
.sea-forge/ledgers/<stream_id>/
├── stream.lock         # flock lockfile for append serialization
├── entries.jsonl       # Append-only journal of LedgerEntry records
├── mmr.bin             # Binary serialization of the Merkle Mountain Range
└── checkpoints.jsonl   # Signed checkpoints and witness receipts
```

---

## 8. Failure Modes & Invariants

* **Invariant DATA-01 (Append-Only Truth):** Corrupted or truncated entries trigger a verification failure during `stream.verify()`. The ledger refuses further appends until repaired.
* **Torn Record Protection:** If a system crashes mid-write, readers scan entries using strict JSON parsing. Any incomplete line causes verification to fail closed.
* **Tamper Detection:** If an entry's payload is modified on disk, its `payload_hash` no longer matches the canonical JSON digest, and its `entry_hash` breaks the hash chain, invalidating all subsequent entries and MMR proofs.

---

## 9. Source Trail

* [`crates/sea-forge-ledger/src/types.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-ledger/src/types.rs) — `LedgerManager`, `LedgerStream`, `LedgerEntry`, MMR algorithms, and Merkle proofs.
* [`crates/sea-forge-ledger/src/signing.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-ledger/src/signing.rs) — Ed25519 cryptographic key generation, signing, and verification.
* [`tests/conformance_ledger.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-server/tests/conformance_lifecycle.rs) — Ledger verification, hash-chain integrity, and MMR proof tests.
