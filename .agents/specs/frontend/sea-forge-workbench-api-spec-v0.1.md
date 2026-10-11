# SEA Forge Workbench API Specification

**Protocol name:** SEA Forge Workbench Protocol (SFWP)  
**Status:** Target semantic specification v0.1 — repository grounding required  
**Primary transport recommendation:** Unix-domain socket with newline-delimited JSON frames  
**Primary consumers:** SEA Forge Workbench, first-party CLI adapters, trusted local integrations  
**Not selected:** GraphQL as the primary command/query/event surface

---

## 0. Purpose

This API connects the SEA Forge Workbench to the SEA Forge server without turning the frontend into a second kernel.

It is designed to make the correct mental model obvious:

```text
inspect what is true
→ evaluate what is possible
→ request one governed transition
→ observe durable facts
→ inspect evidence and settlement
→ understand what capability changed
```

The API is not generic CRUD.

It does not expose mutable resources that clients can patch into desired states. It exposes:

1. **source-backed views** for inspection;
2. **protected commands** for governed transitions;
3. **judgments** for approvals and declarations;
4. **verifications** for integrity, evidence, and projections;
5. **resumable events** describing facts that have occurred;
6. **content access** through explicit, disclosure-scoped references.

The contract optimizes for:

- low representational error;
- predictable naming;
- one intent per request;
- fail-closed behavior;
- explicit side-effect boundaries;
- deterministic recovery;
- progressive disclosure;
- machine precision plus human explanation;
- evidence reachability;
- compatibility with generated Rust/TypeScript contracts.

---

## 1. Canonical invariants

The API MUST preserve the following distinctions:

```text
navigation ≠ mutation
inspection ≠ protected evaluation
draft ≠ committed record
preflight allow ≠ execution authority
approval ≠ successful execution
execution state ≠ settlement state
agent dialogue state ≠ termination reason
agent termination ≠ settlement
settlement acceptance ≠ capability promotion
derived view ≠ source truth
event delivery ≠ source truth
imported record ≠ local authority
unknown ≠ absent
restricted ≠ empty
denied ≠ system error
retry ≠ replay
```

### 1.1 Server ownership

The server or kernel owns:

- actor identity resolution;
- sponsor eligibility;
- authority evaluation;
- disclosure-before-retrieval;
- semantic validation as final authority;
- case and plan reducers;
- sentry evaluation;
- execution and sandbox enforcement;
- agent permission mediation;
- evidence records;
- settlement evaluation;
- declaration standing;
- capability promotion and contraction;
- ledger integrity;
- artifact transition gates;
- federation trust and adoption;
- authoritative event ordering.

### 1.2 Client ownership

The client may own:

- local reversible drafts;
- navigation;
- filters and sorting;
- display density;
- selection and expansion state;
- keyboard interaction;
- formatting;
- client-side completeness hints clearly labeled non-authoritative;
- cached source-backed views with explicit freshness.

The client MUST NOT set canonical state directly.

---

## 2. Cognitive ergonomics

### 2.1 One request, one user intent

Every method name describes one recognizable user job.

Good:

```text
case.get_horizon
case.preflight
case.commit
run.cancel
settlement.get
capability.get
```

Rejected:

```text
case.update
resource.mutate
system.execute
workflow.process
record.patch
set_status
```

A command may cause several atomic records to be committed, but it represents one coherent user intent.

### 2.2 Stable grammar

Method names use:

```text
<domain>.<verb_or_question>
```

Approved verbs:

```text
list
get
search
inspect
preview
validate
preflight
create
register
commit
start
cancel
retry
decide
declare
verify
rebuild
request
subscribe
unsubscribe
```

`retry` always creates a new episode. `replay` repeats the same deterministic inputs for verification.

### 2.3 No generic success

Responses never contain only:

```json
{"success": true}
```

They use a typed outcome:

```text
view
committed
accepted_for_processing
denied
escalated
rejected_as_stale
failed
```

The domain state remains separate. For example:

```text
request outcome: committed
execution state: succeeded
settlement state: evaluating
```

### 2.4 Explanation is structured

Every substantial view or governed outcome includes:

```text
what state is true
why it is true
which source records support it
what limitations apply
which action paths are currently spendable
which paths are blocked and what is missing
```

Human-readable summaries are conveniences. Machine clients rely on typed fields and references.

### 2.5 Progressive disclosure

Default responses carry enough information to decide the next move.

Deeper evidence, traces, transcripts, and machine records are requested explicitly through references.

Supported detail levels:

```text
summary
standard
audit
```

The server may reject `audit` detail if disclosure does not permit it.

### 2.6 Unknown, unavailable, and restricted are explicit

Ambiguous `null` values are prohibited for meaningful domain facts.

Use a tagged value:

```ts
type KnowledgeValue<T> =
  | { kind: "known"; value: T }
  | { kind: "unknown"; reason_code: string; explanation: string }
  | { kind: "unavailable"; reason_code: string; explanation: string }
  | { kind: "restricted"; policy_ref: RecordRef; omitted_class: string }
  | { kind: "not_applicable" };
```

A restricted collection is not returned as an empty collection.

### 2.7 Exact next paths

The API may return action paths, but it must not manufacture recommendations.

```ts
interface ActionPaths {
  available: AvailableActionPath[];
  blocked: BlockedActionPath[];
  future: FutureActionPath[];
}
```

Only `available` paths count as current affordances: they are reachable and spendable from the current horizon.

A blocked path names the missing condition and the nearest lawful repair path.

---

## 3. Protocol architecture

```text
Workbench renderer
    ↓ typed request
native host / client adapter
    ↓ NDJSON frame
sea-forge-server
    ↓ authority / services / reducers
kernel + ledger + artifacts
    ↑ durable events and source-backed views
```

The semantic protocol is transport-independent. The recommended v1 carrier is the existing local Unix-socket NDJSON surface.

HTTP may be added later as another carrier for the same envelopes.

GraphQL is not the primary protocol because SEA Forge's central problem is exact protected commands, source references, authority dispositions, expected digests, resumable event cursors, and settlement—not arbitrary client-selected graph traversal.

---

## 4. Wire framing

Every JSON object occupies one NDJSON line.

### 4.1 Frame types

```text
hello
hello_result
request
response
event
stream_chunk
stream_end
ping
pong
protocol_error
```

A domain run cancellation is a method such as `run.cancel`. A transport-level request cancellation is only for abandoning a query or content stream and MUST NOT cancel governed work.

### 4.2 Request frame

```ts
interface RequestFrame<P = unknown> {
  type: "request";
  protocol: "sea-forge.workbench";
  protocol_version: string;

  request_id: RequestId;
  method: string;
  method_version: number;

  context: InteractionContext;
  preconditions?: Preconditions;
  options?: RequestOptions;
  params: P;
}
```

Example:

```json
{
  "type": "request",
  "protocol": "sea-forge.workbench",
  "protocol_version": "1.0",
  "request_id": "req_01K0...",
  "method": "case.get_horizon",
  "method_version": 1,
  "context": {
    "cell_id": "cell_01JZ...",
    "actor_id": "actor_sam",
    "role": "operator",
    "source_view": "case-horizon"
  },
  "options": {
    "detail": "standard",
    "locale": "en-US"
  },
  "params": {
    "case_id": "case_01K0..."
  }
}
```

### 4.3 Interaction context

```ts
interface InteractionContext {
  cell_id: CellId;
  actor_id: ActorId;
  role?: string;
  sponsor_ref?: RecordRef;

  case_id?: CaseId;
  plan_item_id?: PlanItemId;
  run_id?: RunId;

  source_view?: string;
  interaction_id?: string;
  return_route?: string;
}
```

`source_view`, `interaction_id`, and `return_route` are presentation context. They do not grant authority.

### 4.4 Preconditions

```ts
interface Preconditions {
  policy_bundle_digest?: Sha256;
  plan_digest?: Sha256;
  resource_digest?: Sha256;
  configuration_digest?: Sha256;
  self_model_snapshot_digest?: Sha256;
  expected_cursor?: EventCursor;

  records?: Array<{
    ref: RecordRef;
    expected_digest: Sha256;
  }>;
}
```

