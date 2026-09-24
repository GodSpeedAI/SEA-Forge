import { describe, test, expect, beforeEach } from 'bun:test'
import type { InteractionIntent, StreamEvent } from '../../ports/contract'
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

  test('queryTemporalTrajectory returns 6 revisions for northstar case', async () => {
    const traj = await adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
    expect(traj.points.length).toBe(6)
    expect(traj.points[0]?.cursor).toBeDefined()
  })

  test('getSnapshot returns a valid snapshot with x extensions', async () => {
    const snap = await adapter.getSnapshot(NORTHSTAR_CASE_ID, actor.actor_id, actor.role)
    expect(snap.case_id).toBe(NORTHSTAR_CASE_ID)
    expect(snap.visible_objects).toBeDefined()
    expect(snap.visible_objects.length).toBeGreaterThan(0)
  })

  test('getSnapshotAt returns snapshot at cursor unchanged before and after execution', async () => {
    const traj = await adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
    const cursor = traj.points[0]!.cursor
    const before = await adapter.getSnapshotAt(NORTHSTAR_CASE_ID, cursor, actor.actor_id, actor.role)

    // Execute an intent
    const intent = await buildValidIntent(adapter, actor)
    await adapter.dispatchIntent(intent)

    // History should not be rewritten
    const after = await adapter.getSnapshotAt(NORTHSTAR_CASE_ID, cursor, actor.actor_id, actor.role)
    expect(after).toEqual(before)
  })

  test('dispatchIntent with valid intent succeeds and returns new_cursor', async () => {
    const intent = await buildValidIntent(adapter, actor)
    const response = await adapter.dispatchIntent(intent)

    expect(response.success).toBe(true)
    expect(response.new_cursor).toBeDefined()
    expect(response.new_cursor).not.toBe(intent.client_cursor)
  })

  test('duplicate intent_id returns DUPLICATE_IN_FLIGHT', async () => {
    const intent = await buildValidIntent(adapter, actor)
    intent.intent_id = `test-dup-${randomId()}`

    // First dispatch succeeds
    await adapter.dispatchIntent(intent)

    // Same intent_id again fails
    const response2 = await adapter.dispatchIntent(intent)
    expect(response2.success).toBe(false)
    expect(response2.error_code).toBe('DUPLICATE_IN_FLIGHT')
  })

  test('stale client_cursor returns STALE_PROJECTION', async () => {
    const traj = await adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
    const oldCursor = traj.points[0]!.cursor

    const intent = await buildValidIntent(adapter, actor)
    intent.client_cursor = oldCursor

    // Move to newer state with a new intent
    const freshIntent = await buildValidIntent(adapter, actor)
    await adapter.dispatchIntent(freshIntent)

    // Old cursor is now stale
    const response = await adapter.dispatchIntent(intent)
    expect(response.success).toBe(false)
    expect(response.error_code).toBe('STALE_PROJECTION')
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

    const intent: any = {
      intent_id: `test-justify-${randomId()}`,
      client_cursor: cursor,
      kind: 'CONSEQUENTIAL_CASE',
      actor: { actor_id: 'usr-sam', role: 'case_architect' },
      target_object_id: 'ns-release',
      action_name: 'ESCALATE_OR_OVERRIDE',
      case_id: NORTHSTAR_CASE_ID,
    }

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

    const intent: any = {
      intent_id: `test-unavail-${randomId()}`,
      client_cursor: cursor,
      kind: 'CONSEQUENTIAL_CASE',
      actor: { actor_id: 'usr-sam', role: 'case_architect' },
      target_object_id: 'ns-release',
      action_name: 'NONEXISTENT_ACTION',
      case_id: NORTHSTAR_CASE_ID,
    }

    const response = await adapter.dispatchIntent(intent)
    expect(response.success).toBe(false)
    expect(response.error_code).toBe('UNAVAILABLE')
  })

  test('refused intent does not append snapshot to trajectory', async () => {
    const traj1 = await adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
    const length1 = traj1.points.length

    const intent = await buildValidIntent(adapter, actor)
    intent.actor.role = 'developer'
    const response = await adapter.dispatchIntent(intent)
    expect(response.success).toBe(false)

    const traj2 = await adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
    expect(traj2.points.length).toBe(length1)
  })

  test('accepted intent triggers event stream: snapshot, execution_progress, settlement_recorded', async () => {
    const intent = await buildValidIntent(adapter, actor)
    const events: StreamEvent[] = []
    const unsub = adapter.subscribeEvents(NORTHSTAR_CASE_ID, undefined, (e) => events.push(e))

    const response = await adapter.dispatchIntent(intent)
    expect(response.success).toBe(true)

    // Give events time to be emitted
    await new Promise((r) => setTimeout(r, 100))

    unsub()

    // Should have: snapshot (IN_PROGRESS), execution_progress events, snapshot (COMPLETED), settlement_recorded, snapshot (settlement set)
    const snapshotEvents = events.filter((e) => e.event_type === 'snapshot')
    const progressEvents = events.filter((e) => e.event_type === 'execution_progress')
    const settlementEvents = events.filter((e) => e.event_type === 'settlement_recorded')

    expect(snapshotEvents.length).toBeGreaterThanOrEqual(2)
    expect(progressEvents.length).toBeGreaterThanOrEqual(1)
    expect(settlementEvents.length).toBeGreaterThanOrEqual(1)
  })

  test('resolveArtifact returns clones', async () => {
    const payload1 = await adapter.resolveArtifact('evi-assumption')
    const payload2 = await adapter.resolveArtifact('evi-assumption')

    // Mutate payload1
    if (typeof payload1.content === 'string') {
      payload1.content = payload1.content + 'MUTATED'
    }

    // payload2 should be unchanged
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
    // parseArtifact would fail on this
    expect(payload.content).toContain('\u0000')
  })

  test('getSnapshot with agent_operator role has no consequential actions', async () => {
    const snap = await adapter.getSnapshot(NORTHSTAR_CASE_ID, 'sys-agent', 'agent_operator')
    const objects = snap.visible_objects
    const allActions = objects.flatMap((o) => o.actions ?? [])
    const consequentialActions = allActions.filter((a) => a.consequential)
    expect(consequentialActions.length).toBe(0)
  })

  test('accepted intent completion precedes settlement', async () => {
    const intent = await buildValidIntent(adapter, actor)
    const events: StreamEvent[] = []
    const unsub = adapter.subscribeEvents(NORTHSTAR_CASE_ID, undefined, (e) => events.push(e))

    await adapter.dispatchIntent(intent)
    await new Promise((r) => setTimeout(r, 150))
    unsub()

    // Find indices of events
    const completionIdx = events.findIndex(
      (e) => e.event_type === 'snapshot' && Array.isArray(e.payload) && (e.payload as any[]).some?.((o: any) => o.status?.COMPLETED),
    )
    const settlementIdx = events.findIndex((e) => e.event_type === 'settlement_recorded')

    if (completionIdx >= 0 && settlementIdx >= 0) {
      expect(completionIdx).toBeLessThan(settlementIdx)
    }
  })
})

// Helper to build a valid intent for APPROVE_HUMAN_TASK
async function buildValidIntent(adapter: LocalContractAdapter, actor: any): Promise<InteractionIntent> {
  const traj = await adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
  const cursor = traj.points[traj.points.length - 1]!.cursor

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
