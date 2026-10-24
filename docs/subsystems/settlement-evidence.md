# Subsystem: Settlement, Evidence, and Artifact Identity

> **Outcome qualification from evidence, with explicit basis tokens.**
> Governed crates: `sea-forge-settlement`, `sea-forge-evidence`, `sea-forge-trace`
> Interactive companion: [Governed Run Explorer](../explainers/governed-run-explorer.html)

---

## 1. Purpose

Settlement answers a narrower question than "did the command work": did the captured evidence satisfy the criteria that were committed before execution? Process exit status is one input. Required artifacts, stdout content, execution status, batch evaluator results, and authority verdicts all participate.

A settlement record is durable and machine-readable. Every status carries a non-empty basis. An accepted run with no basis is invalid.

---

## 2. What Settlement Evaluates

`settle(claim, workspace, run_dir)` evaluates:

1. **Authority verdicts.** Any `Escalate` produces `Escalated` with `review_required = true`. Any `Deny` produces `Rejected`. Otherwise it records `authority_allow`.
2. **Missing execution.** A write-only item with no execution is accepted on its materialized artifacts and records `write_only`. A non-write item with no execution is rejected.
3. **Execution status.** `SpawnFailed`, `TimedOut`, `SandboxViolation`, and `SuspectedSandboxViolation` reject without evaluating exit status.
4. **Exit code.** Only when `require_exit_zero` is set. Basis is `exit_zero` or `exit_nonzero`.
5. **Required artifacts.** Checked in the run's `workspace/`, not in `artifacts/`. Basis is `required_artifact_present:<path>` or `required_artifact_missing:<path>`.
6. **Stdout.** `stdout_must_contain` is checked against `run_dir + execution.stdout_path`. Basis is `stdout_match` or `stdout_mismatch`.
7. **Evaluator scores.** Recorded as `evaluator_score:<name>=<value>` evidence. They do not override the other checks.
8. **Batch records.** `min_pass_ratio` adds `batch_pass_ratio`, `batch_threshold`, `batch_passed`, and `batch_below_threshold`. Failing records are written to `run_dir/quarantine/<plan_item_id>.jsonl`, never silently dropped.

`SettlementCriteria` also carries `agent_output_must_contain` and `require_approval`. Those are enforced by the agent delegation and server paths before or around settlement, not inside `settle` itself. Read the criteria struct as a shared contract with several evaluators, not as one function's input list.

There is no expected-content-hash criterion. Artifact SHA-256 values are recorded in evidence descriptors; settlement does not compare them to a pre-declared digest.

---

## 3. Criteria

```rust
pub struct SettlementCriteria {
    pub require_exit_zero: bool,
    pub required_artifacts: Vec<String>,
    pub stdout_must_contain: Option<String>,
    pub agent_output_must_contain: Option<String>,
    pub require_approval: bool,
    pub evaluator: Option<String>,
    pub records: Option<String>,
    pub per_record_evaluator: Option<String>,
    pub min_pass_ratio: Option<f64>,
}
```

A criteria record committed to the ledger also carries `criteria_id`, `origin_refs`, `derivation`, `declared_at`, `criteria_sha256`, and `criteria_record_hash`. The `criteria_ref` on the settlement points back to it. If the ref is absent, settlement appends `legacy_unattributed_criteria`.

---

## 4. Settlement Event

```rust
pub struct SettlementEvent {
    pub version: String,
    pub settlement_id: String,
    pub run_id: String,
    pub status: SettlementStatus,
    pub basis: Vec<String>,
    pub review_required: bool,
    pub settled_at: String,
    pub criteria_ref: Option<String>,
}
```

`SettlementStatus` is `Accepted`, `Rejected`, or `Escalated`. The event is committed to the ledger, written to `settlement.json`, and traced as `SettlementRecorded`.

---

## 5. Evidence and Artifact Identity

`sea-forge-evidence` appends `EvidenceRecord` values and captures files. `EvidenceKind` is `Artifact`, `AuthorityDecision`, `ExecutionResult`, or `Recall`.

An `ArtifactDescriptor` carries:

* `artifact_id`: `art_<timestamp>_<hex>`;
* `artifact_type`, `stage`, `producer`, `owner`, `license`, `review_status`;
* `source_refs`;
* `content_sha256`: SHA-256 of the file content;
* `pre_mint_identity`: `ifl:hash:<sha256>`.

These fields describe the artifact. They are evidence, not settlement acceptance rules.

---

## 6. Persistence

```text
.sea-forge/runs/<run_id>/
├── trace.jsonl           # lifecycle, including SettlementRecorded
├── evidence.jsonl        # execution and artifact evidence
├── settlement.json       # SettlementEvent
└── quarantine/<item>.jsonl
```

The ledger receives the `settlement_event` record. The capability envelope is built later and does not retroactively change the settlement.

---

## 7. Invariants and Failure Modes

* **DOM-01.** Exit zero with a missing required artifact is `Rejected` with `required_artifact_missing:<path>`. Basis would include `authority_allow`, `exit_zero`, and the missing-artifact token.
* **DOM-02.** A settled case activation has trace and evidence, including denials and escalations.
* **Basis is never empty.** Acceptance without a factual basis fails conformance.
* **Artifact scope.** Required artifacts are resolved under `workspace/`. Looking in `artifacts/` is a documentation mistake that leads to false rejections.
* **Suspected is not proven.** A stderr heuristic produces `suspected_jail_violation`, not `jail_violation`.
* **Quarantine is explicit.** Batch failures are written down. They are not discarded.

---

## 8. Source Trail

* [`crates/sea-forge-settlement/src/lib.rs`](../../crates/sea-forge-settlement/src/lib.rs) — `settle`, batch evaluation, prompt-schema checks.
* [`crates/sea-forge-settlement/src/declaration.rs`](../../crates/sea-forge-settlement/src/declaration.rs) — declarations and settlement authorities.
* [`crates/sea-forge-evidence/src/lib.rs`](../../crates/sea-forge-evidence/src/lib.rs) — evidence writer, artifact capture, hashing.
* [`crates/sea-forge-trace/src/lib.rs`](../../crates/sea-forge-trace/src/lib.rs) — trace recorder.
* [`crates/sea-forge-core/src/types.rs`](../../crates/sea-forge-core/src/types.rs) — criteria, claim, event, artifact descriptor.
* [`crates/sea-forge-settlement/tests/criteria_provenance.rs`](../../crates/sea-forge-settlement/tests/criteria_provenance.rs) — criteria and batch tests.
