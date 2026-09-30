import { describe, expect, it } from 'bun:test'
import { buildNorthstarHistory } from '../adapters/local/northstarData'
import { createStore, initialState, nowRevision } from '../model/store'
import { projectHistory } from '../ports/project'
import type { CaseworkPort, StreamEvent, XSnapshot } from '../ports/contract'
import { connectLive } from './live'

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

describe('connectLive connection recovery', () => {
  it('marks a prolonged outage interrupted and returns live only after a source snapshot', async () => {
    const raw = buildNorthstarHistory({ actor_id: 'operator-1', role: 'case_architect' })
    const history = projectHistory(raw, {}, 'local-contract', { withCore: true })
    const store = createStore(initialState(history))
    let onEvent: ((event: StreamEvent) => void) | undefined
    let onError: ((error: Error) => void) | undefined
    const port = {
      subscribeEvents(
        _caseId: string,
        _since: string | undefined,
        eventHandler: (event: StreamEvent) => void,
        errorHandler: (error: Error) => void,
      ) {
        onEvent = eventHandler
        onError = errorHandler
        return () => undefined
      },
    } as unknown as Pick<CaseworkPort, 'subscribeEvents'>
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
    const port = {
      subscribeEvents(_caseId: string, _since: string | undefined, _onEvent: (event: StreamEvent) => void, errorHandler: (error: Error) => void) {
        onError = errorHandler
        return () => undefined
      },
    } as unknown as Pick<CaseworkPort, 'subscribeEvents'>
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
    const port = {
      subscribeEvents(_caseId: string, _since: string | undefined, eventHandler: (event: StreamEvent) => void) {
        onEvent = eventHandler
        return () => undefined
      },
    } as unknown as Pick<CaseworkPort, 'subscribeEvents'>
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