Protected commands use preconditions to prevent stale decisions.

A mismatch returns `rejected_as_stale` with the changed records and a repair action. It is not silently retried.

### 4.5 Request options

```ts
interface RequestOptions {
  detail?: "summary" | "standard" | "audit";
  locale?: string;
  page?: PageRequest;
  include?: string[];
  deadline_ms?: number;
}
```

`include` uses method-specific allow-listed values. There is no arbitrary field-selection language.

### 4.6 Response frame

```ts
interface ResponseFrame<T = unknown> {
  type: "response";
  protocol: "sea-forge.workbench";
  protocol_version: string;

  request_id: RequestId;
  method: string;
  method_version: number;

  outcome: ResponseOutcome<T>;
  response_meta: ResponseMeta;
}
```

### 4.7 Response outcomes

```ts
type ResponseOutcome<T> =
  | { kind: "view"; view: ViewEnvelope<T> }
  | { kind: "committed"; result: T; operation?: OperationRef }
  | { kind: "accepted_for_processing"; operation: OperationRef }
  | { kind: "denied"; denial: GovernedDenial }
  | { kind: "escalated"; escalation: GovernedEscalation }
  | { kind: "rejected_as_stale"; stale: StaleRejection }
  | { kind: "failed"; error: GovernedError };
```

Denial and escalation are governed outcomes, not protocol failures.

### 4.8 Response metadata

```ts
interface ResponseMeta {
  server_time: string;
  duration_ms: number;
  server_version: string;
  correlation_id: string;
  causation_id?: string;
  cursor?: EventCursor;
  warnings: ApiWarning[];
}
```

---

## 5. Self-description and negotiation

### 5.1 Hello

The first client frame SHOULD be:

```json
{
  "type": "hello",
  "protocol": "sea-forge.workbench",
  "supported_versions": ["1.0"],
  "client": {
    "name": "sea-forge-workbench",
    "version": "0.1.0"
  }
}
```

Result:

```ts
interface HelloResult {
  selected_protocol_version: string;
  server_version: string;
  server_instance_id: string;

  compatibility:
    | "compatible"
    | "compatible_with_limitations"
    | "upgrade_client"
    | "upgrade_server"
    | "incompatible";

  supported_features: string[];
  method_catalog_ref: SchemaRef;
  schema_catalog_ref: SchemaRef;
  limits: {
    max_request_bytes: number;
    max_inline_content_bytes: number;
    max_page_size: number;
    heartbeat_ms: number;
  };
}
```

### 5.2 Method description

`system.describe` returns descriptors such as:

```ts
interface MethodDescriptor {
  name: string;
  version: number;
  purpose: string;
  interaction_class: InteractionClass;

  authority_surface: string;
  mutation: "none" | "audit_append" | "append" | "derived_rebuild";
  side_effect_timing: "none" | "after_authority" | "asynchronous";

  request_schema_ref: SchemaRef;
  response_schema_ref: SchemaRef;
  event_topics: string[];

  settlement_target?: string;
  availability: KnowledgeValue<{
    enabled: boolean;
    limitations: string[];
  }>;
}
```

This makes the API inspectable by humans, CLIs, test harnesses, and code generators.

### 5.3 Schema format

The preferred external schema form is JSON Schema 2020-12 generated from canonical Rust types or an equivalent repository-native generator.

Generated artifacts include:

```text
source crate and type
source version
generator version
parameters
output digest
generated time
```

Handwritten duplicate canonical enums are prohibited when generation is feasible.

---

## 6. Interaction classes

| Class | Meaning | May create records? | Example |
|---|---|---:|---|
| `inspect` | Read a source-backed or derived view | No, unless explicitly audited by policy | `case.get_horizon` |
| `evaluate` | Perform bounded analysis or preflight | Often audit/evidence only | `case.preflight` |
| `propose` | Create a durable proposal with no execution privilege | Yes | `case.propose_replan` |
| `command` | Request a governed state transition or side effect | Yes, only after authority | `run.cancel` |
| `decide` | Exercise approval or declaration standing | Yes | `approval.decide` |
| `verify` | Verify evidence, integrity, projection, or readiness | Yes or derived rebuild | `integrity.verify` |
| `subscribe` | Receive ordered event facts | No target mutation | `events.subscribe` |

The method catalog exposes the class so clients know what kind of interaction they are presenting.

---

## 7. Common semantic types

### 7.1 Opaque identifiers

```ts
type CellId = string;
type ActorId = string;
type RequestId = string;
type OperationId = string;
type RecordId = string;
type CaseId = string;
type PlanItemId = string;
type RunId = string;
type ApprovalId = string;
type SettlementId = string;
type CapabilityId = string;
type EventCursor = string;
type Sha256 = string;
```

Clients treat identifiers as opaque.

### 7.2 Record reference

```ts
interface RecordRef {
  id: RecordId;
  kind: string;
  version: string;
  digest?: Sha256;
  display_label?: string;
}
```

### 7.3 View envelope

```ts
interface ViewEnvelope<T> {
  view_kind: string;
  view_version: number;
  data: T;

  explanation: StateExplanation;
  action_paths: ActionPaths;

  source: SourceSummary;
  freshness: FreshnessSummary;
  integrity: IntegritySummary;
  limitations: Limitation[];

  cursor?: EventCursor;
}
```

### 7.4 Source summary

```ts
interface SourceSummary {
  source_cell_id: CellId;
  record_refs: RecordRef[];
  authoritative_as_of: string;

  projection?: {
    kind: string;
    version: string;
    descriptor_ref: RecordRef;
    rebuilt_at?: string;
    output_digest: Sha256;
  };
}
```

### 7.5 Freshness

```ts
interface FreshnessSummary {
  state: "current" | "stale" | "unknown";
  last_verified_at?: string;
  invalidated_by?: RecordRef;
  explanation?: string;
}
```

### 7.6 Integrity

```ts
interface IntegritySummary {
  state:
    | "unverified"
    | "verifying"
    | "verified_local"
    | "tamper_evident"
    | "externally_witnessed"
    | "integrity_pending"
    | "invalid"
    | "fork_detected";

  assurance_label: string;
  checkpoint_ref?: RecordRef;
  witness_refs: RecordRef[];
  affected_operations: string[];
}
```

### 7.7 State explanation

```ts
interface StateExplanation {
  summary: string;
  state_domain: string;
  state_value: string;

  because: ConditionResult[];
  evidence_refs: RecordRef[];
  rule_refs: RecordRef[];
  limitations: string[];
}
```

Example:

```json
{
  "summary": "Execution completed, but settlement evaluation continues.",
  "state_domain": "settlement",
  "state_value": "evaluating",
  "because": [
    {
      "condition": "independent_declaration_present",
      "result": "not_satisfied",
      "explanation": "A qualifying reviewer has not declared yet."
    }
  ],
  "evidence_refs": ["evidence:ev_01..."],
  "rule_refs": ["criterion:crit_03"],
  "limitations": []
}
```

### 7.8 Action paths

```ts
interface AvailableActionPath {
  action_id: string;
  label: string;
  method: string;
  method_version: number;
  params_preview: unknown;

  authority: {
    surface: string;
    preflight_state?: string;
  };

  observed_burden?: BurdenSummary;
  settlement_target: string;
}

interface BlockedActionPath {
  action_id: string;
  label: string;
  method?: string;

  blocked_by: ConditionResult[];
  missing_conditions: string[];
  repair_actions: AvailableActionPath[];
}

interface FutureActionPath {
  action_id: string;
  label: string;
  activation_conditions: string[];
  source_refs: RecordRef[];
}
```

The API does not invent price or confidence. `observed_burden` is returned only when supported by evidence or a trusted external valuation feed.

### 7.9 Condition result

```ts
interface ConditionResult {
  condition: string;
  result: "satisfied" | "not_satisfied" | "unknown" | "restricted";
  explanation: string;
  source_refs?: RecordRef[];
}
```

### 7.10 Operation reference

