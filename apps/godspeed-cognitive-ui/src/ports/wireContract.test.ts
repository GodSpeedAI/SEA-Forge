// T01: pin the canonical UI<->gateway wire contract (operator decision D-1) to the golden
// fixtures. The contract is the spec-04 CognitiveWorldSnapshot package at
// .agents/reports/interface-contracts/typescript/types.ts; the goldens live under
// .agents/reports/interface-contracts/golden/ and are byte-pinned by the Go gateway's mirror
// package apps/godspeed-casework-go/internal/contract. Together: a kind or field added, renamed
// or removed on one side only fails either `bun run typecheck` / `bun test` here or the Go
// contract tests there.
//
// The exhaustive-kind lists below are deliberately hardcoded: they must equal the unions in
// types.ts exactly (compile-time pins, checked by `bun run typecheck`) and must be exactly the
// kinds exercised by the golden fixtures (runtime checks below).

import { describe, test, expect } from 'bun:test'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import type {
  ActionIntentKind,
  ActionVariant,
  ActorRole,
  AttentionFocus,
  CognitiveObjectKind,
  CognitiveObjectStatus,
  ConsequentialIntentName,
  IntentRefusalKind,
  InteractionActionName,
  PayloadCarryingIntentName,
  RunExecutionStanding,
  RunSettlementStanding,
  RunTraceCommandExecutionStatus,
  RunTraceFrame,
  RunTraceFrameKind,
  RunTraceHydrationReadBudget,
  RunTraceObservation,
  RunTraceObservationEvent,
  RunTraceRunObservation,
  StreamEventType,
  ThothAnswerView,
  ThothAskRequest,
  ThothClaimClass,
  ThothClaimStatus,
  ThothClaimView,
  ThothDisposition,
  ThothFreshness,
  ThothQuestionKind,
  WorldSummary,
} from '../../../../.agents/reports/interface-contracts/typescript/types'
import type { NarrationBeat } from './contract'
import type * as PortContract from './contract'

import worldSnapshotJson from '../../../../.agents/reports/interface-contracts/golden/world-snapshot.json'
import intentProposeCase from '../../../../.agents/reports/interface-contracts/golden/intent-propose-case.json'
import intentAddDiscretionaryWork from '../../../../.agents/reports/interface-contracts/golden/intent-add-discretionary-work.json'
import intentExecuteItem from '../../../../.agents/reports/interface-contracts/golden/intent-execute-item.json'
import intentCompleteHumanTask from '../../../../.agents/reports/interface-contracts/golden/intent-complete-human-task.json'
import intentApproveHumanTask from '../../../../.agents/reports/interface-contracts/golden/intent-approve-human-task.json'
import intentRejectHumanTask from '../../../../.agents/reports/interface-contracts/golden/intent-reject-human-task.json'
import intentEscalateOrOverride from '../../../../.agents/reports/interface-contracts/golden/intent-escalate-or-override.json'
import intentOpenArtifact from '../../../../.agents/reports/interface-contracts/golden/intent-open-artifact.json'
import intentReopenCase from '../../../../.agents/reports/interface-contracts/golden/intent-reopen-case.json'
import intentTerminateCase from '../../../../.agents/reports/interface-contracts/golden/intent-terminate-case.json'
import templatesEntryOptions from '../../../../.agents/reports/interface-contracts/golden/templates-entry-options.json'
import templatePreflightPass from '../../../../.agents/reports/interface-contracts/golden/template-preflight-pass.json'
import templatePreflightFail from '../../../../.agents/reports/interface-contracts/golden/template-preflight-fail.json'
import thothAskRequest from '../../../../.agents/reports/interface-contracts/golden/ask-request.json'
import thothAnswerAnswered from '../../../../.agents/reports/interface-contracts/golden/ask-answer-answered.json'
import thothAnswerPartial from '../../../../.agents/reports/interface-contracts/golden/ask-answer-partial.json'
import thothAnswerDenied from '../../../../.agents/reports/interface-contracts/golden/ask-answer-denied.json'

const GOLDEN_DIR = join(import.meta.dir, '..', '..', '..', '..', '.agents', 'reports', 'interface-contracts', 'golden')

// ---------------------------------------------------------------------------
// Exhaustive vocabularies (hardcoded; pinned to the types.ts unions at compile time)

const ACTOR_ROLES = [
  'developer',
  'security_officer',
  'case_architect',
  'auditor',
  'customer_stakeholder',
  'agent_operator',
] as const

const OBJECT_KINDS = [
  'work_item',
  'stage',
  'milestone',
  'decision_gate',
  'discretionary_opportunity',
  'execution_trace',
  'evidence_record',
] as const

const OBJECT_STATUSES = [
  'WAITING',
  'READY_TO_BEGIN',
  'IN_PROGRESS',
  'ACTION_REQUIRED',
  'WAITING_ON_OTHERS',
  'COMPLETED',
  'REJECTED',
  'FAILED',
  'AVAILABLE_TO_ADD',
  'ACTIVE',
] as const

const ACTION_INTENT_KINDS = [
  'BEGIN_WORK',
  'PROPOSE_CASE',
  'APPROVE_HUMAN_TASK',
  'REJECT_HUMAN_TASK',
  'ADD_DISCRETIONARY_WORK',
  'EXECUTE_ITEM',
  'COMPLETE_HUMAN_TASK',
  'REOPEN_WORK',
  'REOPEN_CASE',
  'TERMINATE_CASE',
  'ESCALATE_OR_OVERRIDE',
  'OPEN_ARTIFACT',
  'RESOLVE_SOURCE',
  'EXPORT_AUDIT_BUNDLE',
  'RESUME_LIVE_STREAM',
] as const

const ACTION_VARIANTS = ['PRIMARY', 'SECONDARY', 'DANGER', 'WARNING', 'GHOST'] as const

const CONSEQUENTIAL_INTENT_KINDS = [
  'PROPOSE_CASE',
  'ADD_DISCRETIONARY_WORK',
  'EXECUTE_ITEM',
  'COMPLETE_HUMAN_TASK',
  'APPROVE_HUMAN_TASK',
  'REJECT_HUMAN_TASK',
  'ESCALATE_OR_OVERRIDE',
  'OPEN_ARTIFACT',
  'REOPEN_CASE',
  'TERMINATE_CASE',
] as const

const PAYLOAD_CARRYING_KINDS = [
  'PROPOSE_CASE',
  'ADD_DISCRETIONARY_WORK',
  'EXECUTE_ITEM',
  'COMPLETE_HUMAN_TASK',
  'REOPEN_CASE',
  'TERMINATE_CASE',
] as const

