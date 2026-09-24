/**
 * Complete typed application boundary for GodSpeed Casework & Cognitive Environment.
 * Matches schemas in ../schemas/ and adheres to Anti-Collapse Axioms.
 */

export type ActorRole =
  | 'developer'
  | 'security_officer'
  | 'case_architect'
  | 'auditor'
  | 'customer_stakeholder'
  | 'agent_operator';

export type CognitiveObjectKind =
  | 'work_item'
  | 'stage'
  | 'milestone'
  | 'decision_gate'
  | 'discretionary_opportunity'
  | 'execution_trace'
  | 'evidence_record';

export type CognitiveObjectStatus =
  | 'WAITING'
  | 'READY_TO_BEGIN'
  | 'IN_PROGRESS'
  | 'ACTION_REQUIRED'
  | 'WAITING_ON_OTHERS'
  | 'COMPLETED'
  | 'REJECTED'
  | 'FAILED'
  | 'AVAILABLE_TO_ADD'
  | 'ACTIVE';

export type ActionIntentKind =
  | 'BEGIN_WORK'
  | 'PROPOSE_CASE'
  | 'APPROVE_HUMAN_TASK'
  | 'REJECT_HUMAN_TASK'
  | 'ADD_DISCRETIONARY_WORK'
  | 'EXECUTE_ITEM'
  | 'COMPLETE_HUMAN_TASK'
  | 'REOPEN_WORK'
  | 'REOPEN_CASE'
  | 'TERMINATE_CASE'
  | 'ESCALATE_OR_OVERRIDE'
  | 'OPEN_ARTIFACT'
  | 'RESOLVE_SOURCE'
  | 'EXPORT_AUDIT_BUNDLE'
  | 'RESUME_LIVE_STREAM';

export type ActionVariant = 'PRIMARY' | 'SECONDARY' | 'DANGER' | 'WARNING' | 'GHOST';

export interface ActionDescriptor {
  id: string;
  label: string;
  intent: ActionIntentKind;
  variant?: ActionVariant;
  consequential: boolean;
  requires_justification?: boolean;
}

export interface CognitiveObject {
  id: string;
  kind: CognitiveObjectKind;
  name: string;
  status: CognitiveObjectStatus;
  badge: string;
  explanation?: string;
  salience: number; // 0.0 to 1.0
  parent_id?: string;
  depends_on?: string[];
  spatial_layout?: {
    x: number;
    y: number;
    z: number;
    radius?: number;
  };
  actions: ActionDescriptor[];
}

export interface ActorPerspective {
  actor_id: string;
  role: ActorRole;
  display_name?: string;
}

export interface WorldSummary {
  headline: string;
  phase: string;
  status_phrase: string;
  progress_percent?: number;
}

export interface AttentionFocus {
  primary_object_id: string;
  salience_rank?: string[];
  narration?: string;
}

export interface CognitiveWorldSnapshot {
  world_id: string;
  case_id: string;
  cursor: string; // <epoch>.<seq> e.g. "1.0000000042"
  timestamp: string; // ISO RFC3339
  perspective: ActorPerspective;
  summary: WorldSummary;
  visible_objects: CognitiveObject[];
  available_actions: ActionDescriptor[];
  attention_focus: AttentionFocus;
}

/** Complete wire-level action vocabulary accepted by InteractionIntent. */
export type InteractionActionName =
  | 'SELECT_OBJECT'
  | 'NAVIGATE_CAMERA'
  | 'FILTER_VIEW'
  | 'RESOLVE_SOURCE'
  | 'REQUEST_DEEP_PROJECTION'
  | 'REQUEST_HISTORY_SCRUB'
  | 'BEGIN_WORK'
  | 'PROPOSE_CASE'
  | 'APPROVE_HUMAN_TASK'
  | 'REJECT_HUMAN_TASK'
  | 'ADD_DISCRETIONARY_WORK'
  | 'EXECUTE_ITEM'
  | 'COMPLETE_HUMAN_TASK'
  | 'REOPEN_WORK'
  | 'REOPEN_CASE'
  | 'TERMINATE_CASE'
  | 'ESCALATE_OR_OVERRIDE'
  | 'OPEN_ARTIFACT'
  | 'EXPORT_AUDIT_BUNDLE'
  | 'RESUME_LIVE_STREAM';

/**
 * The consequential journey intents that cross the gateway as CONSEQUENTIAL_CASE intents
 * (T01 wire set). Every member is also an ActionIntentKind and an InteractionActionName.
 */
