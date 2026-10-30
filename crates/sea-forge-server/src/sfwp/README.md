# sea-forge-server — sfwp

## Overview

SFWP (SEA Forge Workbench Protocol) additive transport layer. Per ADR-003, SFWP
is layered *additively* on the existing Unix-socket NDJSON server: new capability
lands as new `verb`-tagged `crate::Request` variants — never as a reshape of the
existing ones, and never as a nested wrapper frame. Old clients omitting the new
verbs keep working; old servers reject an unknown verb cleanly via serde's
unknown-variant error (`sfwp/mod.rs` module doc). The module provides version
negotiation, method/schema description, case and run views, mutations,
approvals, delegations, event streaming, readiness, and precondition guards.

## Interface

- Protocol negotiation and description (`mod.rs`):
  - `SFWP_PROTOCOL_VERSION` (`"1"`; only major version `"1"` is supported) —
    `mod.rs:SFWP_PROTOCOL_VERSION`.
  - `hello(requested_version)` / `describe()` / `get_schema(method)` with
    `HelloResult`, `DescribeResult`, `GetSchemaResult`, `UnsupportedVersion` —
    `mod.rs:hello`, `mod.rs:describe`, `mod.rs:get_schema`.
  - `IMPLEMENTED_METHODS`, `MethodDescriptor`, `InteractionClass`,
    `SCHEMA_TYPES` — `mod.rs:IMPLEMENTED_METHODS`, `mod.rs:SCHEMA_TYPES`.
- Case authoring and views:
  - `case.rs`: `entry_options(root)` — `case.rs:entry_options`;
    `preflight(root, params)` — `case.rs:preflight`;
    `resolve_template_digest_source` — `case.rs:resolve_template_digest_source`;
    types `EntryOptionsResult`, `PreflightResult`, `TemplateOption`,
    `TemplateParameter`, `PlanItemSummary`.
  - `case_views.rs`: `list(root)` — `case_views.rs:list`;
    `get_overview(root, case_id)` — `case_views.rs:get_overview`;
    `get_horizon(root, case_id)` — `case_views.rs:get_horizon`.
  - `case_mutations.rs`: result types `CaseAddItemResult`, `CaseReopenResult`,
    `CaseTerminateResult`, `CaseAdvanceResult`, `HumanTaskCompleteResult`,
    `EpisodeSummary`, `ArtifactView` — `case_mutations.rs`.
- Runs: `run_views.rs` — `list(root, case_id)` (`run_views.rs:list`),
  `get(root, run_id)` (`run_views.rs:get`); types `RunSummary`,
  `RunRecord`, `RunListResult`, `RunViewError`, `TraceRow`, `EvidenceRow`,
  `AuthorityProjection`, `SettlementDetail`, `DeclarationRow`, `CriterionCheck`,
  `RecordPresence`, `RunTermination`.
- Approvals: `approvals.rs` — `list(root, case_id)` (`approvals.rs:list`);
  types `PendingApproval`, `ApprovalListResult`, `ApprovalGovernanceContext`.
- Delegations: `delegations.rs` — `list(root, live)` (`delegations.rs:list`);
  types `DelegationRow`, `DelegationStanding`, `LiveDelegation`.
  `delegation_preview.rs` — `preview(...)` (`delegation_preview.rs:preview`).
- Events: `events.rs` — `open_events_ledger(root)`, `append_event(...)`,
  `replay_after(...)`, `get_range(...)` (`events.rs:append_event`,
  `events.rs:replay_after`); type `EventFrame`.
- Readiness: `readiness.rs` — `get(config, params)` (`readiness.rs:get`);
  types `ReadinessView`, `ReadinessItem`, `OverallReadiness`,
  `ReadinessStatus`, `SourceFreshness`, `RebuildStanding`, `Invalidation`.
- Preconditions (optimistic concurrency): `precondition.rs` —
  `evaluate(resolver, preconditions)` (`precondition.rs:evaluate`),
  `digest_of(value)` (`precondition.rs:digest_of`), trait `RecordResolver`,
  types `Precondition`, `RecordDigest`, `ChangedRecord`, `RejectedAsStale`,
  const `MAX_PRECONDITION_RECORDS` (`precondition.rs:MAX_PRECONDITION_RECORDS`).
- Correlation/idempotency: `correlation.rs` — `payload_hash(payload)`
  (`correlation.rs:payload_hash`), `RequestCorrelationStore`
  (`correlation.rs:RequestCorrelationStore`), `RequestStatus`,
  `RequestRecord`, `DedupeVerdict`.
- Thoth read-only surface: `thoth.rs` (answer views built over
  `sea-forge-thoth` service results).
- Internal codec: `continuation_codec` (private; continuation-key encoding).

## Invariants

- Additive-only evolution: SFWP capability is added as new flat `verb`-tagged
  request variants on `crate::Request`; no wrapper frame, no reshape of existing
  verbs (`sfwp/mod.rs` module doc, ADR-003).
- Only major protocol version `"1"` is supported; an unsupported major version
  is rejected with a structured `unsupported_version` error, never a panic
  (`mod.rs:SFWP_PROTOCOL_VERSION`, `mod.rs:hello`).
- Record-size caps: any single JSON record a view reads whole is capped at
  `MAX_RECORD_BYTES` (4 MiB) and any journal at the wider journal cap, checked
  via `fs::metadata` before reading (SUP-09c). An oversized record/journal folds
  to "unreadable" rather than sizing an allocation from attacker-influenced
  lengths (`mod.rs:MAX_RECORD_BYTES`).
- Precondition evaluation is bounded: at most `MAX_PRECONDITION_RECORDS` (16)
  records per guard (`precondition.rs:MAX_PRECONDITION_RECORDS`).
- Views are read-only projections over the cell directory; only
  `case_mutations` and `events.rs:append_event` mutate case state, and every
  mutation must pass authority/precondition checks at the server layer.

## Side Effects

- Disk reads: views read `case.json`, `plan.json`, `settlement.json`, journals,
  and run records under the cell root, bounded by the record caps above
  (`mod.rs:MAX_RECORD_BYTES`, `run_views.rs`, `case_views.rs`).
- Disk writes (append): `events.rs:append_event` appends frames to the events
  ledger via `LedgerStream` (`events.rs:open_events_ledger`).
- Mutations (`case_mutations.rs`) drive case state transitions that the server
  persists to the cell's ledger and case files.

## Verification

- Run: `cargo test -p sea-forge-server` (repo-wide: `just test`)
