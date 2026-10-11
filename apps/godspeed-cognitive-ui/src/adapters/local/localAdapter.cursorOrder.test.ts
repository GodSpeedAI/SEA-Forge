import { describe, expect, test } from 'bun:test'
import type { CaseworkPort, InteractionIntent, StreamEvent, XObject, XSnapshot } from '../../ports/contract'
import { LocalContractAdapter, NORTHSTAR_CASE_ID } from './localAdapter'
import { TEMPLATE_CASE_ID } from './templateData'

type ManualTimer = { callback: () => void; due: number; order: number; cancelled: boolean; fired: boolean }

function installManualTimers() {
  const pending: ManualTimer[] = []
  let now = 0
  let scheduled = 0
  const maxCallbacks = 500
  const maxVirtualTime = 60_000
  const setTimeoutBefore = globalThis.setTimeout
  const clearTimeoutBefore = globalThis.clearTimeout
  globalThis.setTimeout = ((callback: TimerHandler, delay?: number) => {
    if (typeof callback !== 'function') throw new TypeError('manual timers require a callback')
    if (++scheduled > maxCallbacks) throw new Error(`manual timer scheduling exceeded ${maxCallbacks} callbacks`)
    pending.push({ callback: () => callback(), due: now + Math.max(0, Number(delay ?? 0)), order: scheduled, cancelled: false, fired: false })
    return pending.length as unknown as ReturnType<typeof setTimeout>
  }) as unknown as typeof setTimeout
  globalThis.clearTimeout = ((handle: ReturnType<typeof setTimeout>) => {
    const timer = pending[Number(handle) - 1]
    if (timer) timer.cancelled = true
  }) as typeof clearTimeout
  return {
    runNext() {
      const timer = pending.filter((candidate) => !candidate.cancelled && !candidate.fired)
        .sort((left, right) => left.due - right.due || left.order - right.order)[0]
      if (!timer) throw new Error('no pending timer')
      if (timer.due > maxVirtualTime) throw new Error(`manual timer advancement exceeded ${maxVirtualTime}ms`)
      now = timer.due
      timer.fired = true
      timer.callback()
      return timer
    },
    hasPending: () => pending.some((timer) => !timer.cancelled && !timer.fired),
    restore() {
      globalThis.setTimeout = setTimeoutBefore
      globalThis.clearTimeout = clearTimeoutBefore
    },
  }
}

const actor = { actor_id: 'usr-sam', role: 'case_architect' as const }

function appendFixture(adapter: LocalContractAdapter, summary: string, caseId = NORTHSTAR_CASE_ID): XSnapshot {
  // Whitebox is limited to the adapter's existing synchronous append signature so a
  // pre-subscription event can be queued deterministically; public dispatch is covered below.
  const subject = adapter as unknown as {
    append(caseId: string, eventType: string, summary: string, intent: InteractionIntent | null, edit: (objects: XObject[]) => void): XSnapshot
  }
  return subject.append(caseId, 'fixture_revision', summary, null, () => undefined)
}

async function readSnapshot(adapter: LocalContractAdapter, timers: ReturnType<typeof installManualTimers>) {
  const pending = adapter.getSnapshot(NORTHSTAR_CASE_ID, actor.actor_id, actor.role)
  timers.runNext()
  return pending
}

async function drain(timers: ReturnType<typeof installManualTimers>) {
  let callbacks = 0
  while (timers.hasPending()) {
    if (++callbacks > 500) throw new Error('manual timer drain exceeded 500 callbacks')
    timers.runNext()
    await Promise.resolve()
  }
  await Promise.resolve()
}

function intent(cursor: string): InteractionIntent {
  return {
    intent_id: `cursor-order-${Math.random().toString(36).slice(2)}`,
    client_cursor: cursor,
    kind: 'CONSEQUENTIAL_CASE',
    actor,
    target_object_id: 'ns-release',
    action_name: 'APPROVE_HUMAN_TASK',
    case_id: NORTHSTAR_CASE_ID,
  } as InteractionIntent
}