export type ConsequentialIntentName =
  | 'PROPOSE_CASE'
  | 'ADD_DISCRETIONARY_WORK'
  | 'EXECUTE_ITEM'
  | 'COMPLETE_HUMAN_TASK'
  | 'APPROVE_HUMAN_TASK'
  | 'REJECT_HUMAN_TASK'
  | 'ESCALATE_OR_OVERRIDE'
  | 'OPEN_ARTIFACT'
  | 'REOPEN_CASE'
  | 'TERMINATE_CASE';

export type InteractionIntentKind =
  | 'REACT_LOCAL'
  | 'BACKEND_INFORMATION'
  | 'CONSEQUENTIAL_CASE';

export interface InteractionIntent {
  intent_id: string; // uuid
  kind: InteractionIntentKind;
  action_name: InteractionActionName;
  target_object_id: string;
  case_id: string;
  client_cursor: string;
  actor: {
    actor_id: string;
    role: ActorRole;
  };
  parameters?: Record<string, unknown>;
  justification?: string;
}

/** Typed refusal vocabulary for refused intents. A refusal is an outcome, not a transport error. */
export type IntentRefusalKind =
  | 'AUTHORITY_DENIED'
  | 'UNAUTHORIZED_ROLE'
  | 'SOD_VIOLATION'
  | 'STALE_PROJECTION'
  | 'JUSTIFICATION_REQUIRED'
  | 'UNAVAILABLE'
  | 'INVALID';

/** The typed refusal envelope carried by a failed IntentResponse. */
export interface IntentRefusal {
  refusal_kind: IntentRefusalKind;
  message: string;
  /** Current projection cursor, when the gateway knows a fresher one (e.g. STALE_PROJECTION). */
  current_cursor?: string;
}

export interface IntentResponse {
  intent_id: string;
  success: boolean;
  new_cursor?: string;
  /** Present exactly when success is false; the typed refusal envelope. */
  refusal?: IntentRefusal;
  error_code?: string;
  error_message?: string;
  resulting_object?: CognitiveObject;
}

// ---------------------------------------------------------------------------
// Typed intent request payloads (T01). Each rides in InteractionIntent.parameters
// under the action name it is keyed by; intents without an entry here carry only
// the envelope fields (target_object_id, case_id, justification).

/** PROPOSE_CASE: commit a new case from a template after a passing preflight. */
export interface ProposeCasePayload {
  template_ref: string;
  params: Record<string, unknown>;
  /** Digest of the passing preflight the commit must echo (binds plan to what was reviewed). */
  preflight_digest: string;
}

/** ADD_DISCRETIONARY_WORK: propose optional work anchored to a stage (and optionally an item). */
export interface AddDiscretionaryWorkPayload {
  case_id: string;
  stage_id: string;
  anchor_item_id?: string;
  kind: CognitiveObjectKind;
  title: string;
  summary?: string;
  justification: string;
}

/** EXECUTE_ITEM: run one enabled plan item through the governed execution path. */
export interface ExecuteItemPayload {
  item_id: string;
}

/** COMPLETE_HUMAN_TASK: submit the result of an assigned human task; justification is mandatory. */
export interface CompleteHumanTaskPayload {
  item_id: string;
  result: Record<string, unknown>;
  justification: string;
}

/** REOPEN_CASE / TERMINATE_CASE: lifecycle transitions with a recorded reason. */
export interface CaseLifecyclePayload {
  case_id: string;
  reason: string;
}

/** Request payloads keyed by the consequential intent whose parameters carry them. */
export interface IntentRequestPayloads {
  PROPOSE_CASE: ProposeCasePayload;
  ADD_DISCRETIONARY_WORK: AddDiscretionaryWorkPayload;
  EXECUTE_ITEM: ExecuteItemPayload;
  COMPLETE_HUMAN_TASK: CompleteHumanTaskPayload;
  REOPEN_CASE: CaseLifecyclePayload;
  TERMINATE_CASE: CaseLifecyclePayload;
}

/** The action names that carry a typed payload in parameters. */
export type PayloadCarryingIntentName = keyof IntentRequestPayloads;

export interface ExecutionOpportunity {
  opportunity_id: string;
  case_id: string;
  plan_item_id: string;
  required_role: ActorRole;
  contract_slug: string;
  status: 'OPEN' | 'CLAIMED' | 'EXPIRED';
  created_at: string;
}

