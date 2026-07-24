# API and event contracts

Concise map from the **actual transport** to **target SFWP semantics**. The
full specification is `.agents/specs/frontend/sea-forge-workbench-api-spec-v0.1.md`
and the catalog `sea-forge-workbench-api-method-catalog-v0.1.yaml` — both
self-declared `target-unmapped`. This file tells you how to ground them; it
does not duplicate them.

## Actual transport today

- Unix-domain socket, newline-delimited JSON
  (`crates/sea-forge-server/src/lib.rs:515-535`, 0600 socket perms).
- Request: `#[serde(tag = "verb", rename_all = "snake_case")] enum Request`
  (`lib.rs:396`): `submit`, `status`, `approve`, `reject`, `agent_list`,
  `agent_probe`, `delegate`, `cancel_delegation`, `ask`.
- Response: one ad-hoc JSON line per request. No `request_id`, no
  negotiation, no subscriptions, no structured error frames.
- Unknown verbs fail cleanly (serde unknown-variant), which is the ADR-003
  additive-evolution seam: new SFWP surface lands as new verbs/frames without
  breaking old clients or servers.

## Target SFWP semantics (what additive work builds toward)

- **Frame types:** `hello`, `hello_result`, `request`, `response`, `event`,
  `stream_chunk`, `stream_end`, `ping`, `pong`, `protocol_error`. Transport
  cancellation only abandons a query/stream; it never cancels governed work
  (that is a method like `run.cancel`).
- **Request envelope:** `type`, `protocol: "sea-forge.workbench"`,
  `protocol_version`, `request_id`, `method`, `method_version`,
  `context {cell_id, actor_id, role, source_view}`, `preconditions`, `params`.
- **Interaction classes** (every method declares one): `inspect`, `evaluate`,
  `propose`, `command`, `decide`, `verify`, `subscribe`. Separate queries,
  commands, and events; one user intent per method.
- **Preconditions / expected digests:** `policy_bundle_digest`,
  `plan_digest`, `resource_digest`, `configuration_digest`,
  `self_model_snapshot_digest`, `expected_cursor`, per-record
  `{ref, expected_digest}`. Mismatch → `rejected_as_stale` with
  `code: "precondition_failed"`, the changed records, invalidated checks, and
  repair `next_actions`. Never silently retried.
- **Idempotency & recovery:** every request carries a unique `request_id`;
  protected methods store enough correlation to answer `request.get_status`.
  After transport ambiguity the client MUST recover status before any retry;
  retry is a new episode (`retry ≠ replay`).
- **Events:** `events.subscribe` (opaque durable cursor — ground it on ledger
  `entry_ulid`, `crates/sea-forge-ledger/src/types.rs:206`),
  `events.unsubscribe`, `events.get_range` for deterministic gap recovery.
  Event delivery is not source truth: on gap or doubt, refetch the
  authoritative view.
- **Content access:** bounded, disclosure-gated content requests
  (transcripts, evidence) via streams; disclosure decision happens **before**
  retrieval — no broad fetch + client redaction.
- **Governed outcomes:** responses carry the domain result (allowed / denied
  / escalated / rejected_as_stale / accepted / …) as typed variants, plus
  source, freshness, and integrity metadata and evidence references. No
  generic success envelope; no single status field shared across domains.
- **Errors:** map repository `ForgeError` classes
  (`crates/sea-forge-core/src/errors.rs`) into the SFWP taxonomy: denial,
  escalation, expiry, execution failure, settlement failure, integrity
  failure, transport ambiguity, unsupported version. Unknown enum variants
  render as `unknown`/`unsupported`, mark views stale/blocked, and are never
  mapped to permissive success. No secrets in any frame.

## Grounding procedure (per method, before implementing)

```text
target method (catalog entry, class, authority surface)
  → existing request/service/type      (file:line in crates/)
  → existing event/read model          (ledger records, projections)
  → verdict: reuse / adapt / merge / add / reject
```

Examples of the mapping already known:

| Target method | Existing substrate | Verdict |
|---|---|---|
| `case.submit`-family | `Request::Submit` + `SubmitPayload` (`sea-forge-server/src/lib.rs`) | adapt (add envelope, preconditions) |
| `case.get_overview` / `case.get_horizon` | `Request::Status` + ledger records | add view-shaped queries over existing records |
| `approval.decide` | `Request::Approve`/`Reject` + SoD checks in settlement/core | merge into one decide method with typed outcome |
| `agent_run.*` / delegation | `Request::Delegate`, `CancelDelegation`, `crates/sea-forge-server/src/delegation.rs` | adapt |
| `thoth.ask` | `Request::Ask` → `sea_forge_thoth::service::ask` (`service.rs:258`) | reuse; add streaming presentation via channel |
| `events.subscribe` / `get_range` | ledger `entry_ulid` + append-only streams | add |
| `system.hello` / `describe` / `get_schema` | none | add (negotiation surface) |
| `request.get_status` / `operation.get` | none | add (correlation store) |

Any method that would weaken authority, disclosure-before-retrieval,
settlement, evidence, or ledger invariants: **reject**, and record why in the
task report.

## Prohibitions

GraphQL as primary API; generic REST mutation; generic success responses; one
status field for all domains; direct file access for renderer convenience;
silent provider fallback; inventing methods the catalog does not name without
a spec update; client-side redaction of over-fetched restricted content.