```ts
interface OperationRef {
  operation_id: OperationId;
  operation_kind: string;
  state:
    | "accepted"
    | "awaiting_authority"
    | "awaiting_approval"
    | "queued"
    | "running"
    | "terminal";

  resource_refs: RecordRef[];
  status_method: "operation.get";
  event_scope?: EventScope;
}
```

---

## 8. Idempotency and stale-state protection

### 8.1 Request identity

Every request has a unique `request_id`.

For protected methods, the server stores enough request correlation to answer `request.get_status`.

### 8.2 Idempotency key

A client MAY provide:

```ts
interface Idempotency {
  key: string;
  scope: "actor_cell_method";
}
```

If the same key and identical canonical payload are seen again, the prior result is returned.

If the same key is reused with a different canonical payload, return:

```text
idempotency_conflict
```

### 8.3 Submission ambiguity

If the transport fails after submission:

```text
do not resubmit
→ call request.get_status
→ recover committed, denied, escalated, or accepted result
→ resubmit only after confirmed_not_received
```

### 8.4 Retry rule

A domain retry creates a new run or episode with a new ID and explicit lineage.

It never reopens or overwrites the terminal episode.

---

## 9. Pagination, search, and filtering

### 9.1 Cursor pagination

```ts
interface PageRequest {
  limit?: number;
  after?: string;
}

interface PageInfo {
  returned: number;
  has_more: boolean;
  next_cursor?: string;
}
```

Offset pagination is not used for mutable event-backed collections.

### 9.2 Typed filters

Each list/search method defines its own filter object.

Example:

```ts
interface CaseSearchFilter {
  states?: string[];
  owner_ids?: ActorId[];
  template_refs?: RecordRef[];
  has_attention?: boolean;
  settlement_states?: string[];
  changed_after?: string;
}
```

There is no public arbitrary SQL, GraphQL, SPARQL, or generic expression filter.

### 9.3 Sorting

Sort keys are method-specific enums.

The server always adds a stable tie-breaker.

Event and audit order uses append ordinal or opaque cursor, never wall-clock time alone.

---

## 10. Events and live consistency

### 10.1 Event envelope

```ts
interface EventFrame<E = unknown> {
  type: "event";
  protocol: "sea-forge.workbench";
  protocol_version: string;

  subscription_id: string;
  event_id: RecordId;
  topic: string;

  cursor: EventCursor;
  stream_id: string;
  ordinal: number;

  cell_id: CellId;
  case_id?: CaseId;
  run_id?: RunId;

  occurred_at: string;
  correlation_id?: string;
  causation_id?: string;

  attention: "none" | "notice" | "action_required" | "critical";
  payload: E;

  source_record_refs: RecordRef[];
  integrity: IntegritySummary;
}
```

### 10.2 Topic naming

Topics describe facts in past tense or state changes:

```text
cell.readiness_changed
projection.invalidated
case.committed
case.plan_version_committed
case.item_activated
case.item_blocked
case.milestone_achieved
case.state_changed
approval.created
approval.decided
approval.expired
run.accepted
run.execution_state_changed
run.output_available
run.evidence_committed
agent.dialogue_state_changed
agent.permission_requested
agent.permission_decided
agent.termination_recorded
agent.transcript_committed
settlement.evaluation_started
settlement.resolved
capability.changed
integrity.changed
artifact.transitioned
federation.bundle_imported
```

Events do not instruct clients to mutate truth. They announce facts and source references.

### 10.3 Subscription request

```ts
interface EventSubscriptionParams {
  scope: {
    cell_id: CellId;
    case_id?: CaseId;
    run_id?: RunId;
  };

  after?: EventCursor;
  topics?: string[];
  attention_at_least?: "none" | "notice" | "action_required" | "critical";
}
```

### 10.4 Reconnect algorithm

```text
1. Keep the last confirmed view and cursor.
2. Mark the live view stale.
3. Reconnect using `after`.
4. Apply events in ordinal order.
5. If a gap or unsupported event appears, stop local reduction.
6. Refetch the authoritative view.
7. Resume after the new cursor.
8. Clear stale only after continuity is verified.
```

### 10.5 Unknown event

An unknown event topic or schema version MUST cause:

```text
refetch_required
```

It must not be silently ignored when it may affect the active view.

---

## 11. Content and large payloads

### 11.1 Content references

Large evidence, logs, artifacts, and transcripts are represented by:

```ts
interface ContentRef {
  content_id: string;
  media_type: string;
  byte_length: number;
  sha256: Sha256;
  encoding: "utf-8" | "binary";
  sensitivity: "ordinary" | "restricted" | "sealed";
}
```

### 11.2 Inline threshold

Small content may be returned inline below the negotiated limit.

Large content is retrieved through a domain method such as:

```text
run.get_output
evidence.get_content
agent_run.get_transcript
```

### 11.3 Stream chunks

```ts
interface StreamChunkFrame {
  type: "stream_chunk";
  request_id: RequestId;
  stream_id: string;
  sequence: number;
  offset: number;
  data_base64?: string;
  text?: string;
  sha256_so_far?: Sha256;
}
```

Final frame:

```ts
interface StreamEndFrame {
  type: "stream_end";
  request_id: RequestId;
  stream_id: string;
  final_sha256: Sha256;
  total_bytes: number;
  complete: boolean;
}
```

### 11.4 Disclosure

Restricted content is authorized before retrieval.

The server does not fetch broadly and depend on the client to redact.

Secret values are never returned. Credentials remain opaque references and status.

---

## 12. Governed outcomes and errors

### 12.1 Denial

```ts
interface GovernedDenial {
  code: string;
  summary: string;
  authority_decision_ref: RecordRef;
  requested_action: string;
  requested_resource: string;
  boundaries: Boundary[];
  side_effect_state: "prevented";
  next_actions: AvailableActionPath[];
}
```

A denial proves that the action did not execute.

### 12.2 Escalation

```ts
interface GovernedEscalation {
  code: string;
  summary: string;
  approval_ref: RecordRef;
  expires_at?: string;
  requested_action: string;
  requested_resource: string;
  boundaries: Boundary[];
  next_actions: AvailableActionPath[];
}
```

### 12.3 Stale rejection

```ts
interface StaleRejection {
  code: "precondition_failed";
  summary: string;
  changed_records: Array<{
    ref: RecordRef;
    expected_digest?: Sha256;
    current_digest?: Sha256;
  }>;
  invalidated_checks: string[];
  next_actions: AvailableActionPath[];
}
```

### 12.4 Error

```ts
interface GovernedError {
  class:
    | "configuration"
    | "identity"
    | "authority"
    | "disclosure"
    | "model"
    | "plan"
    | "integrity"
    | "sandbox"
    | "environment"
    | "endpoint"
    | "acp"
    | "evidence"
    | "settlement"
    | "compatibility"
    | "transport"
    | "internal";

  code: string;
  summary: string;
  explanation: string;

  side_effect_state:
    | "none"
    | "prevented"
    | "partial"
    | "completed_before_failure"
    | "unknown";

  field_errors: FieldError[];
  source_refs: RecordRef[];
  evidence_refs: RecordRef[];

  recovery:
    | "never"
    | "after_correction"
    | "new_episode"
    | "query_request_status"
    | "repair_integrity";

  next_actions: AvailableActionPath[];
}
```

### 12.5 Error language

Every error answers:

```text
what failed
where it failed
whether side effects occurred
what evidence remains
what can happen next
```

Avoid generic messages such as `invalid state`.

---

## 13. Primary-path view contracts

### 13.1 Readiness

`readiness.get` returns:

```ts
interface ReadinessView {
  intended_operation?: {
    method: string;
    resource_ref?: RecordRef;
  };

  overall:
    | "ready"
    | "ready_with_limitations"
    | "blocked"
    | "integrity_halted"
    | "stale";

  snapshot_ref?: RecordRef;
  policy_bundle_ref?: RecordRef;

  foundations: ReadinessItem[];
  operational_capabilities: ReadinessItem[];
  recent_invalidations: Invalidation[];
}
```

`readiness.check` returns `accepted_for_processing` when probes continue asynchronously.

### 13.2 New case and configuration

The server returns entry options but local draft editing remains client-owned.