export interface ExecutionLease {
  lease_id: string;
  opportunity_id: string;
  worker_id: string;
  role: ActorRole;
  expires_at: string;
  nonce: string;
}

export interface AuthorizedInvocation {
  invocation_id: string;
  case_id: string;
  plan_item_id: string;
  lease_id: string;
  max_duration_seconds: number;
  sandbox_profile: string;
  permitted_capabilities: string[];
  signature: string;
}

export interface ExecutionObservation {
  run_id: string;
  invocation_id: string;
  exit_code: number;
  status: 'SUCCEEDED' | 'FAILED' | 'TIMED_OUT' | 'CANCELLED';
  duration_ms: number;
  artifacts: Array<{
    name: string;
    digest: string;
    size_bytes: number;
  }>;
  reality_trace?: {
    question_id: string;
    evidence_id: string;
    discrepancy_count: number;
    attribution_locus?: string | null;
  };
}

export interface OperationalSettlement {
  settlement_id: string;
  case_id: string;
  plan_item_id: string;
  invocation_id: string;
  decision: 'ACCEPTED' | 'REJECTED' | 'INCONCLUSIVE';
  evidence_id?: string;
  settled_at: string;
  consequence_summary: string;
}

export interface ArtifactProvenance {
  case_id: string;
  plan_item_id: string;
  invocation_id: string;
  run_id: string;
  question_id?: string;
  claim_id?: string;
  commit_sha?: string;
  pr_number?: number;
}

export interface ArtifactPayload {
  evidence_id: string;
  name: string;
  digest: string;
  content_type: 'text/markdown' | 'application/json' | 'text/x-diff' | 'text/plain';
  content: string;
  provenance: ArtifactProvenance;
}

export interface TemporalCheckpoint {
  cursor: string;
  timestamp: string;
  event_type: string;
  summary: string;
  actor_id: string;
  actor_role: string;
  consequential: boolean;
  active_stage_id?: string;
  completed_plan_items_count: number;
  total_plan_items_count: number;
}

export interface TemporalTrajectoryResponse {
  case_id: string;
  base_cursor: string;
  head_cursor: string;
  points: TemporalCheckpoint[];
}

export type StreamEventType =
  | 'snapshot'
  | 'patch'
  | 'execution_progress'
  | 'settlement_recorded'
  | 'lease_expired'
  | 'resync_required'
  | 'interrupted'
  | 'error'
  | 'heartbeat';

export interface StreamEvent<T = unknown> {
  event_type: StreamEventType;
  cursor: string;
  timestamp: string;
  payload: T;
}

export interface ExecutionProgressPayload {
  run_id: string;
  phase: 'orchestrator' | 'builder' | 'critic' | 'verifier' | 'settling';
  progress_percent: number;
  log_line: string;
}

export interface ResyncRequiredPayload {
  requested_cursor: string;
  oldest_available_cursor: string;
  reason: string;
}

/** StreamEvent payload for 'error': the stream itself failed (e.g. gap_exceeded). */
export interface ErrorPayload {
  error_code: string;
  message: string;
}

/** StreamEvent payload for 'interrupted': the connection signal between heartbeat and resync. */
export interface InterruptedPayload {
  reason: string;
  /** Last cursor the client can resume from with Last-Event-ID / ?last=. */
  last_cursor?: string;
}

// ---------------------------------------------------------------------------
// Template discovery and preflight (T01): gateway-facing DTOs for the case
// entry_options / preflight endpoints that back PROPOSE_CASE.

/** One declaratively typed parameter of a template. */
export interface TemplateParameter {
  name: string;
  title?: string;
  description?: string;
  type: 'string' | 'number' | 'boolean' | 'enum';
  required?: boolean;
  default_value?: unknown;
  /** Allowed values, exactly when type is 'enum'. */
  options?: string[];
}

/** One entry option as returned by case.entry_options / GET /api/templates. */
export interface TemplateEntryOption {
  template_ref: string;
  title: string;
  description?: string;
  parameters: TemplateParameter[];
}

/** Preflight verdict for a filled template: pass or fail, with reasons and the commit digest. */
export interface TemplatePreflightResult {
  template_ref: string;
  params: Record<string, unknown>;
  passed: boolean;
  /** Ordinary-language failure reasons; empty exactly when passed. */
  reasons: string[];
  /** Commit digest, present exactly when passed; PROPOSE_CASE must echo it. */
  digest?: string;
}
