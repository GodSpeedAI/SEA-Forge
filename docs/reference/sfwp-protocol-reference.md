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
| `case.get_overview` | `Inspect` | Returns summary status, timestamps, and active items for a single case. |
| `case.get_horizon` | `Inspect` | Returns the complete CMMN sentry dependency horizon and enabled items. |
| `case.preflight` | `Inspect` | Pure dry-run check of a proposed case plan; verifies policy without side effects. |
| `case.commit` | `Command` | Admits, creates, and activates a case plan. Requires `request_id` for idempotency. |

### Approvals & Human Tasks
| Method | Class | Description |
|---|---|---|
| `approval.list` | `Inspect` | Lists all pending, approved, or rejected approval requests. |
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