const REFUSAL_KINDS = [
  'AUTHORITY_DENIED',
  'UNAUTHORIZED_ROLE',
  'SOD_VIOLATION',
  'STALE_PROJECTION',
  'JUSTIFICATION_REQUIRED',
  'UNAVAILABLE',
  'INVALID',
] as const

const STREAM_EVENT_KINDS = [
  'snapshot',
  'patch',
  'execution_progress',
  'settlement_recorded',
  'lease_expired',
  'resync_required',
  'interrupted',
  'error',
  'heartbeat',
  'execution_observation',
] as const

const THOTH_QUESTION_KINDS = [
  'ask_capability',
  'ask_operation_requirements',
  'ask_authority_requirements',
  'ask_projection_support',
  'ask_environment_status',
  'ask_failure_explanation',
  'ask_evidence_for_claim',
  'ask_available_affordances',
  'ask_why_denied',
] as const

const THOTH_CLAIM_CLASSES = [
  'identity',
  'architecture',
  'declared_capability',
  'installed_capability',
  'demonstrated_capability',
  'authority_requirements',
  'environment_status',
  'failure_condition',
  'security_implementation',
  'customer_private',
  'credential_bearing',
  'policy_thresholds',
] as const

const THOTH_CLAIM_STATUSES = ['unknown', 'unsupported', 'declared', 'installed', 'available', 'validated', 'demonstrated'] as const
const THOTH_DISPOSITIONS = ['answered', 'partial', 'denied'] as const
const THOTH_FRESHNESS = ['current', 'stale'] as const

const RUN_TRACE_FRAME_KINDS = [
  'run_started',
  'item_activated',
  'command_started',
  'command_finished',
  'item_completed',
  'item_failed',
  'item_terminated',
  'human_task_completed',
  'run_halted',
  'run_finished',
] as const

const RUN_TRACE_COMMAND_STATUSES = ['completed', 'spawn_failed', 'timed_out', 'sandbox_violation', 'suspected_sandbox_violation'] as const
const RUN_EXECUTION_STANDINGS = ['pending', 'enabled', 'active', 'completed', 'failed', 'terminated'] as const
const RUN_SETTLEMENT_STANDINGS = ['unsettled', 'accepted', 'rejected', 'escalated'] as const
const RUN_OBSERVATION_STATES = ['complete', 'no_runs', 'capacity_limited', 'unavailable'] as const
const RUN_LIST_STATES = ['complete', 'unavailable'] as const
const RUN_TRACE_RUN_STATES = ['validated', 'unavailable'] as const

type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false
type Expect<T extends true> = T
type OptionalKeys<T> = { [K in keyof T]-?: {} extends Pick<T, K> ? K : never }[keyof T]

/**
 * Compile-time pins (run by `bun run typecheck`): every hardcoded list above must equal the
 * canonical union in types.ts exactly, and every consequential intent must also be a
 * wire-level InteractionActionName. A union member added or removed in types.ts alone, or in a
 * list alone, breaks this type and fails typecheck.
 */
export type WireContractPins = [
  Expect<Equal<(typeof ACTOR_ROLES)[number], ActorRole>>,
  Expect<Equal<(typeof OBJECT_KINDS)[number], CognitiveObjectKind>>,
  Expect<Equal<(typeof OBJECT_STATUSES)[number], CognitiveObjectStatus>>,
  Expect<Equal<(typeof ACTION_INTENT_KINDS)[number], ActionIntentKind>>,
  Expect<Equal<(typeof ACTION_VARIANTS)[number], ActionVariant>>,
  Expect<Equal<(typeof CONSEQUENTIAL_INTENT_KINDS)[number], ConsequentialIntentName>>,
  Expect<Equal<(typeof PAYLOAD_CARRYING_KINDS)[number], PayloadCarryingIntentName>>,
  Expect<Equal<(typeof REFUSAL_KINDS)[number], IntentRefusalKind>>,
  Expect<Equal<(typeof STREAM_EVENT_KINDS)[number], StreamEventType>>,
  Expect<Equal<Exclude<ConsequentialIntentName, InteractionActionName>, never>>,
  Expect<Equal<Exclude<ConsequentialIntentName, ActionIntentKind>, never>>,
]

/** Exact type/property pins for the T09 canonical Go/UI mirror additions. */
type CanonicalMirrorFieldTypes = [
  {
    kind: ThothQuestionKind
    subject: string
    purpose?: string
    case?: string
  },
  {
    claim_id: string
    claim_class: ThothClaimClass
    subject: string
    status: ThothClaimStatus
    statement: string
    snapshot_ref: string
    evidence_refs: string[]
    settlement_refs: string[]
    capability_record_ref?: string
  },
  {
    answer_id: string
    question_id: string
    disposition: ThothDisposition
    claims: ThothClaimView[]
    omitted_claim_classes: ThothClaimClass[]
    snapshot_ref: string
    freshness: ThothFreshness
    assurance: string
    limitations: string[]
    authority_notice: string
    answered_at: string
  },
  {
    event_type: 'execution_observation'
    cursor: string
    timestamp: string
    payload: RunTraceObservation
  },
  {
    case_id: string
    observed_at: string
    run_list_state: 'complete' | 'unavailable'
    observation_state: 'complete' | 'no_runs' | 'capacity_limited' | 'unavailable'
    listed_run_count?: number
    selected_run_count?: number
    validated_run_count?: number
    unreadable_run_count?: number
    unavailable_run_count?: number
    omitted_run_count?: number
    hydration_read_budget: RunTraceHydrationReadBudget
    runs: RunTraceRunObservation[]
  },
  {
    run_id: string
    observed_at: string
    plan_item_id: string
    execution: RunExecutionStanding
    settlement: RunSettlementStanding
    observation_state: 'validated' | 'unavailable'
    frames: RunTraceFrame[]
    total_frame_count: number
    retained_frame_count: number
    omitted_frame_count: number
    truncated: boolean
  },
  { limit: 8; reads_attempted: number; exhausted: boolean },
  {
    event_id: string
    kind: RunTraceFrameKind
    timestamp: string
    execution_status?: RunTraceCommandExecutionStatus
    exit_code?: number
  },
]

