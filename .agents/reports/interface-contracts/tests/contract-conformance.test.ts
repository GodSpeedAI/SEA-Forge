import { describe, expect, it } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { createCaseworkClient, MockCaseworkAdapter } from '../typescript';
import type {
  InteractionActionName,
  InteractionIntent,
  RunTraceFrame,
  RunTraceObservation,
  RunTraceObservationEvent,
  RunTraceRunObservation,
  StreamEventType,
  ThothAnswerView,
  ThothAskRequest,
  ThothClaimClass,
  ThothClaimStatus,
  ThothDisposition,
  ThothFreshness,
  ThothQuestionKind,
} from '../typescript/types';

const GOLDEN_DIR = join(import.meta.dir, '../golden');
const SCHEMA_DIR = join(import.meta.dir, '../schemas');

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
] as const;

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
] as const;

const THOTH_CLAIM_STATUSES = [
  'unknown',
  'unsupported',
  'declared',
  'installed',
  'available',
  'validated',
  'demonstrated',
] as const;

const THOTH_DISPOSITIONS = ['answered', 'partial', 'denied'] as const;
const THOTH_FRESHNESS = ['current', 'stale'] as const;

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
] as const;

const SAFE_TRACE_KINDS = [
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
] as const;

const RUN_EXECUTION_STANDINGS = ['pending', 'enabled', 'active', 'completed', 'failed', 'terminated'] as const;
const COMMAND_EXECUTION_STATUSES = [
  'completed', 'spawn_failed', 'timed_out', 'sandbox_violation', 'suspected_sandbox_violation',
] as const;

const ANSWER_KEYS = [
  'answer_id',
  'question_id',
  'disposition',
  'claims',
  'omitted_claim_classes',
  'snapshot_ref',
  'freshness',
  'assurance',
  'limitations',
  'authority_notice',
  'answered_at',
] as const;

const CLAIM_KEYS = [
  'claim_id',
  'claim_class',
  'subject',
  'status',
  'statement',
  'snapshot_ref',
  'evidence_refs',
  'settlement_refs',
] as const;
const ANSWER_SAMPLE_FILES = [
  'ask-answer-answered.json',
  'ask-answer-partial.json',
  'ask-answer-denied.json',
] as const;

type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
type Expect<T extends true> = T;
type OptionalKeys<T> = { [K in keyof T]-?: {} extends Pick<T, K> ? K : never }[keyof T];

const CONTRACT_SCHEMA_FILES = [
  'cognitive-world.schema.json',
  'interaction-intents.schema.json',
  'case-operations.schema.json',
  'execution-contracts.schema.json',
  'realitytrace-evidence.schema.json',
  'event-stream.schema.json',
  'thoth-ask.schema.json',
] as const;

export type ThothContractPins = [
  Expect<Equal<ThothQuestionKind, (typeof THOTH_QUESTION_KINDS)[number]>>,
  Expect<Equal<ThothClaimClass, (typeof THOTH_CLAIM_CLASSES)[number]>>,
  Expect<Equal<ThothClaimStatus, (typeof THOTH_CLAIM_STATUSES)[number]>>,
  Expect<Equal<ThothDisposition, (typeof THOTH_DISPOSITIONS)[number]>>,
  Expect<Equal<ThothFreshness, (typeof THOTH_FRESHNESS)[number]>>,
  Expect<Equal<keyof ThothAskRequest, 'kind' | 'subject' | 'purpose' | 'case'>>,
  Expect<Equal<keyof ThothAnswerView, (typeof ANSWER_KEYS)[number]>>,
  Expect<Equal<keyof NonNullable<ThothAnswerView['claims']>[number], (typeof CLAIM_KEYS)[number] | 'capability_record_ref'>>,
  Expect<Equal<StreamEventType, (typeof STREAM_EVENT_KINDS)[number]>>,
  Expect<Equal<RunTraceObservationEvent['event_type'], 'execution_observation'>>,
  Expect<Equal<RunTraceFrame['kind'], (typeof SAFE_TRACE_KINDS)[number]>>,
  Expect<Equal<NonNullable<RunTraceFrame['execution_status']>, (typeof COMMAND_EXECUTION_STATUSES)[number]>>,
  Expect<Equal<keyof RunTraceRunObservation,
    'run_id' | 'observed_at' | 'plan_item_id' | 'execution' | 'settlement' | 'observation_state' |
    'frames' | 'total_frame_count' | 'retained_frame_count' | 'omitted_frame_count' | 'truncated'>>,
  Expect<Equal<keyof RunTraceObservation,
    'case_id' | 'observed_at' | 'run_list_state' | 'observation_state' | 'listed_run_count' |
    'selected_run_count' | 'validated_run_count' | 'unreadable_run_count' | 'unavailable_run_count' |
    'omitted_run_count' | 'hydration_read_budget' | 'runs'>>,
  Expect<Equal<RunTraceObservation['run_list_state'], 'complete' | 'unavailable'>>,
  Expect<Equal<RunTraceObservation['observation_state'], 'complete' | 'no_runs' | 'capacity_limited' | 'unavailable'>>,
  Expect<Equal<RunTraceRunObservation['observation_state'], 'validated' | 'unavailable'>>,
  Expect<Equal<OptionalKeys<RunTraceObservation>,
    'listed_run_count' | 'selected_run_count' | 'validated_run_count' | 'unreadable_run_count' |
    'unavailable_run_count' | 'omitted_run_count'>>,
];