`case.get_entry_options` returns:

```ts
interface CaseEntryOptionsView {
  available: EntryOption[];
  unavailable: UnavailableEntryOption[];
  recommendation?: KnowledgeValue<{
    option_id: string;
    source: "deterministic_rule" | "trusted_external_router";
    explanation: string;
  }>;
  asset_snapshot_digest: Sha256;
}
```

A recommendation is never auto-selected.

### 13.3 Preflight

`case.preflight` returns:

```ts
interface CasePreflightView {
  contract_summary: CaseContractSummary;
  work_burden: WorkBurdenSummary;
  authority_burden: AuthorityBurdenSummary;
  settlement_burden: SettlementBurdenSummary;

  checks: PreflightCheck[];

  eligibility:
    | "ready"
    | "ready_with_warnings"
    | "blocked"
    | "escalation_expected";

  preflight_digest: Sha256;
  expected_digests: Preconditions;
}
```

The view summary explicitly states:

```text
Nothing has executed.
```

### 13.4 Commit

`case.commit` requires:

```ts
interface CommitCaseParams {
  preflight_digest: Sha256;
  draft_digest: Sha256;
  plan: CasePlanDraftWire;
}
```

A committed result includes exact created references:

```ts
interface CaseCommitted {
  case_ref: RecordRef;
  plan_ref: RecordRef;
  criterion_refs: RecordRef[];
  origin_refs: RecordRef[];
  initial_event_refs: RecordRef[];
}
```

### 13.5 Case horizon

`case.get_horizon` returns server-derived regions:

```ts
interface CaseHorizonView {
  case_ref: RecordRef;
  plan_ref: RecordRef;
  case_state: string;

  spendable_now: PlanItemView[];
  active: PlanItemView[];
  awaiting: PlanItemView[];
  blocked: PlanItemView[];
  future: PlanItemView[];
  settled: PlanItemView[];

  milestones: MilestoneView[];
  attention: AttentionItem[];
}
```

Clients cannot move an item between regions.

### 13.6 Run detail

`run.get` returns two separate state tracks:

```ts
interface RunDetailView {
  run_ref: RecordRef;
  plan_item_ref: RecordRef;

  execution_state: string;
  settlement_state: string;

  authority_boundary: AuthorityBoundaryView;
  sandbox: SandboxView;
  environment: EnvironmentExecutionView;

  trace_summary: TraceSummary;
  output_channels: OutputChannelDescriptor[];
  evidence: EvidenceRefView[];
  settlement_preview?: SettlementPreview;

  retry_lineage: RecordRef[];
}
```

### 13.7 Agent run

`agent_run.get` adds:

```ts
interface AgentRunDetailView extends RunDetailView {
  dialogue_state: string;
  termination_reason?: string;

  endpoint: AgentEndpointView;
  session: AgentSessionView;
  instruction: InstructionContractView;
  budget: AgentBudgetView;

  turns: AgentTurnSummary[];
  pending_permission?: ApprovalSummary;
  harness_proof: HarnessProofView[];
  transcript: TranscriptEvidenceView;
}
```

Agent narration is data, not settlement.

### 13.8 Settlement

`settlement.get` returns:

```ts
interface SettlementDetailView {
  settlement_ref: RecordRef;
  run_ref?: RecordRef;

  state:
    | "unsettled"
    | "evaluating"
    | "accepted"
    | "rejected"
    | "escalated"
    | "quarantined";

  expected_outcome: string;
  observed_outcome: string;
  plain_explanation: string;

  criteria: CriterionSettlementRow[];
  declarations: SettlementDeclarationView[];
  reliability: ReliabilityView;
  consequences: SettlementConsequences;
}
```

### 13.9 Capability

`capability.get` returns:

```ts
interface CapabilityDetailView {
  capability_ref: RecordRef;
  state:
    | "attempted"
    | "demonstrated"
    | "proven"
    | "degraded"
    | "quarantined";

  claim_summary: string;
  evidence_strength: EvidenceStrengthView;
  variation_coverage: VariationCoverageView;
  recovery_coverage: RecoveryCoverageView;
  orchestration_burden: BurdenTrendView;
  promotion: PromotionExplanationView;
  source_settlements: SettlementSummary[];
  next_proof_paths: ProofPathView[];
}
```

A proof path initiates a new case through preflight. It cannot mutate capability directly.

---

## 14. Thoth and disclosure

### 14.1 Typed questions only

`thoth.ask` accepts a tagged question union:

```ts
type ThothQuestion =
  | AskCapability
  | AskOperationRequirements
  | AskAuthorityRequirements
  | AskProjectionSupport
  | AskEnvironmentStatus
  | AskFailureExplanation
  | AskEvidenceForClaim
  | AskAvailableAffordances
  | AskWhyDenied;
```

There is no raw graph query method.

### 14.2 Disclosure before retrieval

The response records:

- disclosure decision;
- permitted claim classes;
- bounded query-plan digest;
- snapshot reference;
- omitted claim classes;
- evidence and settlement refs.

A denial shape must not vary according to restricted content beyond IDs and timestamps.

### 14.3 Grounded answer

```ts
interface ThothAnswerView {
  answer_ref: RecordRef;
  question_ref: RecordRef;
  snapshot_ref: RecordRef;

  claims: Array<{
    subject_ref: RecordRef;
    status:
      | "declared"
      | "installed"
      | "available"
      | "validated"
      | "demonstrated"
      | "degraded"
      | "unsupported"
      | "unknown";
    statement: string;
    evidence_refs: RecordRef[];
    settlement_refs: RecordRef[];
  }>;

  omitted_claim_classes: string[];
  limitations: string[];
  authority_notice: string;
}
```

Knowledge never confers execution authority.

---

## 15. Approval and agent permission ergonomics

ACP permission requests are represented through the same approval API.

An approval request identifies:

```text
originating agent run
requested tool action
exact resource boundary
existing run grant
expiry
policy basis
```

`approval.decide` rejects decisions when:

- the request expired;
- the request digest changed;
- the actor lacks standing;
- separation of duty fails.

Permission denial does not imply run cancellation.

---

## 16. Security and privacy

- Every request is bound to a cell and actor context.
- Sponsor context is explicit for automated actors.
- Secret values never appear in requests, responses, events, errors, evidence, or transcripts.
- Credentials use opaque `credential_ref` identifiers.
- Restricted data is scoped before retrieval.
- Cache keys include cell, actor, role, sponsor where material, and disclosure context.
- Switching role or cell invalidates incompatible cached content.
- File import and export require explicit user-selected boundaries.
- Unknown protocol variants fail closed.
- No provider or endpoint fallback occurs silently.
- Content hashes are calculated after required redaction rules.
- The client never reads canonical `.sea-forge` files as a normal API shortcut.

---

## 17. Versioning and compatibility

### 17.1 Protocol version

`protocol_version` uses major/minor semantics.

- Minor: additive methods, optional fields, new schemas behind negotiation.
- Major: changed meaning, removed field, incompatible enum or framing.

### 17.2 Method version

Every method has an integer `method_version`.

A server may support multiple method versions during migration.

### 17.3 Enum evolution

Clients MUST:

- exhaustively handle known variants;
- render unknown variants as `unknown` or `unsupported`;
- mark affected views stale or blocked when a safe interpretation is impossible;
- never map an unknown authority, settlement, or integrity state to permissive success.

### 17.4 Compatibility response

Incompatible clients may retain safe read-only inspection if schemas are understood, but protected commands are disabled.

---

## 18. Observability

Every request is correlated by:

```text
request_id
correlation_id
causation_id
cell_id
actor_id
method
operation_id when created
source record refs
event cursor
```

Client diagnostic telemetry must not contain:

- secrets;
- full transcripts;
- restricted evidence;
- unredacted domain source;
- approval notes unless explicitly governed.

Client logs are not settlement evidence unless deliberately captured and governed.

---

## 19. Conformance requirements

An implementation conforms when it proves:

