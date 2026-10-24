# Reference: Persisted Record Schemas

> **Exhaustive field definitions, types, invariants, and JSON examples for all persisted governance record kinds.**  
> Canonical Rust Definitions: `crates/sea-forge-core/src/types.rs` and `crates/sea-forge-ledger/src/types.rs`

---

## 1. `CasePlan` (`plan.json`)

Defines the declarative structure of a workflow episode.
* **Rust Struct:** [`sea_forge_core::types::CasePlan`](../../crates/sea-forge-core/src/types.rs#L37-L48)

```json
{
  "version": "0.1",
  "plan_id": "plan_01",
  "case_id": "case_20260902T190000Z_abcdef",
  "run_id": "run_20260902T190000Z_987654",
  "intent_id": "int_01J7K...",
  "items": [
    {
      "plan_item_id": "item_01",
      "name": "generate_and_validate_sea_model",
      "operations": [
        {
          "kind": "write_file",
          "path": "model.sea",
          "content_hint": "{\"domain\": \"demo\", \"entities\": [{\"name\": \"Sample\"}]}"
        },
        {
          "kind": "execute_command",
          "argv": ["/absolute/path/to/sea-forge", "validate", "model.sea"],
          "cwd": "."
        }
      ],
      "entry_criteria": [],
      "entry_criteria_mode": "any",
      "exit_criteria": [],
      "settlement_criteria": {
        "require_exit_zero": true,
        "required_artifacts": ["model.sea"],
        "stdout_must_contain": "sea-forge: model valid",
        "require_approval": false
      },
      "settlement_criteria_ref": "crit_01J7K...",
      "item_kind": "sandboxed_task",
      "markers": {
        "required": false,
        "repetition": false,
        "manual_activation": false
      },
      "max_instances": 1,
      "depends_on": []
    }
  ],
  "template_ref": null,
  "job_contract_ref": null
}
```

---

## 2. `AuthorityDecision` (`authority.json`)

Records the evaluated verdict and rule binding for an action before execution.
* **Rust Struct:** [`sea_forge_core::types::AuthorityDecision`](../../crates/sea-forge-core/src/types.rs#L428-L466)

```json
{
  "version": "0.2",
  "decision_id": "auth_01",
  "run_id": "run_20260902T190000Z_987654",
  "plan_item_id": "item_01",
  "action_id": "act_fdb7f2",
  "correlation_id": "run_20260902T190000Z_987654",
  "operation": {
    "kind": "write_file",
    "path": "model.sea",
    "content_hint": "sha256:400ae8862c37e7beab47a331b4b380495466672f456da29358f1f00e451dfab5"
  },
  "outcome": "allow",
  "verdict": "allow",
  "normalized_disposition": "allow",
  "matched_rule": "allow-demo-write",
  "reason_codes": ["policy_allow"],
  "reason": "policy_allow",
  "policy_refs": ["local-policy:allow-demo-write"],
  "required_next_steps": [],
  "identity_binding": {
    "identity_id": "identity:operator_local",
    "principal": "operator_local",
    "roles": ["operator"],
    "actor_type": "human",
    "binding_resolution": "local-default",
    "identity_binding_source": "custom-cell",
    "source": "custom-cell",
    "sponsor": null,
    "issued_at": null,
    "expires_at": null,
    "identity_binding_hash": "0822d401e4a47a80924adefe50bbc9917347369c0656f373b6c693cc46c4685e"
  },
  "determinism": {
    "policy_bundle_hash": "sha256:5a4fc6f8d69901f5151b9da0eb8cb92604879ac3fc49572894332b35256d072c",
    "action_request_hash": "sha256:73915c76d5071eef9d25db9055a0285afd10c4c1ac5f48cf81b9fbc4b1e55d42",
    "identity_binding_hash": "sha256:359fd1068be0d2c607d8ae6774ce0da5be406f7c616da94e0650614c0ad1919b"
  },
  "action_request": {
    "schema_version": "cam.v1",
    "action_id": "act_fdb7f2",
    "correlation_id": "run_20260902T190000Z_987654",
    "timestamp_utc": "2026-09-02T19:00:00.648324565+00:00",
    "actor": {
      "actor_id": "operator_local",
      "actor_type": "human",
      "principal": "operator_local"
    },
    "action": {
      "operation": "write_file",
      "parameters": {
        "content_sha256": "400ae8862c37e7beab47a331b4b380495466672f456da29358f1f00e451dfab5",
        "path": "model.sea"
      },
      "resource_id": "model.sea",
      "resource_type": "file",
      "tool_name": "sea-forge-cli"
    },
    "context": {
      "artifacts_root": null,
      "branch": null,
      "channel": "cli",
      "domain_model_ref": null,
      "env_keys": [],
      "environment": "local-slice",
      "plan_item_id": "item_01",
      "repo": null,
      "run_id": "run_20260902T190000Z_987654",
      "source_platform": "cli",
      "workspace_root": "/tmp/custom-cell/runs/run_20260902T190000Z_987654/workspace"
    },
    "evidence": {
      "identity_binding_source": "custom-cell",
      "payload_hash": "sha256:bd4f04d696ed1aa0a2860736eda6cf19b3f883bb689b8e9bd808203d5322db3e6",
      "tool_trace_ref": null
    }
  },
  "audit_record": {
    "engine": "sea-forge-authority",
    "disposition": "allow",
    "subject": "file:model.sea",
    "reason": "policy_allow",
    "evidence_refs": ["evi_0001"],
    "recorded_at": "2026-09-02T19:00:00.648324565+00:00",
    "decision_id": "auth_01",
    "case_id": "case_20260902T190000Z_abcdef",
    "run_id": "run_20260902T190000Z_987654",
    "policy_bundle_hash": "sha256:5a4fc6f8d69901f5151b9da0eb8cb92604879ac3fc49572894332b35256d072c",
    "action_request_hash": "sha256:73915c76d5071eef9d25db9055a0285afd10c4c1ac5f48cf81b9fbc4b1e55d42",
    "identity_binding_hash": "sha256:359fd1068be0d2c607d8ae6774ce0da5be406f7c616da94e0650614c0ad1919b"
  },
  "decided_at": "2026-09-02T19:00:00.648324565+00:00",
  "candidate_verdicts": [
    {
      "engine": "local",
      "disposition": "allow",
      "subject": "model.sea",
      "reason": "policy_allow",
      "evidence_refs": ["local-policy:allow-demo-write"],
      "recorded_at": "2026-09-02T19:00:00.648324565+00:00",
      "boundary_constraints": {},
      "compensating_controls": []
    }
  ],
  "winning_source": "local",
  "precedence_reason": "typed deterministic candidate resolution",
  "sandbox_class_granted": "local",
  "approval_request_id": null,
  "opaque_constraint_id": null,
  "boundary_constraints": {},
  "compensating_controls": []
}
```

---

## 3. `TraceEvent` (`trace.jsonl` / `case-events.jsonl`)

Records discrete lifecycle state transitions in chronological order.
* **Rust Struct:** [`sea_forge_core::types::TraceEvent`](../../crates/sea-forge-core/src/types.rs#L528-L541)

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
* **Rust Struct:** [`sea_forge_core::types::EvidenceRecord`](../../crates/sea-forge-core/src/types.rs#L595-L608)

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
* **Rust Struct:** [`sea_forge_core::types::SettlementEvent`](../../crates/sea-forge-core/src/types.rs#L794-L805)

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
  "criteria_ref": "crit_01J7K..."
}
```

---

## 6. `ApprovalRequest` (`approvals.jsonl`)

The append-only human review record. Latest record per `(case_id, approval_id)` establishes standing.
* **Rust Struct:** [`sea_forge_core::types::ApprovalRequest`](../../crates/sea-forge-core/src/types.rs#L867-L892)

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
* **Rust Struct:** [`sea_forge_ledger::types::LedgerEntry`](../../crates/sea-forge-ledger/src/types.rs#L89-L108)

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
