# ADR-003: Public/persisted contract deltas for the four-spec audit remediation plan

## Status

Accepted

## Date

2026-07-22

## Context

`.agents/plans/2026-07-22-spec-audit-remediation.md` Task 0 requires an
inventory of additive policy/config/public-type changes needed by Tasks 1-18,
and owner approval of the compatibility approach before any task lands code.
The plan's guardrails forbid altering persisted schemas, policy precedence, ID
grammar, exit codes, or public interfaces without explicit approval — this ADR
is that approval, scoped to *additive* deltas only.

Before drafting the inventory, the repository was checked field-by-field
rather than assumed, because several items the plan's per-task steps describe
as needing new fields already exist from prior milestone work (M9-M16 landed
between this plan's audit and today per `git log`):

- `Operation::AgentTask` (`crates/sea-forge-core/src/types.rs:161-171`)
  **already carries** `response_schema: Option<serde_json::Value>` and
  `transcript_retention: Option<String>`. Tasks 15/16's defect is that
  `case_dispatch.rs` currently *drops* these fields on the floor, not that the
  fields are missing. **No new persisted field is needed for these two.**
- `PolicyRule.memory_scope: Option<String>` (`crates/sea-forge-authority/src/lib.rs:738`)
  **already exists**. Task 6's defect is that generic reserved-authority
  matching ignores it, not that it is absent. **No new field needed.**
- `OriginRefKind::DesiredOutcome`, `ItemKind::AgentTask`,
  `ManagerIteration` (with `case_id`/`iteration`/`snapshot_*`/`judgment`
  fields) and the M9 `ProjectionKind` self-model variants are **already
  landed and versioned** (see the in-code compatibility comments at
  `types.rs:659-663` and `:1199-1202`). These are cross-referenced, not
  re-approved, by this ADR.

This matters because it shrinks the genuinely new surface this ADR must
approve, and it establishes the compatibility convention already in
production use, which the remaining genuinely-new deltas below follow rather
than invent.

## Established compatibility convention (already in production, not new)

Confirmed from four live precedents in `sea-forge-core::types`:

1. **New optional struct field**: `#[serde(default)]` +
   `Option<T>` (+ `skip_serializing_if = "Option::is_none"` when the type
   should omit the field entirely on old-shape output). Old readers that
   don't know the field simply never populate it; new readers treat absence
   as "not set." Precedent: `PolicyRule.memory_scope`,
   `Operation::AgentTask.{response_schema,transcript_retention}`.
2. **New enum variant, old readers structurally never see it**: append the
   variant; document (in a doc-comment on the variant) which record_kind or
   context gates its appearance so an old binary never deserializes a record
   that could contain it. Precedent: `ProjectionKind`'s M9 self-model
   variants (gated on `record_kind == self_model_projection` /
   `.sea-forge/self-model/`).
3. **New enum variant, old readers may see it and must fail clean**: append
   the variant and accept that an old reader's `serde` deserialization
   returns `Err` (not a panic, not silent data loss) because the variant only
   appears in records a new-only code path produces. Precedent:
   `OriginRefKind::DesiredOutcome`.
4. **No version bump for additive changes**: `RECORD_VERSION`/schema-tag
   values only change for a genuinely breaking shape change; purely additive
   deltas reuse the existing version and rely on (1)-(3) instead.

Every genuinely new delta below is approved under one of these three shapes,
not a fourth pattern.

## Decision — genuinely new deltas by task

- **Task 5 (DomainForge source-set boundary, M0)**: new finite-limits +
  pinned-version fields on the adapter's source-set contract in
  `crates/sea-forge-domainforge/src/lib.rs` (not `sea-forge-core`, per
  ADR-001's boundary) — shape (1), additive `Option`/defaulted fields on a
  contract that is itself new-ish and not yet widely persisted across old
  records. `source_refs`/model identity gain these as required inputs to
  hashing, per the spec's stable-identity requirement.
- **Task 6 (memory authority scope, M4b)**: no new field (see Context above).
  Behavior-only: `crates/sea-forge-authority/src/lib.rs` must match
  `PolicyRule.memory_scope` against the exact requested target
  entity/process instead of ignoring it, and the canonical `recall_memory`
  action must include target scope so the decision hash changes when scope
  changes. This is an enforcement fix, not a contract delta, and needed no
  owner sign-off under the "no persisted schema change" guardrail — recorded
  here only so Task 6 is not blocked waiting for an ADR entry that isn't
  required.
- **Task 9 (spec-pipeline prerequisites, M5 library)**: shape (1) additive
  fields on the stage/input contract naming immutable predecessor records
  (status, hash, schema, DomainModelRef, pinned DomainForge version) plus
  validation evidence refs. **Owner note (corrected):** `schema_ref`,
  `domain_model_ref`, and `domainforge_version` are owned by
  `sea-forge-core::types::StageFile`, NOT by `sea-forge-spec-pipeline — the
  landed core `StageFile` fields already provide the compatibility path Task
  9 needs, so no future core addition is required for this contract. Task 9
  step 2 keeps a small order helper if the addition can be avoided; this ADR
  pre-approves the addition only for fields not already on core `StageFile`.
- **Task 10A (M5 stage CasePlan, M5 runner)**: shape (1)/(2) — the smallest
  typed stage operation/metadata addition to `sea-forge-core::types`, added
  *only if* existing `Operation`/`ItemKind`/`StageFile` vocabulary cannot
  carry Task 9's predecessor refs (Task 10A step 2 explicitly prefers reuse
  first; the core `StageFile` fields above are the primary reuse target).
- **Task 11 (self-model realization, M9)**: no new record types — the M9
  `ProjectionKind` self-model variants and snapshot/projection shapes already
  exist (see Context). The delta is behavior-only: populate the
  already-defined governance refs (authority/evidence/settlement) instead of
  leaving them empty, per the "Key facts" table.
- **Task 12 (ODI provenance, M10)**: no new persisted type — replaces
  `sha256:placeholder`/`outcome:primary` literal values in
  `crates/sea-forge-planner/src/templates.rs` with real resolved values in
  the *existing* `OriginRefKind::DesiredOutcome`-shaped fields. Behavior-only.
- **Task 13/13B (Thoth claims, M11)**: shape (1) — `authored_by` on
  Thoth-constructed claims (already-current claim record gains an immutable
  actor-identity field, included in canonical claim hashing per Task 13B step
  2). The shared SoD predicate moves to
  `sea_forge_core::types::validate_claim_authorship_sod` (a new *public
  function*, not a persisted-schema change — approved as the "smallest shared
  typed helper" the plan's redesign trigger anticipates).
- **Task 14A/14B (Thoth service + adapters, M11)**: shape (1) — a new `ask`
  `Request` variant on `crates/sea-forge-server/src/lib.rs`'s `Request` enum
  (`enum Request { Submit, Status, Approve, ... }`, confirmed
  `#[serde(rename_all = "snake_case")]` tagged enum at `lib.rs:393-405`).
  Task 14B step 4 explicitly requires "version-skew/protocol tests for
  clients that do not understand the new request variant" — shape (3): an
  old server rejects/`serde` errors on the new tag cleanly rather than
  panicking; a new client talking to an old server gets a typed protocol
  error, never a silent misroute.
- **Task 15 (delegation schema/termination, M13)**: no new field (see
  Context: `response_schema`/`transcript_retention` already exist on
  `Operation::AgentTask`). Behavior-only: carry the existing fields through
  `DelegationRequest`/`case_dispatch.rs` instead of dropping them, and return
  the real committed `SettlementEvent` instead of constructing a synthetic
  `basis: ["delegation_completed"]`.
- **Task 16 (retention/sealed storage, M13)**: shape (1) — retention
  fields/defaults added to `crates/sea-forge-agent/src/config.rs`, server
  config, and the delegation request struct (distinct from the per-task-item
  `Operation::AgentTask.transcript_retention` override, which already
  exists) — these are the *endpoint/global/default* precedence levels the
  spec's resolution order requires, with "explicit version-skew tests" per
  Task 16 step 1. New non-schema artifact: the sealed-transcript key file at
  `.sea-forge/sealed/<run_id>.key` (ADR-002) — a filesystem-path addition,
  not a serialized-record-shape addition, so it needs no version tag, only
  an entry in whatever documentation enumerates `.sea-forge/**` runtime
  paths.
- **Task 17 (endpoint/manager caps, M14/M15)**: shape (1) — a validated
  endpoint parameter/default-resolution contract added to topology templates
  and the manager catalog (replacing the invalid `agent:builtin`/
  `agent:default` placeholder IDs), and binding requested
  `max_manager_iterations` into the canonical manager authority action/
  decision (the existing `ManagerIteration` record shape at
  `types.rs:829-844` is unchanged; the cap becomes an *authority-action*
  input, not a new field on the iteration record itself).
- **Task 18 (SWE_SEED reconciliation, M16)**: no new persisted record type —
  `submit_swe_seed_declaration` and the reconciliation module are new
  *functions/services* over existing `SweSeedTransport`/settlement-
  declaration shapes (per Task 18 step 2's explicit reuse of
  `append_declaration_ledgered_once`). The materialized
  `runs/<run_id>/swe-seed-correlation.json` view is a new *derived,
  rebuildable* file, not a new source-of-truth schema.

## Alternatives Considered

### Bump `RECORD_VERSION` (or an equivalent global schema version) for this plan

Rejected. Every genuinely new delta above is additive under the
already-established compatibility conventions (1)-(3); none removes,
renames, or changes the meaning of an existing field or variant. A version
bump is reserved for breaking changes per the existing convention (item 4
above), and using it here would overstate the compatibility impact and
create unnecessary version-skew surface for adapters that don't touch these
milestones.

### Defer the memory-scope and self-model-governance-ref items as "contract changes" pending this ADR

Rejected. Both are behavior-only fixes to already-existing, already-approved
fields (`PolicyRule.memory_scope`, self-model `ProjectionKind` variants).
Gating them on ADR sign-off would misapply the "no persisted schema change
without approval" guardrail to work that changes no schema at all.

## Consequences

- No task in this plan requires a `RECORD_VERSION` or global schema-tag bump.
- Tasks 5, 9 (conditionally), 10A (conditionally), 13/13B, 14A/14B, and 16
  each add specific new optional fields, one new enum variant
  (`Request::Ask` or equivalently named), or one new public core helper
  (`validate_claim_authorship_sod`); each owning task's commit must state the
  exact final field/variant name it lands (this ADR approves the shape and
  compatibility pattern, not a name freeze) and add or extend a
  `version_skew`-style test per `crates/sea-forge-core/tests/version_skew.rs`
  precedent where the plan's own step calls for one (Tasks 14B, 16).
- Task 6, 11, 12, 15, and 18 require no ADR-gated contract change at all —
  their remediation is behavior/wiring only. Marking them so here prevents
  them being blocked on a nonexistent approval dependency.
- Every new field must be optional/defaulted (never a new required field on
  an existing struct), consistent with the existing convention, so that
  replaying historical ledger records with a new binary never fails
  deserialization.

## References

- `.agents/plans/2026-07-22-spec-audit-remediation.md` Task 0, and the
  per-task steps cited above for Tasks 5, 6, 9, 10A, 11, 12, 13, 13B, 14A,
  14B, 15, 16, 17, 18
- `crates/sea-forge-core/src/types.rs:161-171` (`Operation::AgentTask`
  existing `response_schema`/`transcript_retention`), `:649-664`
  (`OriginRefKind::DesiredOutcome`), `:829-844` (`ManagerIteration`),
  `:1184-1202` (`ProjectionKind` self-model variants)
- `crates/sea-forge-authority/src/lib.rs:716-748` (`PolicyRule`, existing
  `memory_scope`)
- `crates/sea-forge-server/src/lib.rs:393-405` (`Request` enum)
- `crates/sea-forge-core/tests/version_skew.rs` (existing version-skew test
  precedent)
- `docs/decisions/ADR-002-audit-remediation-dependencies.md` (sealed-
  transcript key-file path)
- `.agents/CURRENT_STATUS.md` "Blockers" (historical M10-M16 contract
  inventory — superseded where those items have since landed; cross-checked
  against live source rather than assumed current)
