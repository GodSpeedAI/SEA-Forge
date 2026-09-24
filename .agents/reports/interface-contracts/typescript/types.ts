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
  | 'APPROVE_HUMAN_TASK'
  | 'REJECT_HUMAN_TASK'
  | 'ADD_DISCRETIONARY_WORK'
  | 'REOPEN_WORK'
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
  | 'APPROVE_HUMAN_TASK'
  | 'REJECT_HUMAN_TASK'
  | 'ADD_DISCRETIONARY_WORK'
  | 'REOPEN_WORK'
  | 'ESCALATE_OR_OVERRIDE'
  | 'OPEN_ARTIFACT'
  | 'EXPORT_AUDIT_BUNDLE'
  | 'RESUME_LIVE_STREAM';

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

export interface IntentResponse {
  intent_id: string;
  success: boolean;
  new_cursor?: string;
  error_code?: string;
  error_message?: string;
  resulting_object?: CognitiveObject;
}

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