1. protected commands cannot bypass authority;
2. stale preconditions block commit or decision;
3. transport ambiguity is resolved without duplicate operations;
4. denial proves zero side effects where required;
5. event gaps mark views stale and trigger authoritative recovery;
6. execution and settlement remain separate;
7. dialogue termination and settlement remain separate;
8. permission denial does not cancel the agent run;
9. evidence is reachable from every terminal result;
10. unknown, unavailable, restricted, and empty remain distinct;
11. Thoth disclosure occurs before retrieval;
12. capability views cannot exceed qualifying settlement evidence;
13. imported records cannot confer local authority or capability;
14. retries create new linked episodes;
15. derived stores can be rebuilt without becoming source truth;
16. generated client contracts remain synchronized with canonical server types.

---

## 20. Required end-to-end protocol proofs

### Proof A — Happy command path

```text
system.hello
→ readiness.get
→ case.get_entry_options
→ case.validate_draft
→ case.preflight
→ case.commit
→ events.subscribe(case)
→ case.start_item
→ run.get
→ settlement.get
→ capability.get
```

### Proof B — False-success correction

```text
run execution_state = succeeded
→ settlement resolved = rejected
→ case item remediation activated
→ capability not promoted
```

### Proof C — Approval escalation

```text
case.commit or case.start_item
→ outcome escalated with approval_ref
→ approval.get
→ approval.decide
→ original operation resumes or is newly committed
```

### Proof D — Agent permission denial

```text
agent.permission_requested event
→ approval.decide(reject)
→ agent.permission_decided event
→ dialogue remains active
→ settlement later evaluates independently
```

### Proof E — Ambiguous submission

```text
case.commit sent
→ socket disconnects
→ request.get_status
→ prior committed result returned
→ no duplicate case
```

### Proof F — Event gap

```text
event ordinal gap detected
→ view marked stale
→ events.get_range or authoritative query
→ continuity restored
```

### Proof G — Disclosure denial

```text
thoth.ask
→ disclosure denied
→ byte-stable denial class
→ no restricted retrieval
→ denial recorded as governed outcome
```

---

## 21. Repository-grounding instructions

This specification defines the target semantic API.

A repository inspection MUST produce a mapping table:

| Target method | Existing server request/type | Status | Decision |
|---|---|---|---|
| `case.get_horizon` | path + symbol | exists/partial/missing/conflict | reuse/adapt/add/reject |
| `case.commit` | path + symbol | … | … |
| `events.subscribe` | path + symbol | … | … |

The grounding pass must:

1. inspect `sea-forge-server` request and response enums;
2. inspect existing Unix-socket framing;
3. inspect request IDs and idempotency behavior;
4. inspect event cursor and ordinal types;
5. identify existing query/read-model types;
6. identify canonical Rust types suitable for schema generation;
7. merge equivalent methods rather than duplicating them;
8. preserve repository naming when it is equally clear;
9. rename repository methods only when ambiguity creates a real UX or safety cost;
10. mark unsupported target methods as `MISSING`, not pretend they exist;
11. reject target methods that duplicate kernel logic or weaken invariants;
12. generate a patched API spec with exact paths, symbols, and test owners.

The repository-grounded result should retain this API's mental model even when concrete method names change:

```text
inspect
evaluate
propose
command
decide
verify
subscribe
```

---

# Appendix A — Target method catalog

The catalog below is intentionally comprehensive enough to cover the current GUI page families. Repository grounding may merge methods where an existing contract already provides the same coherent user intent.

### system

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `system.hello` | `inspect` | Negotiate protocol compatibility and discover the server's supported contract surface. | `connection` | `none` | `HelloView` |
| `system.describe` | `inspect` | Return the self-describing method catalog, schema references, lifecycle meanings, and availability. | `connection` | `none` | `ApiDescriptionView` |
| `system.get_schema` | `inspect` | Return one versioned JSON Schema or equivalent generated contract by schema reference. | `connection` | `none` | `SchemaDocument` |

### request

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `request.get_status` | `inspect` | Resolve the authoritative result of a previously submitted request after transport ambiguity. | `same_actor_or_auditor` | `none` | `RequestStatusView` |

### operation

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `operation.get` | `inspect` | Inspect a durable asynchronous operation and its current authoritative state. | `resource_visibility` | `none` | `OperationView` |

### events

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `events.subscribe` | `subscribe` | Open a resumable, disclosure-scoped event subscription from an opaque durable cursor. | `event_scope_visibility` | `none` | `SubscriptionAccepted` |
| `events.unsubscribe` | `subscribe` | Close a client subscription without affecting governed work. | `subscription_owner` | `none` | `SubscriptionClosed` |
| `events.get_range` | `inspect` | Retrieve an authorized event range for deterministic gap recovery. | `event_scope_visibility` | `none` | `EventPage` |

### cell

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `cell.list` | `inspect` | List cells visible to the current actor with readiness and integrity summaries. | `cell_visibility` | `none` | `CellPage` |
| `cell.inspect_candidate` | `inspect` | Inspect a path or connection target before opening, creating, or migrating a cell. | `host_path_access` | `none` | `CellCandidateView` |
| `cell.create` | `command` | Create a new governed cell and its initial identity and ledger genesis records. | `cell_administration` | `append` | `CellCreated` |
| `cell.preview_migration` | `evaluate` | Produce a non-mutating migration plan, compatibility report, and expected consequences. | `cell_administration` | `audit_append` | `MigrationPreview` |
| `cell.migrate` | `command` | Append a verified migration into a new or existing cell without rewriting prior history. | `cell_administration` | `append` | `MigrationAccepted` |

### context

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `context.get` | `inspect` | Resolve the active cell, actor, role, sponsor, policy, and integrity context. | `self_context` | `none` | `ContextView` |
| `context.list_roles` | `inspect` | List roles the actor may select in the current cell and explain unavailable roles. | `self_context` | `none` | `RolePage` |

### sponsorship

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `sponsorship.request` | `propose` | Request an accountable sponsor binding for an automated actor or protected operation. | `sponsorship_request` | `append` | `SponsorshipRequestCreated` |

### authority

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `authority.preflight` | `evaluate` | Evaluate a proposed action and exact boundaries without conferring execution authority. | `authority_preflight` | `audit_append` | `AuthorityPreflightView` |
| `authority.get_decision` | `inspect` | Inspect one authority decision, its inputs, boundaries, and deterministic basis. | `decision_visibility` | `none` | `AuthorityDecisionView` |
| `authority.search` | `inspect` | Search authority, disclosure, approval, and permission history using typed filters. | `audit_visibility` | `none` | `AuthorityDecisionPage` |

### readiness

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `readiness.get` | `inspect` | Get operation-sensitive readiness, limitations, blockers, and currently spendable action paths. | `cell_visibility` | `none` | `ReadinessView` |
| `readiness.check` | `verify` | Run selected readiness checks and probes as a durable, evidenced operation. | `readiness_check` | `append` | `OperationAccepted` |

### self_model

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `self_model.validate` | `verify` | Verify the current self-model source, realization, and projection integrity. | `self_model_maintenance` | `append` | `SelfModelValidationResult` |
| `self_model.rebuild` | `command` | Build a new verified self-model snapshot while preserving the prior last-known-good snapshot. | `self_model_maintenance` | `append_and_projection` | `OperationAccepted` |

### thoth

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `thoth.list_question_kinds` | `inspect` | List supported typed question kinds and their required inputs without exposing raw graph access. | `self_disclosure` | `none` | `ThothQuestionKindPage` |
| `thoth.ask` | `evaluate` | Submit one typed, disclosure-gated question and receive a grounded answer or governed denial. | `self_disclosure` | `append` | `ThothAnswerResult` |
| `thoth.get_answer` | `inspect` | Inspect a prior answer or denial with claims, omissions, source snapshot, and evidence. | `self_disclosure` | `none` | `ThothAnswerView` |
| `thoth.replay_answer` | `verify` | Replay an answer against the same snapshot and policy inputs and record the comparison. | `self_disclosure_replay` | `append` | `ReplayResult` |