export type CanonicalMirrorPins = [
  Expect<Equal<(typeof THOTH_QUESTION_KINDS)[number], ThothQuestionKind>>,
  Expect<Equal<(typeof THOTH_CLAIM_CLASSES)[number], ThothClaimClass>>,
  Expect<Equal<(typeof THOTH_CLAIM_STATUSES)[number], ThothClaimStatus>>,
  Expect<Equal<(typeof THOTH_DISPOSITIONS)[number], ThothDisposition>>,
  Expect<Equal<(typeof THOTH_FRESHNESS)[number], ThothFreshness>>,
  Expect<Equal<(typeof RUN_TRACE_FRAME_KINDS)[number], RunTraceFrameKind>>,
  Expect<Equal<(typeof RUN_TRACE_COMMAND_STATUSES)[number], RunTraceCommandExecutionStatus>>,
  Expect<Equal<(typeof RUN_EXECUTION_STANDINGS)[number], RunExecutionStanding>>,
  Expect<Equal<(typeof RUN_SETTLEMENT_STANDINGS)[number], RunSettlementStanding>>,
  Expect<Equal<(typeof RUN_OBSERVATION_STATES)[number], RunTraceObservation['observation_state']>>,
  Expect<Equal<(typeof RUN_LIST_STATES)[number], RunTraceObservation['run_list_state']>>,
  Expect<Equal<(typeof RUN_TRACE_RUN_STATES)[number], RunTraceRunObservation['observation_state']>>,
  Expect<Equal<RunTraceHydrationReadBudget['limit'], 8>>,
  Expect<Equal<RunTraceObservationEvent['event_type'], 'execution_observation'>>,
  Expect<Equal<keyof RunTraceObservationEvent, 'event_type' | 'cursor' | 'timestamp' | 'payload'>>,
  Expect<Equal<ThothAskRequest['kind'], ThothQuestionKind>>,
  Expect<Equal<Pick<ThothAskRequest, 'purpose' | 'case'>, { purpose?: string; case?: string }>>,
  Expect<Equal<ThothClaimView['claim_class'], ThothClaimClass>>,
  Expect<Equal<ThothClaimView['status'], ThothClaimStatus>>,
  Expect<Equal<Pick<ThothClaimView, 'capability_record_ref'>, { capability_record_ref?: string }>>,
  Expect<Equal<ThothAnswerView['disposition'], ThothDisposition>>,
  Expect<Equal<ThothAnswerView['freshness'], ThothFreshness>>,
  Expect<Equal<RunTraceFrame['kind'], RunTraceFrameKind>>,
  Expect<Equal<RunTraceFrame['execution_status'], RunTraceCommandExecutionStatus | undefined>>,
  Expect<Equal<RunTraceFrame['exit_code'], number | undefined>>,
  Expect<Equal<RunTraceRunObservation['execution'], RunExecutionStanding>>,
  Expect<Equal<RunTraceRunObservation['settlement'], RunSettlementStanding>>,
  Expect<Equal<RunTraceRunObservation['observation_state'], 'validated' | 'unavailable'>>,
  Expect<Equal<RunTraceHydrationReadBudget['reads_attempted'], number>>,
  Expect<Equal<RunTraceHydrationReadBudget['exhausted'], boolean>>,
  Expect<Equal<RunTraceObservation['run_list_state'], 'complete' | 'unavailable'>>,
  Expect<Equal<RunTraceObservation['observation_state'], 'complete' | 'no_runs' | 'capacity_limited' | 'unavailable'>>,
  Expect<Equal<Pick<RunTraceObservation, 'listed_run_count' | 'selected_run_count' | 'validated_run_count' | 'unreadable_run_count' | 'unavailable_run_count' | 'omitted_run_count'>, { listed_run_count?: number; selected_run_count?: number; validated_run_count?: number; unreadable_run_count?: number; unavailable_run_count?: number; omitted_run_count?: number }>>,
  Expect<Equal<RunTraceObservationEvent['payload'], RunTraceObservation>>,
  Expect<Equal<keyof ThothAskRequest, 'kind' | 'subject' | 'purpose' | 'case'>>,
  Expect<Equal<OptionalKeys<ThothAskRequest>, 'purpose' | 'case'>>,
  Expect<Equal<keyof ThothClaimView, 'claim_id' | 'claim_class' | 'subject' | 'status' | 'statement' | 'snapshot_ref' | 'evidence_refs' | 'settlement_refs' | 'capability_record_ref'>>,
  Expect<Equal<OptionalKeys<ThothClaimView>, 'capability_record_ref'>>,
  Expect<Equal<keyof ThothAnswerView, 'answer_id' | 'question_id' | 'disposition' | 'claims' | 'omitted_claim_classes' | 'snapshot_ref' | 'freshness' | 'assurance' | 'limitations' | 'authority_notice' | 'answered_at'>>,
  Expect<Equal<OptionalKeys<ThothAnswerView>, never>>,
  Expect<Equal<keyof RunTraceFrame, 'event_id' | 'kind' | 'timestamp' | 'execution_status' | 'exit_code'>>,
  Expect<Equal<OptionalKeys<RunTraceFrame>, 'execution_status' | 'exit_code'>>,
  Expect<Equal<keyof RunTraceRunObservation, 'run_id' | 'observed_at' | 'plan_item_id' | 'execution' | 'settlement' | 'observation_state' | 'frames' | 'total_frame_count' | 'retained_frame_count' | 'omitted_frame_count' | 'truncated'>>,
  Expect<Equal<OptionalKeys<RunTraceRunObservation>, never>>,
  Expect<Equal<keyof RunTraceHydrationReadBudget, 'limit' | 'reads_attempted' | 'exhausted'>>,
  Expect<Equal<OptionalKeys<RunTraceHydrationReadBudget>, never>>,
  Expect<Equal<keyof RunTraceObservation, 'case_id' | 'observed_at' | 'run_list_state' | 'observation_state' | 'listed_run_count' | 'selected_run_count' | 'validated_run_count' | 'unreadable_run_count' | 'unavailable_run_count' | 'omitted_run_count' | 'hydration_read_budget' | 'runs'>>,
  Expect<Equal<OptionalKeys<RunTraceObservation>, 'listed_run_count' | 'selected_run_count' | 'validated_run_count' | 'unreadable_run_count' | 'unavailable_run_count' | 'omitted_run_count'>>,
  Expect<Equal<NonNullable<NarrationBeat['grounded_answer']>, ThothAnswerView>>,
  Expect<Equal<[
    ThothAskRequest,
    ThothClaimView,
    ThothAnswerView,
    RunTraceObservationEvent,
    RunTraceObservation,
    RunTraceRunObservation,
    RunTraceHydrationReadBudget,
    RunTraceFrame,
  ], CanonicalMirrorFieldTypes>>,
]

