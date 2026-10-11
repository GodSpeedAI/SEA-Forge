import { describe, expect, it } from 'bun:test'
import { buildNorthstarHistory } from '../adapters/local/northstarData'
import { createStore, initialState, nowRevision } from '../model/store'
import { projectHistory } from '../ports/project'
import type { CaseworkPort, StreamEvent, XSnapshot } from '../ports/contract'
import { connectLive, executionStateOf } from './live'

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))
const unexpectedCall = (method: string): never => {
  throw new Error(`Unexpected CaseworkPort.${method} call in live recovery test`)
}

function liveTestPort(subscribe: CaseworkPort['subscribeEvents']): CaseworkPort {
  return {
    getSnapshot: () => unexpectedCall('getSnapshot'),
    getSnapshotAt: () => unexpectedCall('getSnapshotAt'),
    dispatchIntent: () => unexpectedCall('dispatchIntent'),
    resolveArtifact: () => unexpectedCall('resolveArtifact'),
    queryTemporalTrajectory: () => unexpectedCall('queryTemporalTrajectory'),
    subscribeEvents: subscribe,
  }
}

describe('connectLive connection recovery', () => {
  it('marks a prolonged outage interrupted and returns live only after a source snapshot', async () => {
    const raw = buildNorthstarHistory({ actor_id: 'operator-1', role: 'case_architect' })
    const history = projectHistory(raw, {}, 'local-contract', { withCore: true })
    const store = createStore(initialState(history))
    let onEvent: ((event: StreamEvent) => void) | undefined
    let onError: ((error: Error) => void) | undefined
    const port = liveTestPort((_caseId, _since, eventHandler, errorHandler) => {
      onEvent = eventHandler
      onError = errorHandler
      return () => undefined
    })
    const disconnect = connectLive(port, store, history.caseId, raw, {}, { interruptedAfterMs: 15 })

    onError!(new Error('stream disconnected'))
    expect(store.getState().connection).toBe('reconnecting')
    onEvent!({ event_type: 'heartbeat', cursor: '', timestamp: '', payload: {} })
    onEvent!({ event_type: 'error', cursor: '', timestamp: '', payload: { error_code: 'gap', message: 'not recovered' } })
    expect(store.getState().connection).toBe('reconnecting')
    await wait(30)
    expect(store.getState().connection).toBe('interrupted')

    const head = raw.at(-1)!
    const refreshed: XSnapshot = {
      ...head,
      cursor: `${head.cursor.split('.')[0]}.${Number(head.cursor.split('.')[1] ?? 0) + 1}`,
      timestamp: new Date(Date.parse(head.timestamp) + 1000).toISOString(),
      summary: { ...head.summary, phase: 'source resynced' },
    }
    onEvent!({ event_type: 'snapshot', cursor: refreshed.cursor, timestamp: refreshed.timestamp, payload: refreshed })
    expect(store.getState().connection).toBe('live')
    expect(nowRevision(store.getState().history)).toBe(refreshed.cursor)
    disconnect()
  })

  it('clears the interruption timer when the subscription is disposed', async () => {
    const raw = buildNorthstarHistory({ actor_id: 'operator-1', role: 'case_architect' })
    const history = projectHistory(raw, {}, 'local-contract', { withCore: true })
    const store = createStore(initialState(history))
    let onError: ((error: Error) => void) | undefined
    const port = liveTestPort((_caseId, _since, _onEvent, errorHandler) => {
      onError = errorHandler
      return () => undefined
    })
    const disconnect = connectLive(port, store, history.caseId, raw, {}, { interruptedAfterMs: 15 })

    onError!(new Error('stream disconnected'))
    disconnect()
    await wait(30)
    expect(store.getState().connection).toBe('reconnecting')
  })

  it('keeps a stale-refusal refresh in history when a later live snapshot arrives', () => {
    const raw = buildNorthstarHistory({ actor_id: 'operator-1', role: 'case_architect' })
    const history = projectHistory(raw, {}, 'local-contract', { withCore: true })
    const store = createStore(initialState(history))
    const refreshed: XSnapshot = {
      ...raw.at(-1)!,
      cursor: '9.0000000001',
      timestamp: new Date().toISOString(),
      summary: { ...raw.at(-1)!.summary, phase: 'stale refusal refresh' },
    }
    store.dispatch({
      type: 'loadHistory',
      history: {
        ...history,
        revisions: [...history.revisions, { id: refreshed.cursor, at: refreshed.timestamp, label: 'Now', summary: refreshed.summary.phase }],
        snapshots: { ...history.snapshots, [refreshed.cursor]: { ...history.snapshots[history.revisions.at(-1)!.id]!, revision: refreshed.cursor } },
      },
    })
    let onEvent: ((event: StreamEvent) => void) | undefined
    const port = liveTestPort((_caseId, _since, eventHandler) => {
      onEvent = eventHandler
      return () => undefined
    })
    const disconnect = connectLive(port, store, history.caseId, raw, {})
    const later: XSnapshot = {
      ...raw.at(-1)!,
      cursor: '9.0000000002',
      timestamp: new Date(Date.parse(refreshed.timestamp) + 1000).toISOString(),
      summary: { ...raw.at(-1)!.summary, phase: 'later source event' },
    }
    onEvent!({ event_type: 'snapshot', cursor: later.cursor, timestamp: later.timestamp, payload: later })

    expect(store.getState().history.snapshots[refreshed.cursor]).toBeDefined()
    expect(store.getState().history.snapshots[later.cursor]).toBeDefined()
    expect(store.getState().history.revisions.at(-1)?.id).toBe(later.cursor)
    disconnect()
  })
})