### asset

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `asset.list` | `inspect` | List visible templates, environments, endpoints, extensions, capabilities, and projections with actual availability. | `asset_visibility` | `none` | `AssetPage` |
| `asset.get` | `inspect` | Inspect one pinned asset version, dependencies, limitations, evidence, and usable contexts. | `asset_visibility` | `none` | `AssetView` |
| `asset.compare` | `inspect` | Compare two pinned asset versions without silently selecting or activating either. | `asset_visibility` | `none` | `AssetComparisonView` |

### domain_model

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `domain_model.list` | `inspect` | List visible domain model versions with validation, import, projection, and freshness state. | `model_visibility` | `none` | `DomainModelPage` |
| `domain_model.get` | `inspect` | Inspect one domain model version and its source, imports, concepts, validation, and projections. | `model_visibility` | `none` | `DomainModelView` |
| `domain_model.validate` | `verify` | Parse and semantically validate supplied domain source without replacing an existing model. | `model_validation` | `append` | `DomainModelValidationResult` |
| `domain_model.register` | `command` | Register a validated, version-pinned domain model as a new authoritative model record. | `model_administration` | `append` | `DomainModelRegistered` |
| `domain_model.compare` | `inspect` | Compare source, graph, concept, and compatibility drift between model versions. | `model_visibility` | `none` | `DomainModelComparisonView` |

### projection

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `projection.preview` | `evaluate` | Preview adapter compatibility, limitations, and expected outputs before projection. | `projection_preview` | `audit_append` | `ProjectionPreview` |
| `projection.create` | `command` | Run one deterministic, authority-gated projection from a pinned source and adapter. | `projection_execute` | `append_and_artifact` | `OperationAccepted` |
| `projection.get` | `inspect` | Inspect projection inputs, adapter version, output hashes, validation, and acceptance state. | `projection_visibility` | `none` | `ProjectionView` |
| `projection.rebuild` | `verify` | Rebuild a projection from identical pinned inputs and compare deterministic outputs. | `projection_execute` | `append_and_projection` | `OperationAccepted` |

### case

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `case.list` | `inspect` | List visible cases with source-derived state, attention, settlement, and owner summaries. | `case_visibility` | `none` | `CasePage` |
| `case.get_entry_options` | `inspect` | Return currently usable case-entry paths and explain unavailable paths for the actor and cell. | `case_create` | `none` | `CaseEntryOptionsView` |
| `case.validate_draft` | `evaluate` | Validate a draft's semantics, references, compatibility, criteria, and topology without committing it. | `case_create` | `audit_append_optional` | `CaseDraftValidationResult` |
| `case.preflight` | `evaluate` | Produce the exact immutable contract, burden, authority, settlement, and drift checks before commit. | `case_create` | `audit_append` | `CasePreflightView` |
| `case.commit` | `command` | Atomically commit a preflighted case, plan, criteria, origins, and initial events. | `case_create` | `append` | `CaseCommitted` |
| `case.get_overview` | `inspect` | Get the case identity, current consequence, attention items, milestones, and latest settlement effects. | `case_visibility` | `none` | `CaseOverviewView` |
| `case.get_horizon` | `inspect` | Get source-derived spendable, active, awaiting, blocked, future, and settled plan items. | `case_visibility` | `none` | `CaseHorizonView` |
| `case.get_timeline` | `inspect` | Get causally linked case events ordered by authoritative append order. | `case_visibility` | `none` | `CaseTimelinePage` |
| `case.get_plan` | `inspect` | Inspect one immutable plan version, sentries, criteria, origins, and current drift. | `case_visibility` | `none` | `CasePlanView` |
| `case.start_item` | `command` | Request activation or execution of one currently eligible plan item after fresh sentry and authority evaluation. | `plan_item_start` | `append` | `OperationAccepted` |
| `case.add_discretionary_item` | `propose` | Propose one bounded, provenance-linked discretionary plan item through the normal planner path. | `plan_mutation` | `append` | `PlanMutationProposal` |
| `case.propose_replan` | `propose` | Create a new plan-version proposal while preserving the active and historical plan versions. | `plan_mutation` | `append` | `ReplanProposalCreated` |
| `case.commit_replan` | `command` | Commit an approved, freshly preflighted replan as a new active plan version. | `plan_mutation` | `append` | `ReplanCommitted` |
| `case.reopen` | `command` | Reopen an eligible completed or terminated case through a new governed event. | `case_control` | `append` | `CaseReopened` |
| `case.terminate` | `command` | Terminate one case with explicit basis while preserving active run and evidence semantics. | `case_control` | `append` | `CaseTerminationAccepted` |

### approval

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `approval.list` | `inspect` | List approvals visible to the actor, including expiry, impact, and blocked origin. | `approval_visibility` | `none` | `ApprovalPage` |
| `approval.get` | `inspect` | Inspect one exact approval request, boundaries, evidence, policy basis, and downstream consequence. | `approval_visibility` | `none` | `ApprovalView` |
| `approval.decide` | `decide` | Approve or reject one unchanged, unexpired request after standing and separation-of-duty checks. | `approval_decision` | `append` | `ApprovalDecisionResult` |

### human_task

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `human_task.list` | `inspect` | List enabled, active, completed, rejected, or expired human tasks visible to the actor. | `human_task_visibility` | `none` | `HumanTaskPage` |
| `human_task.get` | `inspect` | Inspect one human task's instructions, criteria, evidence requirements, and authority context. | `human_task_visibility` | `none` | `HumanTaskView` |
| `human_task.complete` | `decide` | Submit required evidence and complete one human task against its declared criteria. | `human_task_complete` | `append` | `HumanTaskCompletionResult` |
| `human_task.reject` | `decide` | Reject or escalate one human task when the task contract permits that judgment. | `human_task_decision` | `append` | `HumanTaskDecisionResult` |

### run

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `run.list` | `inspect` | List command and agent runs with execution, settlement, sandbox, and attention summaries. | `run_visibility` | `none` | `RunPage` |
| `run.get` | `inspect` | Inspect one command run's execution state, boundaries, trace, evidence, and settlement handoff. | `run_visibility` | `none` | `RunDetailView` |
| `run.get_output` | `inspect` | Retrieve or stream one authorized output channel by content reference and byte range. | `run_output_visibility` | `none` | `ContentStream` |
| `run.get_trace` | `inspect` | Retrieve a typed, paginated run trace ordered by authoritative event ordinal. | `run_visibility` | `none` | `TracePage` |
| `run.cancel` | `command` | Request cancellation of one run episode only; preserve partial evidence and await a terminal event. | `run_cancel` | `append` | `CancellationRequested` |
| `run.retry` | `command` | Create a new linked run episode after a terminal result; never rewrite or resume the old episode implicitly. | `run_retry` | `append` | `OperationAccepted` |

### agent_run

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `agent_run.get` | `inspect` | Inspect one agent delegation's dialogue, termination, budgets, permissions, proof, transcript, and settlement. | `agent_run_visibility` | `none` | `AgentRunDetailView` |
| `agent_run.get_transcript_summary` | `inspect` | Retrieve an authorized structured transcript summary without broad transcript access. | `transcript_summary_visibility` | `none` | `TranscriptSummaryView` |
| `agent_run.request_transcript_access` | `propose` | Request access to a sealed or restricted canonical transcript under disclosure policy. | `transcript_access` | `append` | `TranscriptAccessRequest` |
| `agent_run.get_transcript` | `inspect` | Retrieve an authorized transcript or bounded range after disclosure is granted. | `transcript_access` | `none` | `ContentStream` |
| `agent_run.cancel` | `command` | Request scoped cancellation of one agent episode without cancelling sibling work. | `run_cancel` | `append` | `CancellationRequested` |
| `agent_run.retry` | `command` | Create a new linked agent episode using explicit continuation policy and current authority. | `run_retry` | `append` | `OperationAccepted` |

### operations

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `operations.get_overview` | `inspect` | Get live active, parked, capacity-waiting, approval-waiting, and interrupted work summaries. | `operations_visibility` | `none` | `OperationsOverviewView` |