export type PortMirrorPins = [
  Expect<Equal<PortContract.RunExecutionStanding, RunExecutionStanding>>,
  Expect<Equal<PortContract.RunSettlementStanding, RunSettlementStanding>>,
  Expect<Equal<PortContract.RunTraceCommandExecutionStatus, RunTraceCommandExecutionStatus>>,
  Expect<Equal<PortContract.RunTraceFrame, RunTraceFrame>>,
  Expect<Equal<PortContract.RunTraceFrameKind, RunTraceFrameKind>>,
  Expect<Equal<PortContract.RunTraceHydrationReadBudget, RunTraceHydrationReadBudget>>,
  Expect<Equal<PortContract.RunTraceObservation, RunTraceObservation>>,
  Expect<Equal<PortContract.RunTraceObservationEvent, RunTraceObservationEvent>>,
  Expect<Equal<PortContract.RunTraceRunObservation, RunTraceRunObservation>>,
  Expect<Equal<PortContract.ThothAnswerView, ThothAnswerView>>,
  Expect<Equal<PortContract.ThothAskRequest, ThothAskRequest>>,
  Expect<Equal<PortContract.ThothClaimClass, ThothClaimClass>>,
  Expect<Equal<PortContract.ThothClaimStatus, ThothClaimStatus>>,
  Expect<Equal<PortContract.ThothClaimView, ThothClaimView>>,
  Expect<Equal<PortContract.ThothDisposition, ThothDisposition>>,
  Expect<Equal<PortContract.ThothFreshness, ThothFreshness>>,
  Expect<Equal<PortContract.ThothQuestionKind, ThothQuestionKind>>,
]

// ---------------------------------------------------------------------------
// Small runtime guards shared by the golden validators

const isStr = (v: unknown): v is string => typeof v === 'string'
const isNum = (v: unknown): v is number => typeof v === 'number'
const isBool = (v: unknown): v is boolean => typeof v === 'boolean'
const isSafeIntegerNumber = (v: unknown): v is number => typeof v === 'number' && Number.isSafeInteger(v)
const isNonnegativeSafeInteger = (v: unknown): v is number => isSafeIntegerNumber(v) && v >= 0
const isObj = (v: unknown): v is Record<string, unknown> =>
  typeof v === 'object' && v !== null && !Array.isArray(v)
const isStrArray = (v: unknown): v is string[] => Array.isArray(v) && v.every(isStr)

const CURSOR_RE = /^\d+\.\d+$/
const oneOf = (allowed: readonly string[]) => (v: unknown): v is string =>
  isStr(v) && (allowed as readonly string[]).includes(v)

const isRole = oneOf(ACTOR_ROLES)
const isObjectKind = oneOf(OBJECT_KINDS)
const isObjectStatus = oneOf(OBJECT_STATUSES)
const isActionIntent = oneOf(ACTION_INTENT_KINDS)
const isActionVariant = oneOf(ACTION_VARIANTS)
const isConsequentialIntent = oneOf(CONSEQUENTIAL_INTENT_KINDS)
const isPayloadCarrying = oneOf(PAYLOAD_CARRYING_KINDS)
const isRefusalKind = oneOf(REFUSAL_KINDS)
const isStreamEventKind = oneOf(STREAM_EVENT_KINDS)
const isThothQuestionKind = oneOf(THOTH_QUESTION_KINDS)
const isThothClaimClass = oneOf(THOTH_CLAIM_CLASSES)
const isThothClaimStatus = oneOf(THOTH_CLAIM_STATUSES)
const isThothDisposition = oneOf(THOTH_DISPOSITIONS)
const isThothFreshness = oneOf(THOTH_FRESHNESS)
const isRunTraceFrameKind = oneOf(RUN_TRACE_FRAME_KINDS)
const isRunTraceCommandStatus = oneOf(RUN_TRACE_COMMAND_STATUSES)
const isRunExecutionStanding = oneOf(RUN_EXECUTION_STANDINGS)
const isRunSettlementStanding = oneOf(RUN_SETTLEMENT_STANDINGS)
const isRunObservationState = oneOf(RUN_OBSERVATION_STATES)
const isRunListState = oneOf(RUN_LIST_STATES)
const isRunTraceRunState = oneOf(RUN_TRACE_RUN_STATES)

function hasOnlyKeys(v: Record<string, unknown>, required: readonly string[], optional: readonly string[] = []): boolean {
  const keys = Object.keys(v)
  return required.every((key) => key in v) && keys.every((key) => [...required, ...optional].includes(key))
}

function checkThothAskRequest(v: unknown, ctx: string): void {
  expect(isObj(v), `${ctx}: object`).toBe(true)
  if (!isObj(v)) return
  expect(hasOnlyKeys(v, ['kind', 'subject'], ['purpose', 'case']), `${ctx}: actor-free request fields`).toBe(true)
  expect(isThothQuestionKind(v.kind), `${ctx}.kind`).toBe(true)
  expect(isStr(v.subject) && v.subject.trim().length > 0, `${ctx}.subject`).toBe(true)
  if (v.purpose !== undefined) expect(isStr(v.purpose), `${ctx}.purpose`).toBe(true)
  if (v.case !== undefined) expect(isStr(v.case), `${ctx}.case`).toBe(true)
}

function checkThothAnswer(v: unknown, ctx: string): void {
  expect(isObj(v), `${ctx}: object`).toBe(true)
  if (!isObj(v)) return
  expect(
    hasOnlyKeys(v, ['answer_id', 'question_id', 'disposition', 'claims', 'omitted_claim_classes', 'snapshot_ref', 'freshness', 'assurance', 'limitations', 'authority_notice', 'answered_at']),
    `${ctx}: complete answer fields`,
  ).toBe(true)
  expect(isStr(v.answer_id) && isStr(v.question_id) && isStr(v.snapshot_ref) && isStr(v.answered_at), `${ctx}: identity and references`).toBe(true)
  expect(isThothDisposition(v.disposition), `${ctx}.disposition`).toBe(true)
  expect(isThothFreshness(v.freshness), `${ctx}.freshness`).toBe(true)
  expect(isStr(v.assurance) && isStr(v.authority_notice), `${ctx}: assurance and authority_notice are strings`).toBe(true)
  expect(Array.isArray(v.claims), `${ctx}.claims`).toBe(true)
  if (Array.isArray(v.claims)) {
    v.claims.forEach((claim, i) => {
      const claimCtx = `${ctx}.claims[${i}]`
      expect(isObj(claim), `${claimCtx}: object`).toBe(true)
      if (!isObj(claim)) return
      expect(
        hasOnlyKeys(claim, ['claim_id', 'claim_class', 'subject', 'status', 'statement', 'snapshot_ref', 'evidence_refs', 'settlement_refs'], ['capability_record_ref']),
        `${claimCtx}: complete claim fields`,
      ).toBe(true)
      expect(isStr(claim.claim_id) && isStr(claim.subject) && isStr(claim.statement) && isStr(claim.snapshot_ref), `${claimCtx}: disclosure strings`).toBe(true)
      expect(isThothClaimClass(claim.claim_class), `${claimCtx}.claim_class`).toBe(true)
      expect(isThothClaimStatus(claim.status), `${claimCtx}.status`).toBe(true)
      expect(isStrArray(claim.evidence_refs) && isStrArray(claim.settlement_refs), `${claimCtx}: references`).toBe(true)
      if (claim.capability_record_ref !== undefined) expect(isStr(claim.capability_record_ref), `${claimCtx}.capability_record_ref`).toBe(true)
    })
  }
  expect(Array.isArray(v.omitted_claim_classes) && v.omitted_claim_classes.every(isThothClaimClass), `${ctx}.omitted_claim_classes`).toBe(true)
  expect(isStrArray(v.limitations), `${ctx}.limitations`).toBe(true)
}