describe('connectLive initial execution standing', () => {
  const noop = () => () => undefined
  // The northstar fixture holds no runs; give its head one, the way the live gateway projects a run.
  const withRun = (raw: XSnapshot[]): XSnapshot[] => {
    const head = raw.at(-1)!
    const parent = head.visible_objects[0]!.id
    const run = { id: 'run_live_1', kind: 'execution_trace', name: 'run_live_1', status: 'WAITING_ON_OTHERS', badge: 'Executed', salience: 0.3, parent_id: parent, actions: [] }
    return [...raw.slice(0, -1), { ...head, visible_objects: [...head.visible_objects, run as unknown as XSnapshot['visible_objects'][number]] }]
  }
  it('shows the runs a live world already holds on connect, with no stream event', () => {
    const raw = withRun(buildNorthstarHistory({ actor_id: 'operator-1', role: 'case_architect' }))
    const live = projectHistory(raw, {}, 'go', { withCore: true })
    const store = createStore(initialState(live))
    expect(Object.keys(store.getState().executions)).toHaveLength(0)
    const disconnect = connectLive(liveTestPort(noop), store, live.caseId, raw, {})
    const runs = Object.values(live.snapshots[live.revisions.at(-1)!.id]!.objects).filter((o) => o.kind === 'run' && o.parent)
    expect(runs.length).toBeGreaterThan(0)
    expect(Object.keys(store.getState().executions).length).toBe(new Set(runs.map((r) => r.parent)).size)
    expect(Object.values(store.getState().executions)[0]!.state).toBe('executed')
    disconnect()
  })
  it('leaves the local fixture event-driven: nothing is shown until a stream event arrives', () => {
    const raw = withRun(buildNorthstarHistory({ actor_id: 'operator-1', role: 'case_architect' }))
    const local = projectHistory(raw, {}, 'local-contract', { withCore: true })
    const store = createStore(initialState(local))
    const disconnect = connectLive(liveTestPort(noop), store, local.caseId, raw, {})
    expect(Object.keys(store.getState().executions)).toHaveLength(0)
    disconnect()
  })
})

describe('executionStateOf', () => {
  it('lets an authoritative settlement decide, in any provenance', () => {
    expect(executionStateOf('WAITING', { decision: 'ACCEPTED' }, 'local-contract')).toBe('settled')
    expect(executionStateOf('COMPLETED', { decision: 'REJECTED' }, 'go')).toBe('rejected')
  })
  it('keeps the fixture rule: completed without a settlement is executed, never settled', () => {
    expect(executionStateOf('COMPLETED', undefined, 'local-contract')).toBe('executed')
    expect(executionStateOf('IN_PROGRESS', undefined, 'local-contract')).toBe('running')
  })
  it('reads a live run by its typed status: executed-unsettled is not settled', () => {
    expect(executionStateOf('IN_PROGRESS', undefined, 'go')).toBe('running')
    expect(executionStateOf('WAITING_ON_OTHERS', undefined, 'go')).toBe('executed')
    expect(executionStateOf('ACTION_REQUIRED', undefined, 'go')).toBe('executed')
    expect(executionStateOf('COMPLETED', undefined, 'go')).toBe('settled')
    expect(executionStateOf('REJECTED', undefined, 'go')).toBe('rejected')
  })
  it('never shows a failed or terminated live run as running', () => {
    expect(executionStateOf('FAILED', undefined, 'go')).toBe('failed')
  })
})

describe('connectLive idle recovery', () => {
  const setup = (getSnapshot: CaseworkPort['getSnapshot']) => {
    const raw = buildNorthstarHistory({ actor_id: 'operator-1', role: 'case_architect' })
    const history = projectHistory(raw, {}, 'local-contract', { withCore: true })
    const store = createStore(initialState(history))
    let onError: ((error: Error) => void) | undefined
    let onOpen: (() => void) | undefined
    let onEvent: ((event: StreamEvent) => void) | undefined
    const port: CaseworkPort = {
      ...liveTestPort((_c, _s, eventHandler, errorHandler, openHandler) => {
        onEvent = eventHandler
        onError = errorHandler
        onOpen = openHandler
        return () => undefined
      }),
      getSnapshot,
    }
    const disconnect = connectLive(port, store, history.caseId, raw, {}, { interruptedAfterMs: 1000 })
    return { raw, store, disconnect, error: () => onError!(new Error('stream disconnected')), open: () => onOpen!(), event: (e: StreamEvent) => onEvent!(e) }
  }

  it('an outage in which nothing was missed recovers once the stream reopens and the source answers a read', async () => {
    const t = setup(async () => t.raw.at(-1)!)
    t.error()
    expect(t.store.getState().connection).toBe('reconnecting')
    const revisions = t.store.getState().history.revisions.length
    t.open()
    await wait(10)
    expect(t.store.getState().connection).toBe('live')
    expect(t.store.getState().history.revisions.length).toBe(revisions)
    t.disconnect()
  })

  it('an opened stream alone does not recover while the source is unreachable; a control frame does not either', async () => {
    const t = setup(async () => {
      throw new Error('source unreachable')
    })
    t.error()
    t.open()
    t.event({ event_type: 'heartbeat', cursor: '', timestamp: '', payload: {} })
    await wait(10)
    expect(t.store.getState().connection).toBe('reconnecting')
    t.disconnect()
  })
})