### evidence

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `evidence.search` | `inspect` | Search authorized evidence by case, run, criterion, type, producer, digest, and verification state. | `evidence_visibility` | `none` | `EvidencePage` |
| `evidence.get` | `inspect` | Inspect one evidence record, content descriptor, producer, criterion links, and verification. | `evidence_visibility` | `none` | `EvidenceView` |
| `evidence.get_content` | `inspect` | Retrieve authorized evidence content or a bounded range through an explicit content reference. | `evidence_content_visibility` | `none` | `ContentStream` |
| `evidence.verify` | `verify` | Recalculate and record verification of one evidence or artifact descriptor. | `evidence_verify` | `append` | `EvidenceVerificationResult` |

### settlement

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `settlement.list` | `inspect` | List pending and terminal settlements with basis, consequence, and required judgment summaries. | `settlement_visibility` | `none` | `SettlementPage` |
| `settlement.get` | `inspect` | Inspect expected versus observed outcome, criterion matrix, evidence, declarations, standing, and consequence. | `settlement_visibility` | `none` | `SettlementDetailView` |
| `settlement.declare` | `decide` | Submit one scoped settlement declaration when the actor has standing and separation of duty permits. | `settlement_declaration` | `append` | `SettlementDeclarationResult` |

### audit

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `audit.search` | `inspect` | Search typed authority, approval, disclosure, mutation, configuration, and lifecycle audit records. | `audit_visibility` | `none` | `AuditPage` |
| `audit.get` | `inspect` | Inspect one audit record and its linked request, decision, evidence, and configuration digests. | `audit_visibility` | `none` | `AuditRecordView` |

### integrity

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `integrity.get_status` | `inspect` | Get assurance state, invalid streams, checkpoints, witnesses, and affected operations. | `integrity_visibility` | `none` | `IntegrityStatusView` |
| `integrity.verify` | `verify` | Verify one declared ledger scope without rewriting history. | `integrity_verify` | `append` | `IntegrityVerificationResult` |
| `integrity.prove_inclusion` | `verify` | Generate an inclusion proof for one authorized record and checkpoint. | `integrity_proof` | `append_optional` | `InclusionProofResult` |
| `integrity.compare_checkpoints` | `verify` | Generate or verify a consistency proof between two checkpoints. | `integrity_proof` | `append_optional` | `ConsistencyProofResult` |

### memory

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `memory.recall` | `evaluate` | Perform disclosure-scoped memory recall and record which source memories were returned or denied. | `memory_recall` | `append` | `MemoryRecallResult` |
| `memory.get` | `inspect` | Inspect one memory item, provenance, influence links, freshness, and source records. | `memory_visibility` | `none` | `MemoryItemView` |
| `memory.rebuild_index` | `verify` | Rebuild the derived memory index from authoritative source records. | `memory_maintenance` | `derived_rebuild` | `OperationAccepted` |

### capability

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `capability.list` | `inspect` | List evidence-backed capability states with variation, recovery, regressions, and freshness. | `capability_visibility` | `none` | `CapabilityPage` |
| `capability.get` | `inspect` | Inspect one capability's ladder, source settlements, promotion policy, gaps, and proof paths. | `capability_visibility` | `none` | `CapabilityDetailView` |
| `capability.assess_routing` | `evaluate` | Assess whether one capability supports a proposed context without granting execution authority. | `capability_routing` | `audit_append` | `CapabilityRoutingAssessment` |
| `capability.rebuild_projection` | `verify` | Rebuild capability projections from qualifying source records and compare the result. | `capability_maintenance` | `derived_rebuild` | `OperationAccepted` |

### pipeline

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `pipeline.get` | `inspect` | Inspect a specification pipeline's stages, inputs, outputs, proofs, quarantine, and settlement. | `pipeline_visibility` | `none` | `PipelineView` |
| `pipeline.start_stage` | `command` | Request activation of one currently eligible pipeline stage through normal case controls. | `plan_item_start` | `append` | `OperationAccepted` |
| `pipeline.rework_stage` | `propose` | Propose a bounded rework path for a rejected or quarantined stage. | `plan_mutation` | `append` | `PlanMutationProposal` |

### artifact

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `artifact.list` | `inspect` | List artifacts with identity, lineage, lifecycle, maturity, review, license, and attestation facets. | `artifact_visibility` | `none` | `ArtifactPage` |
| `artifact.get` | `inspect` | Inspect one artifact version, content hash, lineage, evidence, maturity, and eligible transitions. | `artifact_visibility` | `none` | `ArtifactView` |
| `artifact.verify` | `verify` | Verify artifact bytes and descriptor identity without changing maturity. | `artifact_verify` | `append` | `ArtifactVerificationResult` |
| `artifact.preview_transition` | `evaluate` | Preview a derive or promote transition, required gates, standing, and missing evidence. | `artifact_transition` | `audit_append` | `ArtifactTransitionPreview` |
| `artifact.request_transition` | `command` | Submit one valid artifact transition request and create the target or approval path when allowed. | `artifact_transition` | `append` | `ArtifactTransitionResult` |

### federation

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `federation.preview_export` | `evaluate` | Preview an export bundle, dependency closure, exclusions, and secret-reference handling. | `federation_export` | `audit_append` | `FederationExportPreview` |
| `federation.export` | `command` | Create a signed, hash-linked export bundle without exporting secret values. | `federation_export` | `append_and_artifact` | `FederationExportResult` |
| `federation.verify_import` | `verify` | Verify a bundle atomically before any local records are appended. | `federation_import` | `append_optional` | `FederationImportVerification` |
| `federation.import` | `command` | Import a fully verified bundle into isolated local history and assets without activation. | `federation_import` | `append` | `FederationImportResult` |
| `federation.request_adoption` | `propose` | Request local adoption of selected imported assets or records under current authority. | `federation_adoption` | `append` | `AdoptionRequestCreated` |

### extension

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `extension.list` | `inspect` | List extension descriptors, compatibility, registry state, and affected capabilities. | `admin_visibility` | `none` | `ExtensionPage` |
| `extension.get` | `inspect` | Inspect one extension's descriptor, version, configuration digest, status, and evidence. | `admin_visibility` | `none` | `ExtensionView` |
| `extension.register` | `command` | Register a validated extension descriptor without silently activating it. | `extension_admin` | `append` | `ExtensionRegistered` |
| `extension.activate` | `command` | Activate one compatible extension version and invalidate affected projections explicitly. | `extension_admin` | `append` | `ExtensionActivationResult` |
| `extension.disable` | `command` | Disable one extension version while preserving history and impact visibility. | `extension_admin` | `append` | `ExtensionDisableResult` |

### endpoint

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `endpoint.list` | `inspect` | List configured agent and external endpoints without returning credential values. | `endpoint_visibility` | `none` | `EndpointPage` |
| `endpoint.get` | `inspect` | Inspect one endpoint snapshot, protocol, model metadata, probe state, boundaries, and credential reference status. | `endpoint_visibility` | `none` | `EndpointView` |
| `endpoint.probe` | `verify` | Run one governed endpoint probe with exact network and credential boundaries. | `endpoint_probe` | `append` | `OperationAccepted` |

### environment

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `environment.list` | `inspect` | List execution environments, supplied tools/evaluators, compatibility, and validation state. | `environment_visibility` | `none` | `EnvironmentPage` |
| `environment.get` | `inspect` | Inspect one environment contract, tools, evaluators, versions, and limitations. | `environment_visibility` | `none` | `EnvironmentView` |
| `environment.validate` | `verify` | Validate one environment contract and its actual tool availability. | `environment_admin` | `append` | `EnvironmentValidationResult` |

### policy

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `policy.get_active` | `inspect` | Inspect the active hash-addressed policy bundle and declared surfaces without exposing secrets. | `policy_visibility` | `none` | `PolicyBundleView` |
| `policy.reload` | `command` | Validate and activate a new policy bundle while retaining the last-known-good bundle on failure. | `policy_admin` | `append` | `PolicyReloadResult` |

### maintenance

| Method | Class | Purpose | Authority surface | Mutation | Primary response |
|---|---|---|---|---|---|
| `maintenance.list_debt` | `inspect` | List stale projections, failed probes, interrupted work, expired approvals, quarantine, and version skew. | `maintenance_visibility` | `none` | `MaintenanceDebtPage` |
| `maintenance.get` | `inspect` | Inspect one debt item's affected invariants, evidence, and bounded repair paths. | `maintenance_visibility` | `none` | `MaintenanceDebtView` |
| `maintenance.repair` | `command` | Execute one explicit bounded repair or derived-state rebuild without rewriting authoritative failures. | `maintenance_repair` | `append_or_rebuild` | `OperationAccepted` |