function checkRunTraceObservation(v: unknown, ctx: string): void {
  expect(isObj(v), `${ctx}: event object`).toBe(true)
  if (!isObj(v)) return
  expect(hasOnlyKeys(v, ['event_type', 'cursor', 'timestamp', 'payload']), `${ctx}: named event envelope`).toBe(true)
  expect(v.event_type, `${ctx}.event_type`).toBe('execution_observation')
  expect(isStr(v.cursor) && CURSOR_RE.test(v.cursor), `${ctx}.cursor is an informational real cursor`).toBe(true)
  expect(isStr(v.timestamp), `${ctx}.timestamp`).toBe(true)
  expect(isObj(v.payload), `${ctx}.payload`).toBe(true)
  if (!isObj(v.payload)) return
  const p = v.payload
  const countKeys = ['listed_run_count', 'selected_run_count', 'validated_run_count', 'unreadable_run_count', 'unavailable_run_count', 'omitted_run_count']
  expect(
    hasOnlyKeys(p, ['case_id', 'observed_at', 'run_list_state', 'observation_state', 'hydration_read_budget', 'runs'], countKeys),
    `${ctx}.payload: cohort fields`,
  ).toBe(true)
  expect(isStr(p.case_id) && isStr(p.observed_at), `${ctx}.payload identity`).toBe(true)
  expect(isRunListState(p.run_list_state), `${ctx}.payload.run_list_state`).toBe(true)
  expect(isRunObservationState(p.observation_state), `${ctx}.payload.observation_state`).toBe(true)
  for (const key of countKeys) {
    if (p[key] !== undefined) expect(isNonnegativeSafeInteger(p[key]), `${ctx}.payload.${key}`).toBe(true)
  }
  if (p.run_list_state === 'unavailable') {
    expect(countKeys.every((key) => p[key] === undefined), `${ctx}: unavailable list counts are absent`).toBe(true)
    expect(p.observation_state, `${ctx}: unavailable list observation state`).toBe('unavailable')
  } else {
    expect(countKeys.every((key) => p[key] !== undefined), `${ctx}: complete list reports counts including zero`).toBe(true)
  }
  expect(isObj(p.hydration_read_budget), `${ctx}.payload.hydration_read_budget`).toBe(true)
  if (isObj(p.hydration_read_budget)) {
    expect(
      hasOnlyKeys(p.hydration_read_budget, ['limit', 'reads_attempted', 'exhausted']),
      `${ctx}.payload.hydration_read_budget: exact fields`,
    ).toBe(true)
    expect(p.hydration_read_budget.limit, `${ctx}.payload.hydration_read_budget.limit`).toBe(8)
    const readsAttempted = p.hydration_read_budget.reads_attempted
    expect(isNonnegativeSafeInteger(readsAttempted) && readsAttempted <= 8, `${ctx}.payload.hydration_read_budget.reads_attempted`).toBe(true)
    expect(isBool(p.hydration_read_budget.exhausted), `${ctx}.payload.hydration_read_budget.exhausted`).toBe(true)
  }
  expect(Array.isArray(p.runs) && p.runs.length <= 8, `${ctx}.payload.runs (maximum eight)`).toBe(true)
  if (!Array.isArray(p.runs)) return
  p.runs.forEach((run: unknown, i) => {
    const runCtx = `${ctx}.payload.runs[${i}]`
    expect(isObj(run), `${runCtx}: object`).toBe(true)
    if (!isObj(run)) return
    expect(
      hasOnlyKeys(run, ['run_id', 'observed_at', 'plan_item_id', 'execution', 'settlement', 'observation_state', 'frames', 'total_frame_count', 'retained_frame_count', 'omitted_frame_count', 'truncated']),
      `${runCtx}: nested run fields`,
    ).toBe(true)
    expect(isStr(run.run_id) && isStr(run.observed_at) && isStr(run.plan_item_id), `${runCtx}: run and parent identity`).toBe(true)
    expect(isRunExecutionStanding(run.execution), `${runCtx}.execution`).toBe(true)
    expect(isRunSettlementStanding(run.settlement), `${runCtx}.settlement`).toBe(true)
    expect(isRunTraceRunState(run.observation_state), `${runCtx}.observation_state`).toBe(true)
    expect(Array.isArray(run.frames), `${runCtx}.frames`).toBe(true)
    if (!Array.isArray(run.frames)) return
    expect(run.frames.length <= 1024, `${runCtx}.frames (maximum 1024)`).toBe(true)
    const totalFrameCount = run.total_frame_count
    const retainedFrameCount = run.retained_frame_count
    const omittedFrameCount = run.omitted_frame_count
    const validFrameCounts = isNonnegativeSafeInteger(totalFrameCount) && isNonnegativeSafeInteger(retainedFrameCount) && isNonnegativeSafeInteger(omittedFrameCount)
    expect(validFrameCounts, `${runCtx}: frame counts are nonnegative integers`).toBe(true)
    if (validFrameCounts) {
      expect(totalFrameCount === retainedFrameCount + omittedFrameCount, `${runCtx}: frame count equation`).toBe(true)
      expect(retainedFrameCount === run.frames.length, `${runCtx}: retained count matches frames`).toBe(true)
      expect(isBool(run.truncated) && run.truncated === (omittedFrameCount > 0), `${runCtx}: truncation matches omitted count`).toBe(true)
    }
    run.frames.forEach((frame: unknown, j) => {
      const frameCtx = `${runCtx}.frames[${j}]`
      expect(isObj(frame), `${frameCtx}: object`).toBe(true)
      if (!isObj(frame)) return
      expect(hasOnlyKeys(frame, ['event_id', 'kind', 'timestamp'], ['execution_status', 'exit_code']), `${frameCtx}: safe frame fields`).toBe(true)
      expect(isStr(frame.event_id) && isStr(frame.timestamp), `${frameCtx}: event identity and time`).toBe(true)
      expect(isRunTraceFrameKind(frame.kind), `${frameCtx}.kind`).toBe(true)
      if (frame.execution_status !== undefined) expect(isRunTraceCommandStatus(frame.execution_status), `${frameCtx}.execution_status`).toBe(true)
      if (frame.exit_code !== undefined) expect(isNum(frame.exit_code), `${frameCtx}.exit_code`).toBe(true)
      if (frame.kind !== 'command_finished') {
        expect(frame.execution_status, `${frameCtx}: status only on command_finished`).toBeUndefined()
        expect(frame.exit_code, `${frameCtx}: exit code only on command_finished`).toBeUndefined()
      }
    })
  })
}

