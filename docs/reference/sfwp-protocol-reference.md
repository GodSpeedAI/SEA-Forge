# Reference: SFWP v1 Wire Protocol Specification

> **The normative wire protocol specification for the SEA Forge Wire Protocol (SFWP) Version 1.**  
> Transport: NDJSON over Unix Domain Socket (`server.sock`) · Mode: `0600`

---

## 1. Transport & Framing

SFWP v1 is a line-delimited JSON (NDJSON) protocol operating exclusively over a local Unix domain socket.
* **Socket Path:** Resolved via `docs/CELL_CONTRACT.md` (default: `<root>/server.sock`).
* **Socket Permissions:** Mode `0600` (strictly read/write by socket owner).
* **Framing:** Every message is a single valid JSON object terminated by a single newline character (`\n`).
* **Byte Caps:**
  * `MAX_RECORD_BYTES = 4_194_304` (4 MiB): Maximum size of any single JSON request or response.
  * `MAX_JOURNAL_BYTES = 67_108_864` (64 MiB): Maximum size of streaming event buffers.

---

## 2. Message Envelopes

### Request Envelope
```json
{
  "id": "req_01J7K...",
  "method": "case.commit",
  "params": { ... }
}
```
* `id` (String, required): Caller-generated request ID. For mutations, this ID acts as an **idempotency key**.
* `method` (String, required): Qualified method name from the catalog.
* `params` (Object, optional): Method-specific parameters.

### Success Response Envelope
```json
{
  "id": "req_01J7K...",
  "result": { ... }
}
```

### Error Response Envelope
```json
{
  "id": "req_01J7K...",
  "error": {
    "code": "server_busy",
    "message": "request admission queue capacity exceeded (8 waiters)",
    "data": { ... }
  }
}
```

### Server Event Notification Envelope
```json
{
  "type": "event",
  "event": {
    "event_id": "01J7K...",
    "topic": "case.progress",
    "timestamp": "2026-09-02T19:00:00Z",
    "payload": { ... }
  }
}
```

---

## 3. SFWP v1 Method Catalog

The protocol defines 18 methods across three interaction classes:

### System & Negotiation
| Method | Class | Description |
|---|---|---|
| `system.hello` | `Inspect` | Negotiates protocol version, handshakes server uptime and supported features. |
| `system.describe` | `Inspect` | Returns server descriptor, cell root, active capabilities, and method catalog. |
| `system.get_schema` | `Inspect` | Returns the JSON Schema for any registered contract type. |

### Cell Readiness & Identity
| Method | Class | Description |
|---|---|---|
| `readiness.get` | `Inspect` | Evaluates cell readiness (socket, identity, policy, self-model, sandbox, ledger). |
| `identity.get` | `Inspect` | Returns the active local OS actor identity, bound roles, and permissions. |

### Case Management
| Method | Class | Description |
|---|---|---|
| `case.list` | `Inspect` | Lists all cases in the cell with state and progress metrics. |
| `case.get_overview` | `Inspect` | Returns summary status, timestamps, and active items for a single case, plus `digest` (a `case:` precondition record). |
| `case.get_horizon` | `Inspect` | Returns the complete CMMN sentry dependency horizon and enabled items. |
| `case.preflight` | `Inspect` | Pure dry-run check of a proposed case plan; verifies policy without side effects. |
| `case.commit` | `Command` | Admits, creates, and activates a case plan. Requires `request_id` for idempotency. Accepts `preconditions` (`template:` and `plan:` refs). |
| `case.add_item` | `Command` | Adds one discretionary step to a case's plan. Requires `request_id` (except with `dry_run`). Supports `dry_run` and `preconditions`; see section 5. |

### Approvals & Human Tasks
| Method | Class | Description |
|---|---|---|
| `approval.list` | `Inspect` | Lists all pending, approved, or rejected approval requests. Each entry carries `digest` (an `approval:` precondition record). |
| `approval.decide` | `Command` | Commits a human approval or rejection decision to `approvals.jsonl`. |

### Runs & Assets
| Method | Class | Description |
|---|---|---|
| `run.list` | `Inspect` | Lists execution runs across both flat and case-owned layouts. |
| `run.get` | `Inspect` | Retrieves full 6-record episode metadata for a specific run ID. |
| `asset.list` | `Inspect` | Lists captured artifacts, content hashes, and pre-mint identities. |

### Autonomy & Q&A
| Method | Class | Description |
|---|---|---|
| `thoth.ask` | `Command` | Submits a typed knowledge query to the Thoth self-model engine. |

### Streaming Subscriptions
| Method | Class | Description |
|---|---|---|
| `events.subscribe` | `Subscribe` | Opens a live event stream. Supports `from_cursor` for gap recovery. |
| `events.unsubscribe` | `Command` | Closes an active event subscription channel. |

---

## 4. Error Code Taxonomy

| Error Code | HTTP Analog | Meaning & Remediation |
|---|---|---|
| `bad_request` | 400 | Malformed JSON or invalid parameter schema. |
| `unauthorized` | 401 | Local actor identity lacks permission for this command. |
| `not_found` | 404 | Target case ID, run ID, or approval ID does not exist in cell. |
| `server_busy` | 429 / 503 | Admission queue capacity (8 waiters) exceeded. Retry with backoff. |
| `schema_validation_failed`| 422 | Payload failed AJV or JSON Schema validation against canonical type. |
| `sod_violation` | 403 | Separation of Duties violation: proposer attempted to approve own item. |
| `internal_error` | 500 | Unhandled internal server failure; check `server.log`. |