---

# Appendix B — Example flows

## B.1 Hello and readiness

```json
{"type":"hello","protocol":"sea-forge.workbench","supported_versions":["1.0"],"client":{"name":"sea-forge-workbench","version":"0.1.0"}}
```

```json
{
  "type": "request",
  "protocol": "sea-forge.workbench",
  "protocol_version": "1.0",
  "request_id": "req_01K0A",
  "method": "readiness.get",
  "method_version": 1,
  "context": {
    "cell_id": "cell_foundry_01",
    "actor_id": "actor_sam",
    "role": "operator"
  },
  "params": {
    "intended_operation": {
      "method": "case.commit"
    }
  }
}
```

A blocked response still uses a `view` outcome because the query succeeded:

```json
{
  "type": "response",
  "protocol": "sea-forge.workbench",
  "protocol_version": "1.0",
  "request_id": "req_01K0A",
  "method": "readiness.get",
  "method_version": 1,
  "outcome": {
    "kind": "view",
    "view": {
      "view_kind": "readiness",
      "view_version": 1,
      "data": {
        "overall": "blocked"
      },
      "explanation": {
        "summary": "Case commitment is blocked because ledger integrity has not verified.",
        "state_domain": "readiness",
        "state_value": "blocked",
        "because": [
          {
            "condition": "required_integrity",
            "result": "not_satisfied",
            "explanation": "Stream authority-decisions failed verification."
          }
        ],
        "evidence_refs": [],
        "rule_refs": [],
        "limitations": []
      },
      "action_paths": {
        "available": [
          {
            "action_id": "open_integrity",
            "label": "Inspect integrity",
            "method": "integrity.get_status",
            "method_version": 1,
            "params_preview": {},
            "authority": {"surface": "integrity_visibility"},
            "settlement_target": "Identify the invalid scope and a bounded repair path."
          }
        ],
        "blocked": [
          {
            "action_id": "commit_case",
            "label": "Commit governed case",
            "method": "case.commit",
            "blocked_by": [],
            "missing_conditions": ["required ledger integrity"],
            "repair_actions": []
          }
        ],
        "future": []
      },
      "source": {
        "source_cell_id": "cell_foundry_01",
        "record_refs": [],
        "authoritative_as_of": "2026-07-24T00:00:00Z"
      },
      "freshness": {"state": "current"},
      "integrity": {
        "state": "invalid",
        "assurance_label": "ledger_integrity_error",
        "witness_refs": [],
        "affected_operations": ["case.commit"]
      },
      "limitations": []
    }
  },
  "response_meta": {
    "server_time": "2026-07-24T00:00:00Z",
    "duration_ms": 12,
    "server_version": "0.1.0",
    "correlation_id": "corr_01K0A",
    "warnings": []
  }
}
```

## B.2 Preflight and commit

Preflight:

```json
{
  "type": "request",
  "protocol": "sea-forge.workbench",
  "protocol_version": "1.0",
  "request_id": "req_01K0B",
  "method": "case.preflight",
  "method_version": 1,
  "context": {
    "cell_id": "cell_foundry_01",
    "actor_id": "actor_sam",
    "role": "operator",
    "source_view": "case-preflight"
  },
  "preconditions": {
    "policy_bundle_digest": "sha256:pb14",
    "self_model_snapshot_digest": "sha256:sm019"
  },
  "params": {
    "draft": {"draft_id": "draft_local_17", "content": "..."}
  }
}
```

Commit:

```json
{
  "type": "request",
  "protocol": "sea-forge.workbench",
  "protocol_version": "1.0",
  "request_id": "req_01K0C",
  "method": "case.commit",
  "method_version": 1,
  "context": {
    "cell_id": "cell_foundry_01",
    "actor_id": "actor_sam",
    "role": "operator",
    "source_view": "case-preflight"
  },
  "preconditions": {
    "policy_bundle_digest": "sha256:pb14",
    "plan_digest": "sha256:plan-draft",
    "resource_digest": "sha256:preflight"
  },
  "params": {
    "preflight_digest": "sha256:preflight",
    "draft_digest": "sha256:draft",
    "plan": {}
  }
}
```

If policy changed:

```json
{
  "kind": "rejected_as_stale",
  "stale": {
    "code": "precondition_failed",
    "summary": "Commit blocked because the policy bundle changed after preflight.",
    "changed_records": [
      {
        "ref": {"id":"policy_pb15","kind":"policy_bundle","version":"15"},
        "expected_digest": "sha256:pb14",
        "current_digest": "sha256:pb15"
      }
    ],
    "invalidated_checks": ["authority surfaces", "approval expectations"],
    "next_actions": []
  }
}
```

## B.3 Run succeeds but settlement rejects

Event sequence:

```json
{"type":"event","topic":"run.execution_state_changed","cursor":"cur_1180","stream_id":"run_07","ordinal":31,"payload":{"execution_state":"succeeded"}}
{"type":"event","topic":"settlement.evaluation_started","cursor":"cur_1181","stream_id":"run_07","ordinal":32,"payload":{"settlement_id":"set_22"}}
{"type":"event","topic":"settlement.resolved","cursor":"cur_1182","stream_id":"run_07","ordinal":33,"payload":{"state":"rejected","basis":["recovery_test_failed"]}}
{"type":"event","topic":"case.item_activated","cursor":"cur_1183","stream_id":"case_01","ordinal":76,"payload":{"plan_item_id":"pi_remediation","cause_ref":"set_22"}}
```

The client displays:

```text
Execution: SUCCEEDED
Settlement: REJECTED
Next spendable path: Open activated remediation
```

## B.4 Agent permission denial without cancellation

```json
{"type":"event","topic":"agent.permission_requested","cursor":"cur_201","stream_id":"agent_run_12","ordinal":14,"payload":{"approval_id":"apr_77","action":"write","resource":"crates/agent/src/acp.rs"}}
```

Decision:

```json
{
  "type": "request",
  "protocol": "sea-forge.workbench",
  "protocol_version": "1.0",
  "request_id": "req_01K0D",
  "method": "approval.decide",
  "method_version": 1,
  "context": {
    "cell_id": "cell_foundry_01",
    "actor_id": "actor_reviewer",
    "role": "approver",
    "run_id": "agent_run_12"
  },
  "params": {
    "approval_id": "apr_77",
    "request_digest": "sha256:permission-request",
    "decision": "reject",
    "note": "Write boundary not permitted in this episode."
  }
}
```

Resulting facts:

```json
{"type":"event","topic":"agent.permission_decided","cursor":"cur_202","stream_id":"agent_run_12","ordinal":15,"payload":{"approval_id":"apr_77","decision":"rejected"}}
{"type":"event","topic":"agent.dialogue_state_changed","cursor":"cur_203","stream_id":"agent_run_12","ordinal":16,"payload":{"dialogue_state":"streaming"}}
```

The run remains alive within its remaining grant.

---

# Appendix C — Anti-patterns

The following API designs are non-conforming:

- generic `PATCH /resource` or `record.update`;
- a single `status` field spanning authority, execution, settlement, and capability;
- `success: true` without a typed outcome;
- GraphQL mutations that bypass the protected-command envelope;
- arbitrary GraphQL or SPARQL access to self-model data;
- broad retrieval followed by client-side redaction;
- offset pagination for mutable event-backed history;
- timestamp-only event ordering;
- automatic resubmission after a socket error;
- retry that reuses or mutates a terminal run ID;
- optimistic approval, cancellation, settlement, capability, adoption, or integrity;
- event payloads treated as the only source of truth;
- secret values in API frames;
- unknown enum variants mapped to allow, accepted, ready, or proven;
- one API method that both proposes and approves its own work;
- agent output interpreted as settlement;
- imported evidence interpreted as local capability without adoption and qualifying settlement.