function checkActionDescriptor(v: unknown, ctx: string): void {
  expect(isObj(v), `${ctx}: object`).toBe(true)
  if (!isObj(v)) return
  expect(isStr(v.id) && v.id.length > 0, `${ctx}.id`).toBe(true)
  expect(isStr(v.label) && v.label.length > 0, `${ctx}.label`).toBe(true)
  expect(isActionIntent(v.intent), `${ctx}.intent (${String(v.intent)}) is an ActionIntentKind`).toBe(true)
  if (v.variant !== undefined) expect(isActionVariant(v.variant), `${ctx}.variant`).toBe(true)
  expect(isBool(v.consequential), `${ctx}.consequential`).toBe(true)
  if (v.requires_justification !== undefined)
    expect(isBool(v.requires_justification), `${ctx}.requires_justification`).toBe(true)
}

function checkCognitiveObject(v: unknown, ctx: string): void {
  expect(isObj(v), `${ctx}: object`).toBe(true)
  if (!isObj(v)) return
  expect(isStr(v.id) && v.id.length > 0, `${ctx}.id`).toBe(true)
  expect(isObjectKind(v.kind), `${ctx}.kind (${String(v.kind)}) is a CognitiveObjectKind`).toBe(true)
  expect(isStr(v.name), `${ctx}.name`).toBe(true)
  expect(isObjectStatus(v.status), `${ctx}.status (${String(v.status)}) is a CognitiveObjectStatus`).toBe(true)
  expect(isStr(v.badge), `${ctx}.badge`).toBe(true)
  expect(isNum(v.salience) && v.salience >= 0 && v.salience <= 1, `${ctx}.salience in [0,1]`).toBe(true)
  if (v.explanation !== undefined) expect(isStr(v.explanation), `${ctx}.explanation`).toBe(true)
  if (v.parent_id !== undefined) expect(isStr(v.parent_id), `${ctx}.parent_id`).toBe(true)
  if (v.depends_on !== undefined) expect(isStrArray(v.depends_on), `${ctx}.depends_on`).toBe(true)
  if (v.spatial_layout !== undefined) {
    const sp = v.spatial_layout
    expect(isObj(sp) && isNum(sp.x) && isNum(sp.y) && isNum(sp.z), `${ctx}.spatial_layout`).toBe(true)
    if (isObj(sp) && sp.radius !== undefined) expect(isNum(sp.radius), `${ctx}.spatial_layout.radius`).toBe(true)
  }
  expect(Array.isArray(v.actions), `${ctx}.actions array`).toBe(true)
  if (Array.isArray(v.actions)) v.actions.forEach((a, i) => checkActionDescriptor(a, `${ctx}.actions[${i}]`))
}

function checkSnapshot(v: unknown, ctx: string): void {
  expect(isObj(v), `${ctx}: object`).toBe(true)
  if (!isObj(v)) return
  expect(isStr(v.world_id), `${ctx}.world_id`).toBe(true)
  expect(isStr(v.case_id), `${ctx}.case_id`).toBe(true)
  expect(isStr(v.cursor) && CURSOR_RE.test(v.cursor), `${ctx}.cursor is an epoch.seq kernel cursor`).toBe(true)
  expect(isStr(v.timestamp), `${ctx}.timestamp`).toBe(true)
  expect(isObj(v.perspective) && isStr(v.perspective.actor_id) && isRole(v.perspective.role), `${ctx}.perspective`).toBe(true)
  expect(
    isObj(v.summary) && isStr(v.summary.headline) && isStr(v.summary.phase) && isStr(v.summary.status_phrase),
    `${ctx}.summary`,
  ).toBe(true)
  if (isObj(v.summary) && v.summary.progress_percent !== undefined)
    expect(isNum(v.summary.progress_percent), `${ctx}.summary.progress_percent`).toBe(true)
  expect(Array.isArray(v.visible_objects) && v.visible_objects.length > 0, `${ctx}.visible_objects`).toBe(true)
  if (Array.isArray(v.visible_objects))
    v.visible_objects.forEach((o, i) => checkCognitiveObject(o, `${ctx}.visible_objects[${i}]`))
  if (v.available_actions !== undefined)
    (v.available_actions as unknown[]).forEach((a, i) => checkActionDescriptor(a, `${ctx}.available_actions[${i}]`))
  expect(isObj(v.attention_focus) && isStr(v.attention_focus.primary_object_id), `${ctx}.attention_focus`).toBe(true)
}