const WIRE_ACTION_NAMES: readonly InteractionActionName[] = [
  'SELECT_OBJECT',
  'NAVIGATE_CAMERA',
  'FILTER_VIEW',
  'RESOLVE_SOURCE',
  'REQUEST_DEEP_PROJECTION',
  'REQUEST_HISTORY_SCRUB',
  'BEGIN_WORK',
  'APPROVE_HUMAN_TASK',
  'REJECT_HUMAN_TASK',
  'ADD_DISCRETIONARY_WORK',
  'REOPEN_WORK',
  'ESCALATE_OR_OVERRIDE',
  'OPEN_ARTIFACT',
  'EXPORT_AUDIT_BUNDLE',
  'RESUME_LIVE_STREAM',
];

function readGoldenJson(file: string): Record<string, unknown> {
  return JSON.parse(readFileSync(join(GOLDEN_DIR, file), 'utf-8')) as Record<string, unknown>;
}

function expectExactKeys(value: Record<string, unknown>, keys: readonly string[]): void {
  expect(Object.keys(value).sort()).toEqual([...keys].sort());
}

function expectContainsUnknown(values: readonly unknown[], value: unknown): void {
  expect(values).toContain(value);
}

function expectThothAnswerShape(answer: Record<string, unknown>): void {
  expectExactKeys(answer, ANSWER_KEYS);
  expect(typeof answer.answer_id).toBe('string');
  expect(typeof answer.question_id).toBe('string');
  expectContainsUnknown(THOTH_DISPOSITIONS, answer.disposition);
  expect(Array.isArray(answer.claims)).toBe(true);
  expect(Array.isArray(answer.omitted_claim_classes)).toBe(true);
  if (!Array.isArray(answer.omitted_claim_classes)) throw new TypeError('omitted_claim_classes must be an array');
  for (const claimClass of answer.omitted_claim_classes) {
    expectContainsUnknown(THOTH_CLAIM_CLASSES, claimClass);
  }
  expect(typeof answer.snapshot_ref).toBe('string');
  expectContainsUnknown(THOTH_FRESHNESS, answer.freshness);
  expect(typeof answer.assurance).toBe('string');
  expect(Array.isArray(answer.limitations)).toBe(true);
  expect(answer.authority_notice).toBe('This answer confers no execution authority.');
  expect(typeof answer.answered_at).toBe('string');

  for (const claim of answer.claims as Array<Record<string, unknown>>) {
    expect(Object.keys(claim).sort()).toEqual(
      [...CLAIM_KEYS, ...(Object.hasOwn(claim, 'capability_record_ref') ? ['capability_record_ref'] : [])].sort()
    );
    expectContainsUnknown(THOTH_CLAIM_CLASSES, claim.claim_class);
    expectContainsUnknown(THOTH_CLAIM_STATUSES, claim.status);
    expect(typeof claim.claim_id).toBe('string');
    expect(typeof claim.subject).toBe('string');
    expect(typeof claim.statement).toBe('string');
    expect(typeof claim.snapshot_ref).toBe('string');
    expect(Array.isArray(claim.evidence_refs)).toBe(true);
    expect(Array.isArray(claim.settlement_refs)).toBe(true);
    if (Object.hasOwn(claim, 'capability_record_ref')) expect(typeof claim.capability_record_ref).toBe('string');
  }
}

