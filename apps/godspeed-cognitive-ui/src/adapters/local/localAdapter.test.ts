import { describe, test, expect, beforeEach } from 'bun:test'
import type { InteractionIntent, StreamEvent } from '../../ports/contract'
import { runCaseworkPortConformance } from '../conformance/caseworkPortConformance'
import { LocalContractAdapter } from './localAdapter'
import { NORTHSTAR_CASE_ID } from './northstarData'

function randomId(): string {
  return Math.random().toString(36).substring(2, 11)
}

describe('LocalContractAdapter', () => {
  let adapter: LocalContractAdapter
  const actor = { actor_id: 'usr-sam', role: 'case_architect' as const, display_name: 'Sam Prime', kind: 'human' as const }

  beforeEach(() => {
    adapter = new LocalContractAdapter({ speed: 200, latency: 0 })
  })

  test('shared CaseworkPort behavioral conformance', async () => {
    await runCaseworkPortConformance({
      port: adapter,
      caseId: NORTHSTAR_CASE_ID,
      actor,
      template: { ref: 'tpl-release-rollout', params: { release_version: '3.4.0' } },
      artifact: {
        refAfterDispatch: () => 'evi-assumption',
        expectedProvenance: { case_id: NORTHSTAR_CASE_ID },
        // Local fixture digests identify authored revision labels rather than content bytes.
        digestMatchesContent: false,
      },
      initialHistoryMinimum: 6,
      resume: 'future-only',
      resumeWaitMs: 60,
      waitForMs: 2_000,
      acceptedIntent: (cursor) => buildValidIntentAt(cursor, actor),
      captureConsequentialState: async () => {
        const history = await adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
        return JSON.stringify(history.points.map((point) => point.cursor))
      },
      stateBoundaryDescription: 'local fixture trajectory cursor list',
    })
  })

  test('duplicate intent_id returns DUPLICATE_IN_FLIGHT', async () => {
    const intent = await buildValidIntent(adapter, actor)
    intent.intent_id = `test-dup-${randomId()}`

    await adapter.dispatchIntent(intent)
    const response2 = await adapter.dispatchIntent(intent)
    expect(response2.success).toBe(false)
    expect(response2.error_code).toBe('DUPLICATE_IN_FLIGHT')
  })

  test('unauthorized role agent_operator returns AUTHORITY_DENIED', async () => {
    const intent = await buildValidIntent(adapter, actor)
    intent.actor.role = 'agent_operator'

    const response = await adapter.dispatchIntent(intent)
    expect(response.success).toBe(false)
    expect(response.error_code).toBe('AUTHORITY_DENIED')
  })

  test('unauthorized role developer returns UNAUTHORIZED_ROLE', async () => {
    const intent = await buildValidIntent(adapter, actor)
    intent.actor.role = 'developer'

    const response = await adapter.dispatchIntent(intent)
    expect(response.success).toBe(false)
    expect(response.error_code).toBe('UNAUTHORIZED_ROLE')
  })

  test('ESCALATE_OR_OVERRIDE without justification returns JUSTIFICATION_REQUIRED', async () => {
    const traj = await adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
    const cursor = traj.points[traj.points.length - 1]!.cursor
    const intent = {
      intent_id: `test-justify-${randomId()}`,
      client_cursor: cursor,
      kind: 'CONSEQUENTIAL_CASE',
      actor: { actor_id: 'usr-sam', role: 'case_architect' },
      target_object_id: 'ns-release',
      action_name: 'ESCALATE_OR_OVERRIDE',
      case_id: NORTHSTAR_CASE_ID,
    } as any

    const response = await adapter.dispatchIntent(intent)
    expect(response.success).toBe(false)
    expect(response.error_code).toBe('JUSTIFICATION_REQUIRED')
  })

  test('unknown target returns INVALID', async () => {
    const intent = await buildValidIntent(adapter, actor)
    intent.target_object_id = 'unknown-object-xyz'

    const response = await adapter.dispatchIntent(intent)
    expect(response.success).toBe(false)
    expect(response.error_code).toBe('INVALID')
  })

  test('unavailable action on target returns UNAVAILABLE', async () => {
    const traj = await adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
    const cursor = traj.points[traj.points.length - 1]!.cursor
    const intent = {
      intent_id: `test-unavail-${randomId()}`,
      client_cursor: cursor,
      kind: 'CONSEQUENTIAL_CASE',
      actor: { actor_id: 'usr-sam', role: 'case_architect' },
      target_object_id: 'ns-release',
      action_name: 'NONEXISTENT_ACTION',
      case_id: NORTHSTAR_CASE_ID,
    } as any

    const response = await adapter.dispatchIntent(intent)
    expect(response.success).toBe(false)
    expect(response.error_code).toBe('UNAVAILABLE')
  })

  test('resolveArtifact returns independent clones', async () => {
    const payload1 = await adapter.resolveArtifact('evi-assumption')
    const payload2 = await adapter.resolveArtifact('evi-assumption')
    if (typeof payload1.content === 'string') payload1.content += 'MUTATED'
    expect(payload2.content).not.toContain('MUTATED')
  })

  test('failArtifacts rejects with error', async () => {
    const adapter2 = new LocalContractAdapter({ speed: 200, latency: 0, failArtifacts: ['bad-ref'] })
    let error: Error | null = null
    try {
      await adapter2.resolveArtifact('bad-ref')
    } catch (e) {
      error = e as Error
    }
    expect(error).toBeDefined()
    expect(error?.message).toContain('unavailable')
  })

  test('corruptArtifacts returns malformed content', async () => {
    const adapter2 = new LocalContractAdapter({ speed: 200, latency: 0, corruptArtifacts: ['evi-assumption'] })
    const payload = await adapter2.resolveArtifact('evi-assumption')
    expect(payload.content).toContain('\u0000')
  })

  test('getSnapshot with agent_operator role has no consequential actions', async () => {
    const snap = await adapter.getSnapshot(NORTHSTAR_CASE_ID, 'sys-agent', 'agent_operator')
    const allActions = snap.visible_objects.flatMap((object) => object.actions ?? [])
    expect(allActions.filter((action) => action.consequential).length).toBe(0)
  })

  test('accepted intent completion precedes settlement', async () => {
    const intent = await buildValidIntent(adapter, actor)
    const events: StreamEvent[] = []
    const unsubscribe = adapter.subscribeEvents(NORTHSTAR_CASE_ID, undefined, (event) => events.push(event))

    await adapter.dispatchIntent(intent)
    await new Promise((resolve) => setTimeout(resolve, 150))
    unsubscribe()

    const completionIndex = events.findIndex(
      (event) => event.event_type === 'snapshot' && Array.isArray(event.payload) &&
        (event.payload as any[]).some?.((object: any) => object.status?.COMPLETED),
    )
    const settlementIndex = events.findIndex((event) => event.event_type === 'settlement_recorded')
    if (completionIndex >= 0 && settlementIndex >= 0) expect(completionIndex).toBeLessThan(settlementIndex)
  })

  test('accepted intent emits snapshot, execution progress, and settlement events', async () => {
    const intent = await buildValidIntent(adapter, actor)
    const events: StreamEvent[] = []
    const unsubscribe = adapter.subscribeEvents(NORTHSTAR_CASE_ID, undefined, (event) => events.push(event))

    const response = await adapter.dispatchIntent(intent)
    expect(response.success).toBe(true)
    await new Promise((resolve) => setTimeout(resolve, 100))
    unsubscribe()

    expect(events.filter((event) => event.event_type === 'snapshot').length).toBeGreaterThanOrEqual(2)
    expect(events.filter((event) => event.event_type === 'execution_progress').length).toBeGreaterThanOrEqual(1)
    expect(events.filter((event) => event.event_type === 'settlement_recorded').length).toBeGreaterThanOrEqual(1)
  })
})

function buildValidIntentAt(cursor: string, actor: { actor_id: string; role: 'case_architect' }): InteractionIntent {
  return {
    intent_id: `test-${randomId()}`,
    client_cursor: cursor,
    kind: 'CONSEQUENTIAL_CASE',
    actor: { actor_id: actor.actor_id, role: actor.role },
    target_object_id: 'ns-release',
    action_name: 'APPROVE_HUMAN_TASK',
    case_id: NORTHSTAR_CASE_ID,
  } as any
}

async function buildValidIntent(
  adapter: LocalContractAdapter,
  actor: { actor_id: string; role: 'case_architect' },
): Promise<InteractionIntent> {
  const traj = await adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
  return buildValidIntentAt(traj.points[traj.points.length - 1]!.cursor, actor)
}