function checkIntentFixture(name: string, fx: unknown): void {
  const ctx = `intent fixture ${name}`
  expect(isObj(fx) && isObj(fx.request) && Array.isArray(fx.responses), `${ctx}: shape`).toBe(true)
  if (!isObj(fx) || !isObj(fx.request)) return
  const req = fx.request
  expect(isStr(req.intent_id) && req.intent_id.length > 0, `${ctx}.request.intent_id`).toBe(true)
  expect(req.kind, `${ctx}.request.kind is CONSEQUENTIAL_CASE`).toBe('CONSEQUENTIAL_CASE')
  expect(isConsequentialIntent(req.action_name), `${ctx}.request.action_name is a consequential intent`).toBe(true)
  expect(isStr(req.target_object_id), `${ctx}.request.target_object_id`).toBe(true)
  expect(isStr(req.case_id), `${ctx}.request.case_id`).toBe(true)
  expect(isStr(req.client_cursor) && CURSOR_RE.test(req.client_cursor), `${ctx}.request.client_cursor`).toBe(true)
  expect(isObj(req.actor) && isStr(req.actor.actor_id) && isRole(req.actor.role), `${ctx}.request.actor`).toBe(true)
  if (req.justification !== undefined) expect(isStr(req.justification), `${ctx}.request.justification`).toBe(true)

  if (isPayloadCarrying(req.action_name)) {
    expect(isObj(req.parameters), `${ctx}.parameters carries the typed payload`).toBe(true)
    if (isObj(req.parameters)) checkTypedPayload(req.action_name, req.parameters, ctx)
  } else {
    expect(req.parameters, `${ctx}: non-payload intents carry no parameters`).toBeUndefined()
  }

  const responses = fx.responses as unknown[]
  expect(responses.length > 0, `${ctx}: at least one response`).toBe(true)
  responses.forEach((r, i) => {
    const rctx = `${ctx}.responses[${i}]`
    expect(isObj(r) && isStr(r.intent_id) && r.intent_id === req.intent_id, `${rctx}.intent_id echoes request`).toBe(true)
    expect(isObj(r) && isBool(r.success), `${rctx}.success`).toBe(true)
    if (!isObj(r)) return
    if (r.success === true) {
      expect(r.refusal, `${rctx}: success must not carry refusal`).toBeUndefined()
      if (r.new_cursor !== undefined)
        expect(isStr(r.new_cursor) && CURSOR_RE.test(r.new_cursor), `${rctx}.new_cursor`).toBe(true)
      return
    }
    expect(isObj(r.refusal), `${rctx}: refusal carries the typed envelope`).toBe(true)
    if (isObj(r.refusal)) {
      expect(isRefusalKind(r.refusal.refusal_kind), `${rctx}.refusal.refusal_kind (${String(r.refusal.refusal_kind)})`).toBe(true)
      expect(isStr(r.refusal.message) && r.refusal.message.length > 0, `${rctx}.refusal.message`).toBe(true)
      if (r.refusal.current_cursor !== undefined)
        expect(isStr(r.refusal.current_cursor), `${rctx}.refusal.current_cursor`).toBe(true)
    }
    expect(r.new_cursor, `${rctx}: a refusal must not carry new_cursor`).toBeUndefined()
  })
}

function checkTypedPayload(kind: string, p: Record<string, unknown>, ctx: string): void {
  const strField = (key: string): void => {
    expect(isStr(p[key]) && p[key].length > 0, `${ctx}.parameters.${key}`).toBe(true)
  }
  switch (kind) {
    case 'PROPOSE_CASE':
      strField('template_ref')
      expect(isObj(p.params), `${ctx}.parameters.params`).toBe(true)
      strField('preflight_digest')
      break
    case 'ADD_DISCRETIONARY_WORK':
      strField('case_id')
      strField('stage_id')
      if (p.anchor_item_id !== undefined) expect(isStr(p.anchor_item_id), `${ctx}.parameters.anchor_item_id`).toBe(true)
      expect(isObjectKind(p.kind), `${ctx}.parameters.kind`).toBe(true)
      strField('title')
      if (p.summary !== undefined) expect(isStr(p.summary), `${ctx}.parameters.summary`).toBe(true)
      strField('justification')
      break
    case 'EXECUTE_ITEM':
      strField('item_id')
      break
    case 'COMPLETE_HUMAN_TASK':
      strField('item_id')
      expect(isObj(p.result), `${ctx}.parameters.result`).toBe(true)
      strField('justification')
      break
    case 'REOPEN_CASE':
    case 'TERMINATE_CASE':
      strField('case_id')
      strField('reason')
      break
  }
}

// ---------------------------------------------------------------------------
// Suites

describe('canonical wire contract: exhaustive kind lists', () => {
  test('the hardcoded lists have no duplicates', () => {
    const lists: ReadonlyArray<readonly string[]> = [
      ACTOR_ROLES,
      OBJECT_KINDS,
      OBJECT_STATUSES,
      ACTION_INTENT_KINDS,
      ACTION_VARIANTS,
      CONSEQUENTIAL_INTENT_KINDS,
      PAYLOAD_CARRYING_KINDS,
      REFUSAL_KINDS,
      STREAM_EVENT_KINDS,
      THOTH_QUESTION_KINDS,
      THOTH_CLAIM_CLASSES,
      THOTH_CLAIM_STATUSES,
      THOTH_DISPOSITIONS,
      THOTH_FRESHNESS,
      RUN_TRACE_FRAME_KINDS,
      RUN_TRACE_COMMAND_STATUSES,
      RUN_EXECUTION_STANDINGS,
      RUN_SETTLEMENT_STANDINGS,
      RUN_OBSERVATION_STATES,
      RUN_LIST_STATES,
      RUN_TRACE_RUN_STATES,
    ]
    for (const list of lists) expect(new Set(list).size, [...list].join(',')).toBe(list.length)
  })

  test('the compile-time pins are active (WireContractPins resolves to all-true)', () => {
    const pins: WireContractPins = [
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
    ]
    expect(pins.every((p) => p === true)).toBe(true)
  })

  test('the T09 mirror pins cover canonical fields, optionality, and finite vocabularies', () => {
    const pins: CanonicalMirrorPins = [
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
    ]
    expect(pins.every((p) => p === true)).toBe(true)
  })

  test('the UI port re-exports the canonical mirror DTOs without widening them', () => {
    const pins: PortMirrorPins = [
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
      true,
    ]
    expect(pins.every((p) => p === true)).toBe(true)
  })
})

describe('golden world snapshot', () => {
  test('world-snapshot.json matches the canonical CognitiveWorldSnapshot shape', () => {
    checkSnapshot(worldSnapshotJson, 'world-snapshot.json')
  })

  test('union-free golden fields typecheck directly against the canonical types', () => {
    // Compile-time: these assignments fail if types.ts renames or retypes the fields.
    const summary: WorldSummary = worldSnapshotJson.summary
    const focus: AttentionFocus = worldSnapshotJson.attention_focus
    expect(isStr(summary.headline)).toBe(true)
    expect(isStr(focus.primary_object_id)).toBe(true)
  })
})

describe('golden intent fixtures', () => {
  const INTENT_FIXTURES: Record<string, unknown> = {
    PROPOSE_CASE: intentProposeCase,
    ADD_DISCRETIONARY_WORK: intentAddDiscretionaryWork,
    EXECUTE_ITEM: intentExecuteItem,
    COMPLETE_HUMAN_TASK: intentCompleteHumanTask,
    APPROVE_HUMAN_TASK: intentApproveHumanTask,
    REJECT_HUMAN_TASK: intentRejectHumanTask,
    ESCALATE_OR_OVERRIDE: intentEscalateOrOverride,
    OPEN_ARTIFACT: intentOpenArtifact,
    REOPEN_CASE: intentReopenCase,
    TERMINATE_CASE: intentTerminateCase,
  }

  test('every consequential intent kind has exactly one well-formed golden fixture', () => {
    expect([...new Set(Object.keys(INTENT_FIXTURES))].sort()).toEqual([...CONSEQUENTIAL_INTENT_KINDS].sort())
    for (const [kind, fx] of Object.entries(INTENT_FIXTURES)) {
      checkIntentFixture(kind, fx)
      const actionName = (fx as { request: { action_name: string } }).request.action_name
      expect(actionName, `fixture for ${kind} dispatches its own kind`).toBe(kind)
    }
  })

  test('every refusal kind is exercised at least once across the goldens', () => {
    const seen = new Set<string>()
    for (const fx of Object.values(INTENT_FIXTURES)) {
      const responses = (fx as { responses: Array<{ refusal?: { refusal_kind: string } }> }).responses ?? []
      for (const r of responses) if (r.refusal) seen.add(r.refusal.refusal_kind)
    }
    expect([...seen].sort()).toEqual([...REFUSAL_KINDS].sort())
  })
})