function expectRunTraceObservationShape(observation: Record<string, unknown>): void {
  const countKeys = [
    'listed_run_count', 'selected_run_count', 'validated_run_count', 'unreadable_run_count',
    'unavailable_run_count', 'omitted_run_count',
  ];
  expectExactKeys(observation, [
    'case_id', 'observed_at', 'run_list_state', 'observation_state',
    ...(observation.run_list_state === 'complete' ? countKeys : []), 'hydration_read_budget', 'runs',
  ]);
  expect(typeof observation.case_id).toBe('string');
  expect(typeof observation.observed_at).toBe('string');
  expectContainsUnknown(['complete', 'unavailable'], observation.run_list_state);
  expectContainsUnknown(['complete', 'no_runs', 'capacity_limited', 'unavailable'], observation.observation_state);
  if (observation.run_list_state === 'unavailable') {
    for (const key of countKeys) expect(Object.hasOwn(observation, key)).toBe(false);
  } else {
    for (const key of countKeys) {
      expect(Number.isInteger(observation[key])).toBe(true);
      expect(observation[key] as number).toBeGreaterThanOrEqual(0);
    }
  }
  expect(observation.hydration_read_budget).toEqual({
    limit: 8,
    reads_attempted: expect.any(Number),
    exhausted: expect.any(Boolean),
  });
  const runs = observation.runs as Array<Record<string, unknown>>;
  expect(runs.length).toBeLessThanOrEqual(8);
  for (const run of runs) {
    expectExactKeys(run, [
      'run_id', 'observed_at', 'plan_item_id', 'execution', 'settlement', 'observation_state',
      'frames', 'total_frame_count', 'retained_frame_count', 'omitted_frame_count', 'truncated',
    ]);
    expect(typeof run.run_id).toBe('string');
    expect(typeof run.plan_item_id).toBe('string');
    expectContainsUnknown(RUN_EXECUTION_STANDINGS, run.execution);
    expectContainsUnknown(['unsettled', 'accepted', 'rejected', 'escalated'], run.settlement);
    expectContainsUnknown(['validated', 'unavailable'], run.observation_state);
    const frames = run.frames as Array<Record<string, unknown>>;
    expect(frames.length).toBeLessThanOrEqual(1024);
    expect(run.retained_frame_count).toBe(frames.length);
    const omittedFrameCount = run.omitted_frame_count;
    expect(Number.isInteger(omittedFrameCount)).toBe(true);
    if (typeof omittedFrameCount !== 'number') throw new TypeError('omitted_frame_count must be a number');
    expect((run.total_frame_count as number) - (run.retained_frame_count as number)).toBe(omittedFrameCount);
    expect(run.truncated).toBe((run.omitted_frame_count as number) > 0);
    for (const frame of frames) {
      const keys = Object.keys(frame).sort();
      expect(keys).toEqual(
        [...['event_id', 'kind', 'timestamp'],
          ...(frame.kind === 'command_finished' && Object.hasOwn(frame, 'execution_status') ? ['execution_status'] : []),
          ...(frame.kind === 'command_finished' && Object.hasOwn(frame, 'exit_code') ? ['exit_code'] : [])].sort()
      );
      expect(typeof frame.event_id).toBe('string');
      expectContainsUnknown(SAFE_TRACE_KINDS, frame.kind);
      expect(typeof frame.timestamp).toBe('string');
      if (frame.kind !== 'command_finished') {
        expect(Object.hasOwn(frame, 'execution_status')).toBe(false);
        expect(Object.hasOwn(frame, 'exit_code')).toBe(false);
      }
      if (Object.hasOwn(frame, 'execution_status')) {
        expectContainsUnknown(COMMAND_EXECUTION_STATUSES, frame.execution_status);
      }
      if (Object.hasOwn(frame, 'exit_code')) expect(Number.isInteger(frame.exit_code)).toBe(true);
    }
  }
}

