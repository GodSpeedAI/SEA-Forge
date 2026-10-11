# Reference: Domain Terminology & Glossary

> **The canonical vocabulary of SEA Forge: precise definitions, domain relationships, and overloaded word disambiguation.**

---

## Master Glossary

### ActionGrant
* **What it is:** A non-cloneable, non-serializable move-only Rust struct (`crates/sea-forge-authority/src/lib.rs`) minted by `PolicyAuthorityEngine` upon an `Allow` decision. It acts as an unforgeable token proving that a specific action was pre-authorized, binding the executable binary hash, workspace boundaries, allowed network ports, and timeout.
* **What it is not:** It is not a serializable capability string, an OAuth token, or an ambient permission flag. It cannot be duplicated or reused across runs.

### Cell (`SeaCell`)
* **What it is:** The fundamental unit of local deployment and governance. Defined by a single root directory (`SEA_FORGE_ROOT`) containing one Unix domain socket (`server.sock`), one lockfile (`server.sock.lock`), and all case, run, ledger, and capability records.
* **What it is not:** It is not a Docker container, Kubernetes pod, or virtual machine.

### Episode
* **What it is:** A single, isolated execution lifecycle instance (`run_<timestamp>_<hex>`) of one plan item or task. Every episode produces the complete suite of six governance records (`plan.json`, `authority.json`, `trace.jsonl`, `evidence.jsonl`, `settlement.json`, `semantic-envelope.json`).
* **What it is not:** It is not an informal log session or an unrecorded command invocation.

### Case & CasePlan
* **What it is:** A CMMN-subset (Case Management Model and Notation) workflow instance (`case_<timestamp>_<hex>`) orchestrating multiple plan items across stages and milestones using event-driven sentry triggers.
* **What it is not:** It is not a BPMN process flow or a static DAG script.

### Intent
* **What it is:** Structured, unverified data expressing an operator's desired transformation (`intent_id`, `summary`, `actor_id`).
* **What it is not:** It is not an executable shell command or script string.

### Pre-Mint Identity
* **What it is:** A content-derived cryptographic identifier formatted as `ifl:hash:<sha256>` assigned to every generated work product at evidence capture time before external attestation or capitalization.
* **What it is not:** It is not a public blockchain transaction ID or an external UUID.

### Sentry
* **What it is:** A reactive transition trigger (`on: {source, event}, if: predicate`) that determines when a case plan item transitions from `Enabled` to `Activated` or `Terminated`.
* **What it is not:** It is not a monitoring alert or error tracking tool (like Sentry.io).

### Settlement
* **What it is:** The independent evaluation of run evidence against pre-committed `SettlementCriteria`, producing a `SettlementEvent` with an explicit `basis` array.
* **Disambiguation (The Two Senses of "Settlement"):**
  * **In SEA Forge:** Settlement means evaluating whether a specific run's *declared acceptance criteria* were satisfied by the evidence it produced (equivalent to SWE_SEED work-contract qualification).
  * **In GodSpeed-Agent:** Settlement refers to "developmental settlement" (classifying long-term autonomous learning progress over repeated cycles).
  * *Note:* SEA Forge does not perform developmental settlement; it performs operational settlement.

### Settlement Basis
* **What it is:** An array of explicit, machine-readable string tokens (e.g. `["authority_allow", "exit_zero", "required_artifact_present:model.sea", "stdout_match"]`) recording the exact factual proof supporting acceptance or rejection.
* **What it is not:** It is not a natural language narration or a subjective log message.

### SFWP (SEA Forge Wire Protocol)
* **What it is:** The versioned (v1) NDJSON protocol operating over local Unix domain sockets between `sea-forge-server` and clients (such as the Tauri desktop host).
* **What it is not:** It is not a remote REST, gRPC, or WebSocket API.

### `jcs-nfc-v1`
* **What it is:** SEA Forge's canonical JSON profile: object keys sorted lexicographically by raw UTF-8 bytes and string values deeply normalized to Unicode Normalization Form C (NFC). Object keys are intentionally not NFC-normalized.
* **What it is not:** It is not RFC 8785 JCS (which uses UTF-16 key sorting and ECMAScript float canonicalization).

---

## Cross-Cutting Concepts Summary

| Term | Category | Defined In | Primary Role |
|---|---|---|---|
| `ActionGrant` | Authority | `sea-forge-authority` | Single-use token enabling sandboxed execution |
| `SettlementClaim` | Settlement | `sea-forge-core` | Complete bundle of criteria, evidence, and exit status |
| `LedgerStream` | Integrity | `sea-forge-ledger` | Append-only ULID-ordered stream with MMR |
| `NetworkPosture` | Sandboxing | `sea-forge-sandbox` | Fail-closed network permission boundary |
| `DomainModelRef` | Semantics | `sea-forge-domainforge` | Hash-pinned certificate of a `.sea` model |
| `ManagerIteration` | Orchestration | `sea-forge-thoth` | Auditable decision cycle of the ADLC manager loop |
| `EventFrame` | Transport | `sea-forge-server` | Live event broadcast envelope over SFWP |