describe('golden template and preflight fixtures', () => {
  test('templates-entry-options.json matches the canonical template shapes', () => {
    expect(Array.isArray(templatesEntryOptions) && templatesEntryOptions.length > 0).toBe(true)
    if (!Array.isArray(templatesEntryOptions)) return
    for (const tpl of templatesEntryOptions as unknown[]) {
      expect(isObj(tpl) && isStr(tpl.template_ref) && isStr(tpl.title), 'template_ref and title').toBe(true)
      if (!isObj(tpl)) continue
      if (tpl.description !== undefined) expect(isStr(tpl.description), 'description').toBe(true)
      expect(Array.isArray(tpl.parameters) && tpl.parameters.length > 0, 'parameters').toBe(true)
      for (const param of (tpl.parameters ?? []) as unknown[]) {
        expect(isObj(param) && isStr(param.name), 'parameter.name').toBe(true)
        if (!isObj(param)) continue
        expect(
          ['string', 'number', 'boolean', 'enum'].includes(param.type as string),
          `parameter ${String(param.name)} type`,
        ).toBe(true)
        if (param.options !== undefined) expect(isStrArray(param.options), 'parameter.options').toBe(true)
      }
    }
  })

  test('template-preflight-pass.json passes with a digest and no reasons', () => {
    expect(isObj(templatePreflightPass)).toBe(true)
    if (!isObj(templatePreflightPass)) return
    expect(templatePreflightPass.passed).toBe(true)
    expect(templatePreflightPass.reasons).toEqual([])
    expect(isStr(templatePreflightPass.digest) && templatePreflightPass.digest.length > 0, 'digest').toBe(true)
    expect(isStr(templatePreflightPass.template_ref)).toBe(true)
    expect(isObj(templatePreflightPass.params)).toBe(true)
  })

  test('template-preflight-fail.json fails with reasons and no digest', () => {
    expect(isObj(templatePreflightFail)).toBe(true)
    if (!isObj(templatePreflightFail)) return
    expect(templatePreflightFail.passed).toBe(false)
    expect(isStrArray(templatePreflightFail.reasons) && templatePreflightFail.reasons.length > 0, 'reasons').toBe(true)
    const failRecord: Record<string, unknown> = templatePreflightFail
    expect(failRecord.digest, 'no digest on a failed preflight').toBeUndefined()
  })
})

describe('golden Thoth Ask fixtures', () => {
  test('ask-request.json is an actor-free request with the exact finite question kind', () => {
    checkThothAskRequest(thothAskRequest, 'ask-request.json')
  })

  test('answered, partial, and denied fixtures preserve complete disclosure fields', () => {
    const answers: Record<string, unknown> = {
      answered: thothAnswerAnswered,
      partial: thothAnswerPartial,
      denied: thothAnswerDenied,
    }
    expect(Object.keys(answers).sort()).toEqual([...THOTH_DISPOSITIONS].sort())
    for (const [disposition, answer] of Object.entries(answers)) {
      checkThothAnswer(answer, `ask-answer-${disposition}.json`)
      expect(isObj(answer) && answer.disposition, `fixture ${disposition} keeps its governed disposition`).toBe(disposition)
    }
    const answeredClaim = (thothAnswerAnswered as { claims: Array<Record<string, unknown>> }).claims[0]
    const partialClaim = (thothAnswerPartial as { claims: Array<Record<string, unknown>> }).claims[0]
    expect(isStr(answeredClaim.capability_record_ref), 'answered fixture preserves capability_record_ref').toBe(true)
    expect(partialClaim.capability_record_ref, 'optional capability_record_ref stays absent when not returned').toBeUndefined()
  })
})

describe('golden SSE event fixtures', () => {
  test('sse-events.jsonl carries exactly one well-formed line per stream event kind', () => {
    const raw = readFileSync(join(GOLDEN_DIR, 'sse-events.jsonl'), 'utf8')
    const lines = raw.trimEnd().split('\n')
    expect(lines.length, 'one line per StreamEventType').toBe(STREAM_EVENT_KINDS.length)
    const seen: string[] = []
    lines.forEach((line, i) => {
      const ev = JSON.parse(line) as Record<string, unknown>
      expect(isStreamEventKind(ev.event_type), `line ${i + 1} event_type (${String(ev.event_type)})`).toBe(true)
      expect(isStr(ev.cursor) && CURSOR_RE.test(ev.cursor as string), `line ${i + 1} cursor`).toBe(true)
      expect(isStr(ev.timestamp), `line ${i + 1} timestamp`).toBe(true)
      expect(isObj(ev.payload), `line ${i + 1} payload`).toBe(true)
      if (ev.event_type === 'execution_observation') checkRunTraceObservation(ev, `sse-events.jsonl line ${i + 1}`)
      seen.push(ev.event_type as string)
    })
    expect([...new Set(seen)].sort()).toEqual([...STREAM_EVENT_KINDS].sort())
  })

  test('rejects unknown hydration read budget fields', () => {
    const lines = readFileSync(join(GOLDEN_DIR, 'sse-events.jsonl'), 'utf8').trimEnd().split('\n')
    const event = lines
      .map((line) => JSON.parse(line) as Record<string, unknown>)
      .find((candidate) => candidate.event_type === 'execution_observation')
    expect(event).toBeDefined()
    if (!event) return

    const withUnknownBudgetField = structuredClone(event)
    if (!isObj(withUnknownBudgetField.payload) || !isObj(withUnknownBudgetField.payload.hydration_read_budget)) {
      throw new TypeError('execution observation golden must contain an object hydration budget')
    }
    withUnknownBudgetField.payload.hydration_read_budget.extra = true
    expect(() => checkRunTraceObservation(withUnknownBudgetField, 'unknown hydration budget field')).toThrow()
  })
})