describe('GodSpeed Casework Application Boundary & Contract Conformance', () => {
  const adapter = new MockCaseworkAdapter();
  const client = createCaseworkClient(adapter);
  const caseId = 'case-auth-v2-001';

  // These focused checks cover the required wire fields without adding a JSON
  // Schema dependency. Full draft-2020-12 validation remains an integration
  // concern for a consumer that already provides a validator.
  function expectWorldWireShape(world: Record<string, unknown>): void {
    expect(world.world_id).toMatch(/^ws-[a-z0-9_.-]+-[0-9]+\.[0-9]{10}$/);
    expect(world.case_id).toBe(caseId);
    expect(world.cursor).toMatch(/^[0-9]+\.[0-9]{10}$/);
    expect(typeof world.timestamp).toBe('string');
    expect(world.perspective).toMatchObject({ actor_id: expect.any(String), role: expect.any(String) });
    expect(world.summary).toMatchObject({
      headline: expect.any(String),
      phase: expect.any(String),
      status_phrase: expect.any(String),
    });
    expect(Array.isArray(world.visible_objects)).toBe(true);
    expect(Array.isArray(world.available_actions)).toBe(true);
    expect(world.attention_focus).toMatchObject({ primary_object_id: expect.any(String) });

    for (const object of world.visible_objects as Array<Record<string, unknown>>) {
      expect(typeof object.id).toBe('string');
      expect(typeof object.kind).toBe('string');
      expect(typeof object.name).toBe('string');
      expect(typeof object.status).toBe('string');
      expect(typeof object.badge).toBe('string');
      expect(typeof object.salience).toBe('number');
      expect(Array.isArray(object.actions)).toBe(true);
      for (const action of object.actions as Array<Record<string, unknown>>) {
        expect(typeof action.id).toBe('string');
        expect(typeof action.label).toBe('string');
        expect(typeof action.intent).toBe('string');
        expect(typeof action.consequential).toBe('boolean');
      }
    }
  }

  function expectIntentWireShape(intent: Record<string, unknown>): void {
    expect(intent.intent_id).toMatch(/^[0-9a-f-]{36}$/);
    expectContainsUnknown(['REACT_LOCAL', 'BACKEND_INFORMATION', 'CONSEQUENTIAL_CASE'], intent.kind);
    expect(intent.action_name).toBe('BEGIN_WORK');
    expect(intent.target_object_id).toBe('obj-task-ed25519-impl');
    expect(intent.case_id).toBe(caseId);
    expect(intent.client_cursor).toMatch(/^[0-9]+\.[0-9]{10}$/);
    expect(intent.actor).toMatchObject({ actor_id: expect.any(String), role: expect.any(String) });
  }

  it('Criterion 1 & 6: Client initializes and returns valid world snapshot with monotonic cursor', async () => {
    const world = await client.getLiveWorld(caseId, 'usr-dev-alice', 'developer');

    expect(world).toBeDefined();
    expect(world.case_id).toBe(caseId);
    expect(world.cursor).toMatch(/^[0-9]+\.[0-9]{10}$/);
    expect(world.visible_objects.length).toBeGreaterThan(0);
    expect(world.summary.headline).toBe('Authentication Gateway Refactor');
    expectWorldWireShape(world as unknown as Record<string, unknown>);
  });

  it('Wire shape rejects the stale numeric/object and underspecified intent forms', () => {
    const world = {
      cursor: 42,
      objects: [],
      availableActions: [],
      actionId: 'act-stale',
    } as Record<string, unknown>;
    expect(() => expectWorldWireShape(world)).toThrow();

    const staleIntent = {
      id: 'intent-1',
      category: 'consequential_case',
      kind: 'begin-work',
      target: 'obj-task-ed25519-impl',
      expectedCursor: 42,
    } as Record<string, unknown>;
    expect(() => expectIntentWireShape(staleIntent)).toThrow();
  });

  it('Criterion 2: Role boundaries survive projection (Developer vs Security Officer vs Auditor)', async () => {
    const devWorld = await client.getLiveWorld(caseId, 'usr-dev-alice', 'developer');
    const secWorld = await client.getLiveWorld(caseId, 'usr-sec-bob', 'security_officer');
    const audWorld = await client.getLiveWorld(caseId, 'usr-aud-charlie', 'auditor');

    // 1. Developer Perspective
    const devSecurityGate = devWorld.visible_objects.find((o) => o.id === 'obj-gate-security');
    expect(devSecurityGate).toBeDefined();
    // Developer should have NO approval actions on security gate (Separation of Duties)
    expect(devSecurityGate?.actions.length).toBe(0);
    // Developer has discretionary add action
    const devDiscretionary = devWorld.visible_objects.find((o) => o.id === 'obj-disc-pen-test');
    expect(devDiscretionary?.actions.some((a) => a.id === 'act_add_work')).toBe(true);

    // 2. Security Officer Perspective
    const secSecurityGate = secWorld.visible_objects.find((o) => o.id === 'obj-gate-security');
    expect(secSecurityGate).toBeDefined();
    expect(secSecurityGate?.actions.some((a) => a.id === 'act_sec_approve')).toBe(true);
    expect(secSecurityGate?.actions.some((a) => a.id === 'act_sec_reject')).toBe(true);
    // Security Officer has administrative emergency sentry override
    expect(secWorld.available_actions.some((a) => a.id === 'act_sec_override')).toBe(true);

    // 3. Auditor Perspective
    expect(audWorld.available_actions.every((a) => !a.consequential)).toBe(true);
    audWorld.visible_objects.forEach((obj) => {
      expect(obj.actions.every((a) => !a.consequential)).toBe(true);
    });
  });

  it('Criterion 3 & 5: Governed Work to Gauntlet execution loop is deterministic', async () => {
    // Dispatch BEGIN_WORK
    const beginIntent: InteractionIntent = {
      intent_id: '11111111-2222-3333-4444-555555555555',
      kind: 'CONSEQUENTIAL_CASE',
      action_name: 'BEGIN_WORK',
      target_object_id: 'obj-task-ed25519-impl',
      case_id: caseId,
      client_cursor: '1.0000000042',
      actor: { actor_id: 'usr-dev-alice', role: 'developer' },
    };

    const response = await client.dispatchIntent(beginIntent);
    expectIntentWireShape(beginIntent as unknown as Record<string, unknown>);
    expect(response.success).toBe(true);
    expect(response.new_cursor).toMatch(/^[0-9]+\.[0-9]{10}$/);

    // World updates to reflect IN_PROGRESS
    const updatedWorld = await client.getLiveWorld(caseId, 'usr-dev-alice', 'developer');
    const task = updatedWorld.visible_objects.find((o) => o.id === 'obj-task-ed25519-impl');
    expect(task?.status).toBe('IN_PROGRESS');
  });

  it('Criterion 9: Consequential intents enforce role governance and reject unauthorized actors', async () => {
    // Developer attempts to approve security gate -> MUST BE REJECTED
    const unauthorizedIntent: InteractionIntent = {
      intent_id: '22222222-3333-4444-5555-666666666666',
      kind: 'CONSEQUENTIAL_CASE',
      action_name: 'APPROVE_HUMAN_TASK',
      target_object_id: 'obj-gate-security',
      case_id: caseId,
      client_cursor: '1.0000000043',
      actor: { actor_id: 'usr-dev-alice', role: 'developer' },
    };

    const rejectResponse = await client.dispatchIntent(unauthorizedIntent);
    expect(rejectResponse.success).toBe(false);
    expect(rejectResponse.error_code).toBe('UNAUTHORIZED_ROLE');

    // Security Officer approves gate -> MUST SUCCEED
    const authorizedIntent: InteractionIntent = {
      intent_id: '33333333-4444-5555-6666-777777777777',
      kind: 'CONSEQUENTIAL_CASE',
      action_name: 'APPROVE_HUMAN_TASK',
      target_object_id: 'obj-gate-security',
      case_id: caseId,
      client_cursor: '1.0000000043',
      actor: { actor_id: 'usr-sec-bob', role: 'security_officer' },
      parameters: { justification: 'All tests green and cryptographic diff verified.' },
    };

    const approveResponse = await client.dispatchIntent(authorizedIntent);
    expect(approveResponse.success).toBe(true);

    const settledWorld = await client.getLiveWorld(caseId, 'usr-sec-bob', 'security_officer');
    const approvedGate = settledWorld.visible_objects.find((o) => o.id === 'obj-gate-security');
    expect(approvedGate?.status).toBe('COMPLETED');
  });

  it('Criterion 1: Discretionary work addition extends plan at runtime', async () => {
    const addDiscIntent: InteractionIntent = {
      intent_id: '44444444-5555-6666-7777-888888888888',
      kind: 'CONSEQUENTIAL_CASE',
      action_name: 'ADD_DISCRETIONARY_WORK',
      target_object_id: 'obj-disc-pen-test',
      case_id: caseId,
      client_cursor: '1.0000000044',
      actor: { actor_id: 'usr-dev-alice', role: 'developer' },
    };

    const response = await client.dispatchIntent(addDiscIntent);
    expect(response.success).toBe(true);

    const world = await client.getLiveWorld(caseId, 'usr-dev-alice', 'developer');
    const addedTask = world.visible_objects.find((o) => o.id === 'obj-disc-pen-test-active');
    expect(addedTask).toBeDefined();
    expect(addedTask?.status).toBe('READY_TO_BEGIN');
  });

  it('Criterion 10: Temporal navigation returns checkpoints and truthful historical snapshot', async () => {
    const trajectory = await client.queryTemporalTrajectory(caseId);
    expect(trajectory.points.length).toBeGreaterThan(1);

    const checkpoint = trajectory.points[0];
    expect(checkpoint.cursor).toBe('1.0000000001');

    const historicalWorld = await client.getHistoricalWorld(
      caseId,
      checkpoint.cursor,
      'usr-dev-alice',
      'developer'
    );
    expect(historicalWorld).toBeDefined();
    expect(historicalWorld.summary.status_phrase).toContain('Historical Inspection Mode');
    // Consequential actions must be stripped in historical view
    historicalWorld.visible_objects.forEach((obj) => {
      expect(obj.actions.every((a) => !a.consequential)).toBe(true);
    });
  });

  it('Criterion 11: Artifact provenance survives and resolves to authoritative sources', async () => {
    const artifact = await client.resolveArtifact('evi-5a1b2c3d4e5f6a7b');
    expect(artifact).toBeDefined();
    expect(artifact.evidence_id).toBe('evi-5a1b2c3d4e5f6a7b');
    expect(artifact.digest).toMatch(/^sha256:[a-f0-9]{64}$/);
    expect(artifact.provenance.case_id).toBe(caseId);
    expect(artifact.provenance.plan_item_id).toBe('pi-stage_impl-004');
    expect(artifact.provenance.invocation_id).toBe('inv-lease-opp-pi-7b8c9d0e1f2a3b4c');
    expect(artifact.provenance.run_id).toBe('run-ed25519-bridge-20260920101500');
    expect(artifact.provenance.question_id).toBe('q-ed25519-sig-valid');
  });

  it('Criterion 12: Example JSON payloads parse cleanly and adhere to contract definitions', async () => {
    const fs = await import('fs');
    const path = await import('path');

    const examplesDir = path.resolve(import.meta.dir, '../examples');
    const exampleFiles = [
      'case-lifecycle.json',
      'role-projections-comparison.json',
      'governed-work-to-gauntlet.json',
      'realitytrace-observation-trace.json',
      'temporal-history-progression.json',
      'artifact-evidence-resolution.json',
      'event-stream-resync.json',
    ];

    for (const file of exampleFiles) {
      const filePath = path.join(examplesDir, file);
      expect(fs.existsSync(filePath)).toBe(true);
      const raw = fs.readFileSync(filePath, 'utf-8');
      const parsed = JSON.parse(raw);
      expect(parsed).toBeDefined();
    }

    const schemasDir = path.resolve(import.meta.dir, '../schemas');
    for (const file of CONTRACT_SCHEMA_FILES) {
      const filePath = path.join(schemasDir, file);
      expect(fs.existsSync(filePath)).toBe(true);
      const raw = fs.readFileSync(filePath, 'utf-8');
      const parsed = JSON.parse(raw);
      expect(parsed.$schema).toContain('json-schema.org');
      expect(parsed.$id).toBeDefined();
    }

    const interactionSchema = JSON.parse(
      fs.readFileSync(path.join(schemasDir, 'interaction-intents.schema.json'), 'utf-8')
    ) as { properties: { action_name: { enum: string[] } } };
    expect(interactionSchema.properties.action_name.enum).toEqual([...WIRE_ACTION_NAMES]);
  });

  it('pins the Ask schema inventory, exact literals, request shape, and full answer disclosure envelope', () => {
    const askSchemaPath = join(SCHEMA_DIR, 'thoth-ask.schema.json');
    expect(CONTRACT_SCHEMA_FILES).toContain('thoth-ask.schema.json');
    expect(readFileSync(askSchemaPath, 'utf-8')).toBeTruthy();
    const schema = JSON.parse(readFileSync(askSchemaPath, 'utf-8')) as {
      $schema: string;
      $id: string;
      oneOf: Array<{ $ref: string }>;
      definitions: Record<string, {
        required?: string[];
        additionalProperties?: boolean;
        description?: string;
        properties?: Record<string, { enum?: string[]; type?: string; description?: string }>;
      }>;
    };
    expect(schema.$schema).toContain('json-schema.org/draft/2020-12');
    expect(schema.$id).toBe('https://godspeed.ai/schemas/thoth-ask.schema.json');
    expect(schema.oneOf).toEqual([
      { $ref: '#/definitions/AskRequest' },
      { $ref: '#/definitions/ThothAnswerView' },
    ]);
    const request = schema.definitions.AskRequest;
    expect(request.required).toEqual(['kind', 'subject']);
    expect(request.additionalProperties).toBe(false);
    expect(Object.keys(request.properties ?? {}).sort()).toEqual(['case', 'kind', 'purpose', 'subject']);
    expect(request.properties?.subject?.type).toBe('string');
    expect(request.properties?.purpose?.type).toBe('string');
    expect(request.properties?.case?.type).toBe('string');
    expect(request.properties?.kind?.enum).toEqual([...THOTH_QUESTION_KINDS]);
    expect(request.properties?.purpose?.description?.toLowerCase()).toContain('utf-8 bytes');
    expect(request.description?.toLowerCase()).toContain('8 kib');

    const answer = schema.definitions.ThothAnswerView;
    expect(answer.required).toEqual([...ANSWER_KEYS]);
    expect(answer.additionalProperties).toBe(false);
    expect(Object.keys(answer.properties ?? {}).sort()).toEqual([...ANSWER_KEYS].sort());
    expect(answer.properties?.disposition?.enum).toEqual([...THOTH_DISPOSITIONS]);
    expect(answer.properties?.freshness?.enum).toEqual([...THOTH_FRESHNESS]);
    expect(answer.properties?.assurance?.type).toBe('string');
    expect(answer.properties?.authority_notice?.type).toBe('string');
    const claim = schema.definitions.ThothClaimView;
    expect(claim.required).toEqual([...CLAIM_KEYS]);
    expect(claim.additionalProperties).toBe(false);
    expect(Object.keys(claim.properties ?? {}).sort()).toEqual([...CLAIM_KEYS, 'capability_record_ref'].sort());
    expect(claim.properties?.claim_class?.enum).toEqual([...THOTH_CLAIM_CLASSES]);
    expect(claim.properties?.status?.enum).toEqual([...THOTH_CLAIM_STATUSES]);

    const ask = readGoldenJson('ask-request.json');
    expectExactKeys(ask, ['kind', 'subject', 'purpose', 'case']);
    expectContainsUnknown(THOTH_QUESTION_KINDS, ask.kind);
    expect(ask.subject).toBe('domain-rust');
    expect(Object.hasOwn(ask, 'actor_id')).toBe(false);
    expect(Object.hasOwn(ask, 'role')).toBe(false);

    for (const file of ANSWER_SAMPLE_FILES) {
      const answerGolden = readGoldenJson(file);
      expectThothAnswerShape(answerGolden);
      expect(answerGolden.assurance).toBe('local_tamper_evident');
    }
    expect(readGoldenJson('ask-answer-answered.json').claims).toHaveLength(1);
    expect(readGoldenJson('ask-answer-partial.json').omitted_claim_classes).toContain('security_implementation');
    expect(readGoldenJson('ask-answer-denied.json').claims).toEqual([]);

    const missingNotice = readGoldenJson('ask-answer-answered.json');
    delete missingNotice.authority_notice;
    expect(() => expectThothAnswerShape(missingNotice)).toThrow();
    const leakedClaim = readGoldenJson('ask-answer-answered.json');
    ((leakedClaim.claims as Array<Record<string, unknown>>)[0]).authored_by = 'kernel-internal';
    expect(() => expectThothAnswerShape(leakedClaim)).toThrow();
  });

  it('pins execution observation as a safe, bounded cohort side channel and preserves all stream arms', () => {
    const streamSchema = JSON.parse(
      readFileSync(join(SCHEMA_DIR, 'event-stream.schema.json'), 'utf-8')
    ) as {
      properties: { event_type: { enum: string[] } };
      definitions: Record<string, {
        type?: string;
        required?: string[];
        additionalProperties?: boolean;
        allOf?: unknown[];
        properties?: Record<string, {
          type?: string;
          enum?: string[];
          maxItems?: number;
          minimum?: number;
          items?: { type?: string; properties?: Record<string, { type?: string; enum?: string[] }> };
          properties?: Record<string, { type?: string; enum?: string[]; maxItems?: number }>;
        }>;
      }>;
    };
    expect(streamSchema.properties.event_type.enum).toEqual([...STREAM_EVENT_KINDS]);
    expect(streamSchema.properties.event_type.enum).toContain('interrupted');
    expect(streamSchema.properties.event_type.enum).toContain('error');
    const observationSchema = streamSchema.definitions.RunTraceObservation;
    expect(observationSchema.type).toBe('object');
    expect(observationSchema.required).toEqual([
      'case_id', 'observed_at', 'run_list_state', 'observation_state', 'hydration_read_budget', 'runs',
    ]);
    expect(Object.keys(observationSchema.properties ?? {}).sort()).toEqual([
      'case_id', 'observed_at', 'run_list_state', 'observation_state', 'listed_run_count',
      'selected_run_count', 'validated_run_count', 'unreadable_run_count', 'unavailable_run_count',
      'omitted_run_count', 'hydration_read_budget', 'runs',
    ].sort());
    expect(observationSchema.properties?.run_list_state?.enum).toEqual(['complete', 'unavailable']);
    expect(observationSchema.properties?.observation_state?.enum).toEqual([
      'complete', 'no_runs', 'capacity_limited', 'unavailable',
    ]);
    for (const field of [
      'listed_run_count', 'selected_run_count', 'validated_run_count', 'unreadable_run_count',
      'unavailable_run_count', 'omitted_run_count',
    ]) {
      expect(observationSchema.required).not.toContain(field);
      expect(observationSchema.properties?.[field]?.type).toBe('integer');
      expect(observationSchema.properties?.[field]?.minimum).toBe(0);
    }
    expect(observationSchema.allOf).toContainEqual({
      if: { properties: { run_list_state: { const: 'unavailable' } }, required: ['run_list_state'] },
      then: {
        allOf: [
          { not: { anyOf: [
            { required: ['listed_run_count'] }, { required: ['selected_run_count'] },
            { required: ['validated_run_count'] }, { required: ['unreadable_run_count'] },
            { required: ['unavailable_run_count'] }, { required: ['omitted_run_count'] },
          ] } },
          { properties: { runs: { maxItems: 0 } } },
        ],
      },
    });
    expect(observationSchema.allOf).toContainEqual({
      if: { properties: { observation_state: { const: 'no_runs' } }, required: ['observation_state'] },
      then: { properties: {
        run_list_state: { const: 'complete' },
        listed_run_count: { const: 0 },
        unreadable_run_count: { const: 0 },
        selected_run_count: { const: 0 },
        validated_run_count: { const: 0 },
        unavailable_run_count: { const: 0 },
        omitted_run_count: { const: 0 },
        hydration_read_budget: { properties: { reads_attempted: { const: 0 } } },
        runs: { maxItems: 0 },
      } },
    });
    expect(observationSchema.allOf).toContainEqual({
      if: { properties: { observation_state: { const: 'complete' } }, required: ['observation_state'] },
      then: { properties: {
        run_list_state: { const: 'complete' },
        listed_run_count: { minimum: 1 },
        unreadable_run_count: { const: 0 },
        unavailable_run_count: { const: 0 },
        omitted_run_count: { const: 0 },
      } },
    });
    expect(observationSchema.properties?.runs?.maxItems).toBe(8);
    expect(observationSchema.additionalProperties).toBe(false);
    const runSchema = streamSchema.definitions.RunTraceRunObservation;
    expect(runSchema.type).toBe('object');
    expect(runSchema.required).toEqual([
      'run_id', 'observed_at', 'plan_item_id', 'execution', 'settlement', 'observation_state',
      'frames', 'total_frame_count', 'retained_frame_count', 'omitted_frame_count', 'truncated',
    ]);
    expect(Object.keys(runSchema.properties ?? {}).sort()).toEqual([
      'run_id', 'observed_at', 'plan_item_id', 'execution', 'settlement', 'observation_state',
      'frames', 'total_frame_count', 'retained_frame_count', 'omitted_frame_count', 'truncated',
    ].sort());
    expect(runSchema.properties?.execution?.enum).toEqual(['pending', 'enabled', 'active', 'completed', 'failed', 'terminated']);
    expect(runSchema.properties?.settlement?.enum).toEqual(['unsettled', 'accepted', 'rejected', 'escalated']);
    expect(runSchema.properties?.observation_state?.enum).toEqual(['validated', 'unavailable']);
    expect(runSchema.properties?.frames?.maxItems).toBe(1024);
    expect(runSchema.additionalProperties).toBe(false);
    expect(runSchema.allOf).toContainEqual({
      if: { properties: { omitted_frame_count: { minimum: 1 } }, required: ['omitted_frame_count'] },
      then: { properties: { truncated: { const: true } } },
      else: { properties: { truncated: { const: false } } },
    });
    const frameSchema = streamSchema.definitions.RunTraceFrame;
    expect(frameSchema.type).toBe('object');
    expect(frameSchema.required).toEqual(['event_id', 'kind', 'timestamp']);
    expect(Object.keys(frameSchema.properties ?? {}).sort()).toEqual([
      'event_id', 'kind', 'timestamp', 'execution_status', 'exit_code',
    ].sort());
    expect(frameSchema.properties?.kind?.enum).toEqual([...SAFE_TRACE_KINDS]);
    expect(frameSchema.properties?.execution_status?.enum).toEqual([...COMMAND_EXECUTION_STATUSES]);
    expect(frameSchema.properties?.exit_code?.type).toBe('integer');
    expect(frameSchema.additionalProperties).toBe(false);
    expect(frameSchema.allOf).toContainEqual({
      if: { properties: { kind: { const: 'command_finished' } }, required: ['kind'] },
      then: {},
      else: {
        not: { anyOf: [{ required: ['execution_status'] }, { required: ['exit_code'] }] },
      },
    });

    const events = readFileSync(join(GOLDEN_DIR, 'sse-events.jsonl'), 'utf-8')
      .trim().split('\n').map((line) => JSON.parse(line) as Record<string, unknown>);
    const event = events.find((entry) => entry.event_type === 'execution_observation');
    expect(event).toBeDefined();
    expectExactKeys(event as Record<string, unknown>, ['event_type', 'cursor', 'timestamp', 'payload']);
    expect(event?.cursor).toMatch(/^\d+\.\d{10}$/);
    expect(typeof event?.timestamp).toBe('string');
    expect(event?.event_type).toBe('execution_observation');
    expectRunTraceObservationShape(event?.payload as Record<string, unknown>);
    expect(events.some((entry) => entry.event_type === 'interrupted')).toBe(true);
    expect(events.some((entry) => entry.event_type === 'error')).toBe(true);

    const payload = event?.payload as Record<string, unknown>;
    const run = (payload.runs as Array<Record<string, unknown>>)[0];
    expect(run.run_id).toBe('run-t09-command-20260923101600');
    expect(run.plan_item_id).toBe('task_prepare');
    expect(run.execution).toBe('completed');
    expect(run.settlement).toBe('rejected');
    expect((run.frames as Array<Record<string, unknown>>).some((frame) => frame.kind === 'command_finished')).toBe(true);

    const commandFinished = (run.frames as Array<Record<string, unknown>>)
      .find((frame) => frame.kind === 'command_finished');
    expect(commandFinished).toBeDefined();
    for (const status of COMMAND_EXECUTION_STATUSES.filter((status) => status !== 'completed')) {
      const withActualCommandStatus = structuredClone(payload);
      const statusRun = (withActualCommandStatus.runs as Array<Record<string, unknown>>)[0];
      const statusFrame = (statusRun.frames as Array<Record<string, unknown>>)
        .find((frame) => frame.kind === 'command_finished');
      expect(statusFrame).toBeDefined();
      statusFrame!.execution_status = status;
      expect(() => expectRunTraceObservationShape(withActualCommandStatus)).not.toThrow();
    }
    for (const status of RUN_EXECUTION_STANDINGS.filter((status) => status !== 'completed')) {
      const withStandingAsCommandStatus = structuredClone(payload);
      const statusRun = (withStandingAsCommandStatus.runs as Array<Record<string, unknown>>)[0];
      const statusFrame = (statusRun.frames as Array<Record<string, unknown>>)
        .find((frame) => frame.kind === 'command_finished');
      expect(statusFrame).toBeDefined();
      statusFrame!.execution_status = status;
      expect(() => expectRunTraceObservationShape(withStandingAsCommandStatus)).toThrow();
    }

    const unavailable: RunTraceObservation = {
      case_id: 'case-t09-unavailable',
      observed_at: '2026-09-23T10:18:00Z',
      run_list_state: 'unavailable',
      observation_state: 'unavailable',
      hydration_read_budget: { limit: 8, reads_attempted: 0, exhausted: false },
      runs: [],
    };
    expectRunTraceObservationShape(unavailable as unknown as Record<string, unknown>);
    expect(unavailable.runs).toEqual([]);
    expect(() => expectRunTraceObservationShape({ ...unavailable, listed_run_count: 0 })).toThrow();
    const missingOwnership = { ...payload };
    delete missingOwnership.case_id;
    expect(() => expectRunTraceObservationShape(missingOwnership)).toThrow();
    const fakeProgress = structuredClone(run);
    (fakeProgress.frames as Array<Record<string, unknown>>)[0].progress_percent = 0.5;
    expect(() => expectRunTraceObservationShape({ ...payload, runs: [fakeProgress] })).toThrow();

    const invalidRun = structuredClone(run);
    delete invalidRun.run_id;
    expect(() => expectRunTraceObservationShape({
      ...payload,
      runs: [invalidRun],
    })).toThrow();
    const unsafeFrame = structuredClone(run);
    (unsafeFrame.frames as Array<Record<string, unknown>>)[0].actor_id = 'actor-not-for-forwarding';
    expect(() => expectRunTraceObservationShape({
      ...payload,
      runs: [unsafeFrame],
    })).toThrow();
  });
});
