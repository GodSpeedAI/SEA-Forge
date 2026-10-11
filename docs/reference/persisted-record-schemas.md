# Reference: Persisted Record Schemas

> **Exhaustive field definitions, types, invariants, and JSON examples for all persisted governance record kinds.**  
> Canonical Rust Definitions: `crates/sea-forge-core/src/types.rs` and `crates/sea-forge-ledger/src/types.rs`

---

## 1. `CasePlan` (`plan.json`)

Defines the declarative structure of a workflow episode.
* **Rust Struct:** [`sea_forge_core::types::CasePlan`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-core/src/types.rs#L37-L48)

```json
{
  "version": "0.1",
  "plan_id": "plan_20260902T190000Z_123456",
  "case_id": "case_20260902T190000Z_abcdef",
  "run_id": "run_20260902T190000Z_987654",
  "intent_id": "int_01J7K...",
  "items": [
    {
      "plan_item_id": "item_01",
      "name": "generate_and_validate_sea_model",
      "operations": [
        {
          "type": "write_file",
          "path": "model.sea",
          "content_hint": "{\"domain\": \"demo\"}"
        },
        {
          "type": "execute_command",
          "argv": ["sea-forge", "validate", "model.sea"],
          "cwd": null
        }
      ],
      "entry_criteria": [],
      "entry_criteria_mode": "all",
      "exit_criteria": [],
      "settlement_criteria": {
        "require_exit_zero": true,
        "required_artifacts": ["model.sea"],
        "stdout_must_contain": "sea-forge: model valid",
        "agent_output_must_contain": null,
        "require_approval": false,
        "evaluator": null,
        "records": null,
        "per_record_evaluator": null,
        "min_pass_ratio": null
      },
      "item_kind": "sandboxed_task",
      "sandbox_class": "jail",
      "markers": {
        "is_required": true,
        "is_repeating": false,
        "is_discretionary": false
      },
      "max_instances": 1,
      "depends_on": [],
      "environment": null,
      "proposed_by": null
    }
  ],
  "template_ref": null,
  "job_contract_ref": null
}
```

---

## 2. `AuthorityDecision` (`authority.json`)

Records the evaluated verdict and rule binding for an action before execution.
* **Rust Struct:** [`sea_forge_core::types::AuthorityDecision`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-core/src/types.rs#L428-L466)

```json
{
  "version": "0.1",
  "decision_id": "dec_20260902T190000Z_112233",
  "run_id": "run_20260902T190000Z_987654",
  "plan_item_id": "item_01",
  "actor": "operator_local",
  "action": {
    "type": "write_file",
    "path": "model.sea"
  },
  "verdict": "allow",
  "rule_id": "rule_allow_model_sea",
  "policy_bundle_hash": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "action_request_hash": "sha256:7a38f4d...",
  "identity_binding_hash": "sha256:8f2a1b...",
  "reason": "Matched standard developer allow rule",
  "outcome": "allow",
  "sandbox_class": "jail",
  "boundaries": {
    "network": ["ports:[]"]
  },
  "compensating_controls": [],
  "assurance_ref": null,
  "decided_at": "2026-09-02T19:00:00.123456Z"
}
```

---

## 3. `TraceEvent` (`trace.jsonl` / `case-events.jsonl`)

Records discrete lifecycle state transitions in chronological order.
* **Rust Struct:** [`sea_forge_core::types::TraceEvent`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-core/src/types.rs#L528-L541)

```json
{
  "version": "0.1",
  "event_id": "tev_0001",
  "run_id": "run_20260902T190000Z_987654",
  "kind": "plan_created",
  "timestamp": "2026-09-02T19:00:00.050Z",
  "payload": {
    "plan_id": "plan_20260902T190000Z_123456"
  }
}
```
* **Supported `kind` values:** `run_started`, `plan_created`, `authority_evaluated`, `action_granted`, `command_started`, `command_completed`, `artifact_captured`, `settlement_recorded`, `run_completed`, `run_halted`, `item_activated`, `case_closed`.

---

## 4. `EvidenceRecord` (`evidence.jsonl`)

Encapsulates verifiable claims, SHA-256 hashes, and execution results.
* **Rust Struct:** [`sea_forge_core::types::EvidenceRecord`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-core/src/types.rs#L595-L608)

```json
{
  "version": "0.1",
  "evidence_id": "evi_0002",
  "run_id": "run_20260902T190000Z_987654",
  "kind": "artifact",
  "uri": "file://artifacts/model.sea",
  "sha256": "sha256:7b243b74964645229...",
  "captured_at": "2026-09-02T19:00:01.000Z",
  "metadata": {
    "artifact_descriptor": {
      "artifact_id": "art_0001",
      "artifact_type": "sea_model",
      "owner": "operator_local",
      "pre_mint_identity": "ifl:hash:7b243b74964645229..."
    }
  }
}
```

---

## 5. `SettlementEvent` (`settlement.json`)

The independent evaluation of outcome acceptance.
* **Rust Struct:** [`sea_forge_core::types::SettlementEvent`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-core/src/types.rs#L794-L805)

```json
{
  "version": "0.1",
  "settlement_id": "set_20260902T190001Z_aabbcc",
  "run_id": "run_20260902T190000Z_987654",
  "status": "accepted",
  "basis": [
    "authority_allow",
    "exit_zero",
    "required_artifact_present:model.sea",
    "stdout_match"
  ],
  "review_required": false,
  "settled_at": "2026-09-02T19:00:01.050Z",
  "criteria_ref": "item_01"
}
```

---

## 6. `ApprovalRequest` (`approvals.jsonl`)

The append-only human review record. Latest record per `(case_id, approval_id)` establishes standing.
* **Rust Struct:** [`sea_forge_core::types::ApprovalRequest`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-core/src/types.rs#L867-L892)

```json
{
  "version": "0.1",
  "approval_id": "apr_0001",
  "case_id": "case_20260902T190000Z_abcdef",
  "run_id": "run_20260902T190000Z_987654",
  "plan_item_id": "item_02",
  "actor": "operator_local",
  "status": "approved",
  "decision_ref": "dec_20260902T190000Z_998877",
  "requested_at": "2026-09-02T19:00:00Z",
  "expires_at": "2026-09-03T19:00:00Z",
  "resolved_at": "2026-09-02T19:05:00Z",
  "resolved_by": "sec_lead",
  "note": "Verified deployment configuration"
}
```

---

## 7. `LedgerEntry` (`ledgers/<stream>/entries.jsonl`)

The tamper-evident unit of the integrity ledger.
* **Rust Struct:** [`sea_forge_ledger::types::LedgerEntry`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-ledger/src/types.rs#L89-L108)

```json
{
  "ledger_id": "main",
  "entry_ulid": "01J7K9N8PQR...",
  "append_ordinal": 14,
  "record_kind": "settlement_event",
  "correlation_refs": ["run_20260902T190000Z_987654"],
  "payload_hash": "sha256:d41d8cd98f00b204e9800998ecf8427e...",
  "prior_entry_hash": "sha256:e3b0c44298fc1c149afbf4c8996fb924...",
  "entry_hash": "sha256:4b227777d4dd1fc61c6f884f48641d02...",
  "committed_at": "2026-09-02T19:00:01.100Z",
  "payload": { ... },
  "parent_entry_ulids": ["01J7K9N8ABC..."]
}
```