describe('LocalContractAdapter cursor ordering', () => {
  test('undefined boundary does not receive an already queued event; later revision is delivered', async () => {
    const timers = installManualTimers()
    try {
      const adapter = new LocalContractAdapter({ speed: 200, latency: 0 })
      const prior: StreamEvent[] = []
      const late: StreamEvent[] = []
      const unsubscribePrior = adapter.subscribeEvents(NORTHSTAR_CASE_ID, undefined, (event) => prior.push(event))
      const h1 = appendFixture(adapter, 'queued before undefined subscription')
      const errors: Error[] = []
      const port: CaseworkPort = adapter
      const unsubscribeLate = port.subscribeEvents(NORTHSTAR_CASE_ID, undefined, (event) => late.push(event), (error) => errors.push(error))
      timers.runNext()
      expect(prior.map((event) => event.cursor)).toEqual([h1.cursor])
      expect(late).toEqual([])
      expect(errors).toEqual([])

      const h2 = appendFixture(adapter, 'created after undefined subscription')
      await drain(timers)
      expect(late.map((event) => event.cursor)).toEqual([h2.cursor])
      unsubscribeLate()
      appendFixture(adapter, 'created after unsubscribe')
      await drain(timers)
      expect(late.map((event) => event.cursor)).toEqual([h2.cursor])
      unsubscribePrior()
    } finally {
      timers.restore()
    }
  })

  test('same-epoch future floor suppresses its boundary allocation but delivers a later event', async () => {
    const timers = installManualTimers()
    try {
      const adapter = new LocalContractAdapter({ speed: 200, latency: 0 })
      const trajectory = adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
      timers.runNext()
      const head = (await trajectory).head_cursor
      const floor = nextCursor(head)
      const received: StreamEvent[] = []
      const port: CaseworkPort = adapter
      const unsubscribe = port.subscribeEvents(NORTHSTAR_CASE_ID, floor, (event) => received.push(event), () => {})

      const atBoundary = appendFixture(adapter, 'event allocated at the valid future floor')
      expect(atBoundary.cursor).toBe(floor)
      await drain(timers)
      expect(received).toEqual([])

      const aboveBoundary = appendFixture(adapter, 'genuine event allocated above the floor')
      expect(aboveBoundary.cursor).toBe(nextCursor(floor))
      await drain(timers)
      expect(received.map((event) => event.cursor)).toEqual([aboveBoundary.cursor])
      unsubscribe()
    } finally {
      timers.restore()
    }
  })

  test('supplied revision boundary does not receive queued H+1 and receives later H+2', async () => {
    const timers = installManualTimers()
    try {
      const adapter = new LocalContractAdapter({ speed: 200, latency: 0 })
      const head = adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
      timers.runNext()
      const h = (await head).head_cursor
      const existing: StreamEvent[] = []
      const stopExisting = adapter.subscribeEvents(NORTHSTAR_CASE_ID, undefined, (event) => existing.push(event))
      const queued = appendFixture(adapter, 'H+1 queued before registration')
      const received: StreamEvent[] = []
      const unsubscribe = adapter.subscribeEvents(NORTHSTAR_CASE_ID, h, (event) => received.push(event))
      timers.runNext()
      expect(queued.cursor).not.toBe(h)
      expect(existing.map((event) => event.cursor)).toEqual([queued.cursor])
      expect(received).toEqual([])
      const later = appendFixture(adapter, 'H+2 after registration')
      await drain(timers)
      expect(received.map((event) => event.cursor)).toEqual([later.cursor])
      unsubscribe()
      stopExisting()
    } finally {
      timers.restore()
    }
  })

  test('public execution orders progress, snapshots, settlement, and subsequent snapshot cursors', async () => {
    const timers = installManualTimers()
    try {
      const adapter = new LocalContractAdapter({ speed: 200, latency: 0 })
      const initial = await readSnapshot(adapter, timers)
      const events: StreamEvent[] = []
      const unsubscribe = adapter.subscribeEvents(NORTHSTAR_CASE_ID, initial.cursor, (event) => events.push(event))
      const response = adapter.dispatchIntent(intent(initial.cursor))
      timers.runNext()
      expect((await response).success).toBe(true)
      await drain(timers)
      const firstSettlement = events.findIndex((event) => event.event_type === 'settlement_recorded')
      expect(events.some((event) => event.event_type === 'execution_progress')).toBe(true)
      expect(firstSettlement).toBeGreaterThan(0)
      expect(events[firstSettlement - 1]?.event_type).toBe('snapshot')
      const settlementSnapshot = adapter.getSnapshotAt(
        NORTHSTAR_CASE_ID,
        events[firstSettlement]!.cursor,
        actor.actor_id,
        actor.role,
      )
      timers.runNext()
      await expect(settlementSnapshot).rejects.toThrow('No snapshot at')
      const ordinaryCursors = events.map((event) => event.cursor)
      expect(new Set(ordinaryCursors).size).toBe(ordinaryCursors.length)
      const numeric = ordinaryCursors.map((cursor) => cursor.split('.').map(Number))
      for (let index = 1; index < numeric.length; index++) {
        const [previousEpoch, previousSequence] = numeric[index - 1]!
        const [epoch, sequence] = numeric[index]!
        expect(epoch! > previousEpoch! || (epoch === previousEpoch && sequence! > previousSequence!)).toBe(true)
      }
      const [settlementEpoch, settlementSequence] = events[firstSettlement]!.cursor.split('.').map(Number)
      const [snapshotEpoch, snapshotSequence] = events[firstSettlement - 1]!.cursor.split('.').map(Number)
      expect(settlementEpoch! > snapshotEpoch! || (settlementEpoch === snapshotEpoch && settlementSequence! > snapshotSequence!)).toBe(true)
      const next = appendFixture(adapter, 'snapshot after settlement side event')
      timers.runNext()
      expect(events.at(-1)?.event_type).toBe('snapshot')
      expect(events.at(-1)?.cursor).toBe(next.cursor)
      const [lastEpoch, lastSequence] = ordinaryCursors.at(-1)!.split('.').map(Number)
      const [nextEpoch, nextSequence] = next.cursor.split('.').map(Number)
      expect(nextEpoch! > lastEpoch! || (nextEpoch === lastEpoch && nextSequence! > lastSequence!)).toBe(true)
      unsubscribe()
    } finally {
      timers.restore()
    }
  })

  test('progress leaves retained snapshots unchanged and each new snapshot adds one history point', async () => {
    const timers = installManualTimers()
    try {
      const adapter = new LocalContractAdapter({ speed: 200, latency: 0 })
      const beforePending = adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
      timers.runNext()
      const before = await beforePending
      const snapshotsBefore: XSnapshot[] = []
      for (const point of before.points) {
        const snapshot = adapter.getSnapshotAt(NORTHSTAR_CASE_ID, point.cursor, actor.actor_id, actor.role)
        timers.runNext()
        snapshotsBefore.push(await snapshot)
      }
      const progressOnly = adapter as unknown as {
        progress(caseId: string, run: string, phase: 'builder', percent: number, log: string): void
      }
      const progress: StreamEvent[] = []
      const unsubscribe = adapter.subscribeEvents(NORTHSTAR_CASE_ID, undefined, (event) => progress.push(event))
      progressOnly.progress(NORTHSTAR_CASE_ID, 'run-fixture', 'builder', 0.35, 'controlled progress')
      timers.runNext()
      expect(progress[0]?.event_type).toBe('execution_progress')
      const afterProgress = adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
      timers.runNext()
      expect(await afterProgress).toEqual(before)
      const snapshotsAfterProgress: XSnapshot[] = []
      for (const point of before.points) {
        const snapshot = adapter.getSnapshotAt(NORTHSTAR_CASE_ID, point.cursor, actor.actor_id, actor.role)
        timers.runNext()
        snapshotsAfterProgress.push(await snapshot)
      }
      expect(snapshotsAfterProgress).toEqual(snapshotsBefore)
      unsubscribe()

      const added = appendFixture(adapter, 'one immutable history addition')
      await drain(timers)
      const after = adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
      timers.runNext()
      const finalTrajectory = await after
      expect(finalTrajectory.points).toHaveLength(before.points.length + 1)
      expect(finalTrajectory.points.at(-1)?.cursor).toBe(added.cursor)
      const progressSnapshot = adapter.getSnapshotAt(NORTHSTAR_CASE_ID, progress[0]!.cursor, actor.actor_id, actor.role)
      timers.runNext()
      await expect(progressSnapshot).rejects.toThrow('No snapshot at')
    } finally {
      timers.restore()
    }
  })

  test('event delivery is case-local and unsubscribe before queued drain prevents delivery', async () => {
    const timers = installManualTimers()
    try {
      const adapter = new LocalContractAdapter({ speed: 200, latency: 0 })
      const first: StreamEvent[] = []
      const second: StreamEvent[] = []
      const initialNorthstar = adapter.queryTemporalTrajectory(NORTHSTAR_CASE_ID)
      timers.runNext()
      const northstarHead = (await initialNorthstar).head_cursor
      const initialTemplate = adapter.queryTemporalTrajectory(TEMPLATE_CASE_ID)
      timers.runNext()
      const templateHead = (await initialTemplate).head_cursor
      const stopFirst = adapter.subscribeEvents(NORTHSTAR_CASE_ID, undefined, (event) => first.push(event))
      const stopSecond = adapter.subscribeEvents(TEMPLATE_CASE_ID, undefined, (event) => second.push(event))
      const queued = appendFixture(adapter, 'case local queued event')
      expect(queued.cursor).toBe(nextCursor(northstarHead))
      stopFirst()
      expect(timers.hasPending()).toBe(false)
      expect(first).toEqual([])
      expect(second).toEqual([])
      const independent = appendFixture(adapter, 'independent case event', TEMPLATE_CASE_ID)
      expect(independent.cursor).toBe(nextCursor(templateHead))
      await drain(timers)
      expect(second.map((event) => event.cursor)).toEqual([independent.cursor])
      stopSecond()
    } finally {
      timers.restore()
    }
  })
})

function nextCursor(cursor: string) {
  const [epoch, sequence] = cursor.split('.').map(Number)
  return `${epoch}.${String(sequence! + 1).padStart(10, '0')}`
}
