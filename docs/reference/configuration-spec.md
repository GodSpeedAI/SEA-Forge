# Reference: Configuration & Cell Layout Specification

> **The technical specification for `server.yaml`, environment variable precedence, and on-disk cell layout.**  
> Governing Document: `docs/CELL_CONTRACT.md` · Invariant: `OPS-01`

---

## 1. Environment Variable Precedence

The cell root and communication socket are resolved using strict rank precedence:

| Rank | Environment Variable | Role | Precedence Rule |
|---|---|---|---|
| **1** | `SEA_FORGE_SOCKET` | Absolute path to the Unix domain socket | Wins unconditionally. Socket is used as specified; parent directory is assumed cell root. |
| **2** | `SEA_FORGE_ROOT` | Path to the SeaCell root directory | Socket defaults to `$SEA_FORGE_ROOT/server.sock`. |
| **3** | *(Default fallback)* | `.sea-forge` relative to current directory | Used if neither environment variable is set. |

### Auxiliary Variables
* `SEA_FORGE_CONFIG`: Path to custom `server.yaml` (default: `<cell_root>/server.yaml`).
* `SEA_FORGE_LOG`: Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`). Default is `info`.
* `SEA_FORGE_TIMEOUT`: Default execution timeout in seconds if unspecified in plan (default: `60`).

---

## 2. On-Disk Cell Directory Structure

A fully initialized SEA Forge cell (`SeaCell`) contains the following canonical structure:

```text
<SEA_FORGE_ROOT>/
├── server.sock               # Unix domain socket (mode 0600, owner-only)
├── server.sock.lock          # Exclusive advisory lockfile (flock)
├── server.yaml               # Cell configuration file
├── sea-forge-policy.yaml     # Active policy bundle definition
├── approvals.jsonl           # Append-only journal of human approvals (latest-line-wins)
├── capabilities.jsonl        # Append-only journal of settled SemanticEnvelopes
│
├── ledgers/                  # Cryptographic integrity streams
│   └── <stream_id>/          # E.g. 'main', 'audit'
│       ├── stream.lock       # flock for write serialization
│       ├── entries.jsonl     # Append-only ULID-ordered LedgerEntry journal
│       ├── mmr.bin           # Merkle Mountain Range binary serialization
│       └── checkpoints.jsonl # Signed checkpoints & witness receipts
│
├── memory/                   # Semantic memory & search projections
│   ├── items.jsonl           # Append-only journal of MemoryItems (facts, decisions)
│   └── sqlite.db             # Derived SQLite FTS5 search index (rebuildable)
│
├── authority/                # Rebuildable authority projections
│   ├── active-policy.json    # Mirrored active policy snapshot
│   ├── decisions.jsonl       # Aggregated decision journal
│   └── audit.jsonl           # Rebuildable audit log
│
├── cases/                    # Case management records
│   ├── <case_id>.json        # Case metadata record
│   └── <case_id>/
│       ├── case-events.jsonl # Monotonic dispatch and settlement ordinals
│       └── runs/             # Case-owned execution episodes
│           └── <run_id>/     # Standard 6-record episode layout
│
└── runs/                     # Flat execution episodes (standalone CLI runs)
    └── <run_id>/
        ├── plan.json         # Committed CasePlan
        ├── authority.json    # AuthorityDecision records
        ├── trace.jsonl       # Discrete lifecycle TraceEvents
        ├── evidence.jsonl    # ArtifactDescriptors and ExecutionResult
        ├── settlement.json   # SettlementEvent with basis array
        ├── semantic-envelope.json
        ├── workspace/        # Isolated workload directory (deleted on demand)
        └── artifacts/        # Captured outputs (stdout.txt, stderr.txt, files)
```

---

## 3. `server.yaml` Configuration Schema

Below is the complete annotated reference for `server.yaml`:

```yaml
version: "0.1"
cell_id: "cell_01J7K9ABCDEF123456"

server:
  # Maximum concurrent workload executions in this cell
  max_concurrent_runs: 4

  # Queue capacity for waiting mutation requests (strictly 8 slots)
  admission_wait_queue_capacity: 8

  # Timeout in seconds to wait for an admission permit before failing closed
  admission_timeout_secs: 10

  # Socket permissions (octal string)
  socket_mode: "0600"

policy:
  path: "sea-forge-policy.yaml"
  auto_reload: true

memory:
  fts_enabled: true
  db_path: "memory/sqlite.db"

agents:
  default_timeout_secs: 60
  endpoints:
    - endpoint_ref: "anthropic-claude"
      provider_kind: "anthropic"
      base_url: "https://api.anthropic.com/v1"
      model: "claude-3-5-sonnet-20241022"
      credential_env_var: "ANTHROPIC_API_KEY"
      max_request_bytes: 524288
      max_response_bytes: 1048576
      transcript_retention: "sealed"
```

---

## 4. Configuration Lifecycle & Fail-Closed Preflight (OPS-01)

* **Startup Preflight:** If `server.yaml` is missing, malformed, or contains invalid schemas, `sea-forge-server` halts immediately during startup with exit code 1. It never starts with partial or default fallback assumptions.
* **Atomic Live Reload:** If `server.yaml` is modified while the server is running, the server re-validates the file. If valid, it swaps the configuration via an atomic `Arc` swap (`RwLock<Arc<ServerConfig>>`). The updated configuration governs future dispatches; in-flight tasks retain their original configuration snapshot.
* **Cell Root Immutability:** The cell root cannot be altered at runtime without restarting the server process.
