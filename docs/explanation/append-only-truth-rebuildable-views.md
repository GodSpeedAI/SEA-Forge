# Explanation: Append-Only Truth vs. Rebuildable Views

> **Deep rationale for Invariant DATA-01: Why governance history is strictly append-only, and why databases and indexes are disposable projections.**

---

## 1. The Core Invariant: DATA-01

SEA Forge enforces a clear hierarchy between **immutable source truth** and **derived query projections**:

> **DATA-01:** Governance history is append-only and cryptographically verifiable. Mutable views, JSON caches, SQLite indexes, query caches, and generated projections are rebuildable derivations and never truth. A projection can never override or contradict an append-only source record.

---

## 2. The Failure of Database-First Governance

Most modern application architectures are **database-first**:
* State is stored in relational tables (PostgreSQL, MySQL, SQLite).
* State transitions mutate rows in place:
  ```sql
  UPDATE runs SET status = 'completed', exit_code = 0 WHERE id = 'run_123';
  ```
* Auditing is added as an afterthought via trigger tables or change-data-capture (CDC) logs.

In a capability-execution governance kernel, database-first persistence introduces fatal vulnerabilities:

1. **Destruction of Causal History:**
   An `UPDATE` or `DELETE` statement permanently destroys past states. If an approver authorized an action based on Policy Version 1, and the policy table is later updated in place to Policy Version 2, forensic reconstruction cannot prove what policy ruled at the instant of execution.
2. **Migration & Schema Fragility:**
   Database migrations (`ALTER TABLE`) can fail halfway, lock entire systems, or subtly alter data types, corrupting historical records across versions.
3. **Write Contention & Concurrency Bottlenecks:**
   ACID database transactions require global row-level or table-level locks. Under high concurrency, competing writers stall or deadlock.

---

## 3. The Append-Only Truth Hierarchy

SEA Forge models all state as an immutable sequence of facts:

```text
       APPEND-ONLY TRUTH (Permanent, Signed, Immutable)
       ├── ledgers/<stream>/entries.jsonl    (ULID-ordered hash chain, MMR)
       ├── cases/<id>/case-events.jsonl      (Monotonic dispatch/settlement ordinals)
       ├── runs/<id>/trace.jsonl             (Discrete lifecycle transitions)
       ├── runs/<id>/evidence.jsonl          (Artifact hashes, execution results)
       ├── approvals.jsonl                   (Latest-line-wins journal)
       └── capabilities.jsonl                (Semantic capability envelopes)
                     │
                     │  Deterministic Replay / Materialization
                     ▼
       REBUILDABLE VIEWS (Disposable, Ephemeral Projections)
       ├── memory/sqlite.db                  (Derived SQLite FTS5 search index)
       ├── authority/active-policy.json      (Latest policy snapshot mirror)
       ├── authority/decisions.jsonl         (Aggregate decision log)
       └── SFWP In-Memory Projections        (case.get_horizon, readiness.get)
```

### Properties of Append-Only Truth
* **Power-Loss Durability:** Appends are written using atomic record-plus-newline buffers and explicitly flushed via `file.sync_data()` before acknowledging success to the caller.
* **Hash-Chained Integrity:** Each ledger entry includes the cryptographic digest of the prior entry. Modifying any historical record invalidates all subsequent entries and Merkle proofs.
* **Non-Destructive Resolutions:** If an approval is updated from `Pending` to `Approved`, the original request is **never edited**. A second record with the identical `approval_id` is appended to `approvals.jsonl`. The current standing is derived by folding the journal (`latest_by_id`).

---

## 4. Rebuildability as a Mathematical Invariant

Because projections are pure mathematical functions of append-only truth:
$$\text{Projection State} = f(\text{Ordered Append-Only Source Entries})$$

Any view can be deleted without data loss.

### Example 1: Rebuilding SQLite Semantic Memory
The SQLite full-text search index (`.sea-forge/memory/sqlite.db`) accelerates natural language queries in `sea-forge recall`.
* If `sqlite.db` is corrupted, deleted, or experiences a SQLite version mismatch:
* Running `sea-forge recall --rebuild` opens `.sea-forge/capabilities.jsonl` and `.sea-forge/memory/items.jsonl`, replays every envelope and memory item in sequence, and reconstructs the SQLite database to **byte-equivalent accuracy**.

### Example 2: Materializing Authority Mirrors
The aggregate decision log (`authority/decisions.jsonl`) and active policy mirror (`authority/active-policy.json`) are maintained by `pipeline::rebuild_authority_mirrors`.
* The function reads all ledger streams from `LedgerManager`.
* It verifies all hash chains.
* It sorts entries by `(committed_at, ledger_id, append_ordinal)`.
* It rewrites the projection files using atomic temporary files (`rename()`), ensuring readers never see a partially written mirror.

---

## 5. Architectural Consequences

* **Uncompromised Auditability:** An external auditor can inspect the raw `.jsonl` files using standard tools (`jq`, `sha256sum`, `cat`) without installing specialized database drivers.
* **Simplified Crash Recovery:** The kernel does not need complex write-ahead log (WAL) replay engines. If a process dies mid-episode, the ledger stream either has the complete line or the line is truncated. Truncated lines are detected on startup preflight and rejected.

---

## 6. Source Evidence

* [`crates/sea-forge-ledger/src/types.rs:182-350`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-ledger/src/types.rs#L182-L350) — `LedgerStream` append and hash chaining.
* [`crates/sea-forge-capability/src/memory.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-capability/src/memory.rs) — SQLite FTS5 rebuild and indexing logic.
* [`crates/sea-forge-cli/src/pipeline.rs:138-212`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-cli/src/pipeline.rs#L138-L212) — `rebuild_authority_mirrors()` implementation.
* [`crates/sea-forge-core/src/approvals.rs:86-97`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-core/src/approvals.rs#L86-L97) — Latest-line-wins fold rule for `approvals.jsonl`.