---

## 5. Preconditions, digests, and `case.add_item`

The examples in this section show the wire as the server reads it: one JSON
object per line, flat, with a `verb` field.

### 5.1 Preconditions

A protected request (`case_commit`, `case_add_item`, `approval_decide`) may
carry `preconditions`:

```json
{"preconditions": {"records": [{"ref": "plan:<id>", "expected_digest": "sha256:<hex>"}]}}
```

The server recomputes each record's current digest. If any differs, the call
is refused with no side effect:

```json
{"outcome": "rejected_as_stale", "code": "precondition_failed",
 "changed_records": [{"ref": "...", "expected_digest": "...", "current_digest": "..."}],
 "next_actions": ["..."]}
```

At most 16 records. A ref the verb cannot resolve is stale.

| Ref | Accepted by | Digest of |
|---|---|---|
| `template:<name>@<version>` | `case_commit` | the template file's bytes |
| `plan:<name>@<version>` | `case_commit` | canonical JSON `{template_sha256, params, items}`: the template's bytes, the request's `params`, and the instantiated plan items. Changes if any of the three changes |
| `plan:<case_id>` | `case_add_item` | the case's current `plan.json` with this item appended, before validation (so a changed plan is reported stale even if it would also fail validation) |
| `case:<case_id>` | `approval_decide` | the ordered record kinds and payload hashes of the case ledger |
| `approval:<approval_id>` | `approval_decide` | the latest case-ledger record that names the approval |

### 5.2 Where a client gets each digest

Never compute a digest on the client. Read it. A `case_get_overview` digest describes exactly the state in the same reply (read under the case lock and re-checked; omitted if the case kept changing):

* `plan:<template_ref>`: the `plan_precondition` field of the `case_preflight` reply (same params as the commit).
  `{"plan_precondition": {"ref": "plan:sequential_agents@0.1.0", "expected_digest": "sha256:..."}}`
  Omitted when the plan could not be built.
* `template:<ref>`: the existing `precondition` field of the same reply.
* `plan:<case_id>`: the `plan_precondition` field of a `case_add_item` `dry_run` reply (section 5.3).
* `case:<case_id>`: the `digest` field of the `case_get_overview` reply.
  `{"digest": {"ref": "case:<case_id>", "expected_digest": "sha256:..."}}`
* `approval:<approval_id>`: the `digest` field of the matching entry in the `approval_list` reply.
  `{"digest": {"ref": "approval:<approval_id>", "expected_digest": "sha256:..."}}`

Every one of these is a ready-made `{ref, expected_digest}` record: copy it
into `preconditions.records` unchanged. `digest` is omitted when the record
cannot be resolved. These digests are computed by the same code that checks
them, and they change whenever the record changes (a `case:` digest changes
on every new ledger record for the case, so it pins "nothing happened since I
looked", not just the fields on screen).

### 5.3 `case_add_item`

Adds one discretionary step. The verified actor becomes `proposed_by`; a
client-supplied value is overwritten. The item must pass plan validation (no
cycles, unique ids) and the `discretionary_task_add` authority check.

Request:

```json
{
  "verb": "case_add_item",
  "case_id": "case_...",
  "item": {
    "plan_item_id": "step_9", "name": "Fix the parser",
    "item_kind": "human_task",
    "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
    "markers": {"required": false, "review": false},
    "external_ref": {"kind": "github_issue", "id": "owner/repo#7", "url": "https://github.com/owner/repo/issues/7"}
  },
  "policy": "policy.yaml",
  "request_id": "handoff:<draft id>",
  "dry_run": false,
  "preconditions": {"records": [{"ref": "plan:case_...", "expected_digest": "sha256:..."}]}
}
```

* `request_id`: required for a real call. Same id and same payload replays the first reply and adds nothing. Same id with a different payload is refused with `error_class: "request_id_reused"`.
* `item.external_ref`: optional `{kind, id, url}`; `kind` is `github_issue` or `ticket`. Stored on the plan item and returned unchanged by `case_get_horizon` items. The case keeper never follows it. Omitted from the stored plan when absent, so plans without it keep their hash.
* `item.markers.review`: optional boolean, default false; marks a step that reviews other work. Omitted from the stored plan when false.
* `dry_run: true`: runs the same validation and authority checks and commits nothing: no plan change, no event, no correlation record, no authority record. The returned `item` is the normalized item exactly as it would be stored. It does not consume `request_id` (and needs none). Reply:

```json
{
  "ok": true, "dry_run": true, "outcome": "would_add",
  "case_id": "case_...", "plan_item_id": "step_9", "proposed_by": "operator_local",
  "item": { "...": "the item as it would be stored, proposed_by set" },
  "plan_precondition": {"ref": "plan:case_...", "expected_digest": "sha256:..."}
}
```

* `preconditions`: send the dry run's `plan_precondition` with the same item. If the case's plan changed since the preview, the reply is `rejected_as_stale` and nothing is written. The precondition is checked first, then the same in-memory plan is validated and written, all under the case's lock, so two confirmations of one preview cannot both commit.
* Real reply: `{"ok": true, "case_id": "...", "plan_item_id": "...", "proposed_by": "..."}`.
