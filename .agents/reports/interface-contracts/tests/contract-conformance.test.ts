import { describe, expect, it } from 'bun:test';
import { createCaseworkClient, MockCaseworkAdapter } from '../typescript';
import { InteractionActionName, InteractionIntent } from '../typescript/types';

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
    expect(['REACT_LOCAL', 'BACKEND_INFORMATION', 'CONSEQUENTIAL_CASE']).toContain(intent.kind);
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
    const schemaFiles = [
      'cognitive-world.schema.json',
      'interaction-intents.schema.json',
      'case-operations.schema.json',
      'execution-contracts.schema.json',
      'realitytrace-evidence.schema.json',
      'event-stream.schema.json',
    ];

    for (const file of schemaFiles) {
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
});
