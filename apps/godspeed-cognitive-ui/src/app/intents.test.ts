import { describe, expect, it } from 'bun:test'
import { buildNorthstarHistory } from '../adapters/local/northstarData'
import { createStore, focusOf, initialState, nowRevision, snapshotOf } from '../model/store'
import { projectHistory } from '../ports/project'
import type { Actor, ObjectAction } from '../model/types'
import type { CaseworkPort, IntentResponse, XSnapshot } from '../ports/contract'
import { createIntentPath } from './intents'

const ACTOR: Actor = { id: 'operator-1', role: 'case_architect', name: 'Operator', kind: 'human' }

function snapshots() {
  const raw = buildNorthstarHistory({ actor_id: ACTOR.id, role: 'case_architect' })
  const history = projectHistory(raw, {}, 'local-contract', { withCore: true })
  return { raw, history }
}

function nextCursor(cursor: string): string {
  const [epoch = '0', sequence = '0'] = cursor.split('.')
  return `${epoch}.${Number(sequence) + 1}`
}

function staleResponse(): IntentResponse {
  return {
    intent_id: 'intent-stale',
    success: false,
    refusal: { refusal_kind: 'STALE_PROJECTION', message: 'This projection is stale.' },
  }
}

describe('createIntentPath stale projection recovery', () => {
  it('fetches one fresh snapshot, preserves the refusal, and keeps the current focus', async () => {
    const { raw, history } = snapshots()
    const store = createStore(initialState(history))
    const before = snapshotOf(store.getState())
    const target = Object.values(before.objects).find((o) => o.actions?.some((a) => a.consequential && a.intent !== 'ADD_DISCRETIONARY_WORK'))!
    const action = target.actions!.find((a) => a.consequential && a.intent !== 'ADD_DISCRETIONARY_WORK')! as ObjectAction
    store.dispatch({ type: 'focus', id: target.id })
    // The live UI opens judgment before submission. Historical views are read-only
    // and cannot open judgment, so keep this stale-refusal path on the live head.
    store.dispatch({ type: 'invoke', object: target.id, action })
    const previousFocus = focusOf(store.getState())

    const head = raw.at(-1)!
    const refreshed: XSnapshot = {
      ...head,
      cursor: nextCursor(head.cursor),
      timestamp: new Date(Date.parse(head.timestamp) + 1000).toISOString(),
      summary: { ...head.summary, phase: 'fresh after stale refusal' },
      visible_objects: head.visible_objects.map((o) => (o.id === target.id ? { ...o, badge: 'Refreshed standing' } : o)),
    }
    let dispatchCount = 0
    let snapshotCount = 0
    let readArgs: unknown[] = []
    const port = {
      async dispatchIntent() {
        dispatchCount++
        return staleResponse()
      },
      async getSnapshot(...args: unknown[]) {
        snapshotCount++
        readArgs = args
        return refreshed
      },
    } as unknown as Pick<CaseworkPort, 'dispatchIntent' | 'getSnapshot'>

    const result = await createIntentPath(store, port).submit(ACTOR, target.id, action, { judgment: true })

    expect(dispatchCount).toBe(1)
    expect(snapshotCount).toBe(1)
    expect(readArgs).toEqual([history.caseId, ACTOR.id, ACTOR.role])
    expect(result.state).toBe('refused')
    expect(result.code).toBe('STALE_PROJECTION')
    expect(result.note).toBe('This projection is stale.')
    expect(store.getState().intents.at(-1)?.code).toBe('STALE_PROJECTION')
    expect(store.getState().judgment?.outcome?.code).toBe('STALE_PROJECTION')
    expect(store.getState().history.snapshots[refreshed.cursor]?.objects[target.id]?.status?.label).toBe('Refreshed standing')
    expect(nowRevision(store.getState().history)).toBe(refreshed.cursor)
    expect(store.getState().revision).toBe(refreshed.cursor)
    expect(focusOf(store.getState())).toBe(previousFocus)
  })

  it('keeps the stale refusal and reports a failed refresh without replaying the intent', async () => {
    const { history } = snapshots()
    const store = createStore(initialState(history))
    const target = Object.values(snapshotOf(store.getState()).objects).find((o) => o.actions?.some((a) => a.consequential && a.intent !== 'ADD_DISCRETIONARY_WORK'))!
    const action = target.actions!.find((a) => a.consequential && a.intent !== 'ADD_DISCRETIONARY_WORK')! as ObjectAction
    store.dispatch({ type: 'invoke', object: target.id, action })
    let dispatchCount = 0
    let snapshotCount = 0
    const port = {
      async dispatchIntent() {
        dispatchCount++
        return staleResponse()
      },
      async getSnapshot() {
        snapshotCount++
        throw new Error('gateway unavailable')
      },
    } as unknown as Pick<CaseworkPort, 'dispatchIntent' | 'getSnapshot'>

    const result = await createIntentPath(store, port).submit(ACTOR, target.id, action, { judgment: true })

    expect(dispatchCount).toBe(1)
    expect(snapshotCount).toBe(1)
    expect(result.state).toBe('refused')
    expect(result.code).toBe('STALE_PROJECTION')
    expect(result.note).toContain('This projection is stale.')
    expect(result.note).toContain('Current projection refresh failed: gateway unavailable')
    expect(store.getState().judgment?.outcome?.code).toBe('STALE_PROJECTION')
    expect(store.getState().judgment?.outcome?.note).toContain('Current projection refresh failed: gateway unavailable')
  })

  it('does not merge a deferred old-case refresh after the user switches cases', async () => {
    const { raw, history } = snapshots()
    const store = createStore(initialState(history))
    const target = Object.values(snapshotOf(store.getState()).objects).find((o) => o.actions?.some((a) => a.consequential))!
    const action = target.actions!.find((a) => a.consequential)! as ObjectAction
    const head = raw.at(-1)!
    const refreshed: XSnapshot = {
      ...head,
      cursor: nextCursor(head.cursor),
      timestamp: new Date(Date.parse(head.timestamp) + 1000).toISOString(),
    }
    let resolveRefresh!: (snapshot: XSnapshot) => void
    let signalRefreshStarted!: () => void
    const refreshStarted = new Promise<void>((resolve) => { signalRefreshStarted = resolve })
    const deferredRefresh = new Promise<XSnapshot>((resolve) => { resolveRefresh = resolve })
    let dispatchCount = 0
    let snapshotCount = 0
    const port = {
      async dispatchIntent() {
        dispatchCount++
        return staleResponse()
      },
      getSnapshot() {
        snapshotCount++
        signalRefreshStarted()
        return deferredRefresh
      },
    } as unknown as Pick<CaseworkPort, 'dispatchIntent' | 'getSnapshot'>

    const submitted = createIntentPath(store, port).submit(ACTOR, target.id, action, { judgment: true })
    await refreshStarted
    store.dispatch({ type: 'loadHistory', history: { ...history, caseId: 'other-case' } })
    resolveRefresh(refreshed)
    const result = await submitted

    expect(dispatchCount).toBe(1)
    expect(snapshotCount).toBe(1)
    expect(result.state).toBe('refused')
    expect(result.code).toBe('STALE_PROJECTION')
    expect(result.note).toBe('This projection is stale.')
    expect(store.getState().history.caseId).toBe('other-case')
    expect(store.getState().history.snapshots[refreshed.cursor]).toBeUndefined()
  })

  it('rejects a refresh response that identifies a different case', async () => {
    const { raw, history } = snapshots()
    const store = createStore(initialState(history))
    const target = Object.values(snapshotOf(store.getState()).objects).find((o) => o.actions?.some((a) => a.consequential))!
    const action = target.actions!.find((a) => a.consequential)! as ObjectAction
    const head = raw.at(-1)!
    const wrongCase: XSnapshot = {
      ...head,
      case_id: 'other-case',
      cursor: nextCursor(head.cursor),
      timestamp: new Date(Date.parse(head.timestamp) + 1000).toISOString(),
    }
    let dispatchCount = 0
    let snapshotCount = 0
    const port = {
      async dispatchIntent() {
        dispatchCount++
        return staleResponse()
      },
      async getSnapshot() {
        snapshotCount++
        return wrongCase
      },
    } as unknown as Pick<CaseworkPort, 'dispatchIntent' | 'getSnapshot'>

    const result = await createIntentPath(store, port).submit(ACTOR, target.id, action, { judgment: true })

    expect(dispatchCount).toBe(1)
    expect(snapshotCount).toBe(1)
    expect(result.state).toBe('refused')
    expect(result.code).toBe('STALE_PROJECTION')
    expect(result.note).toContain('This projection is stale.')
    expect(result.note).toContain('Current projection refresh failed: snapshot refresh returned case other-case')
    expect(store.getState().history.snapshots[wrongCase.cursor]).toBeUndefined()
  })

  it('rejects a refresh response with a missing case identity', async () => {
    const { raw, history } = snapshots()
    const store = createStore(initialState(history))
    const target = Object.values(snapshotOf(store.getState()).objects).find((o) => o.actions?.some((a) => a.consequential))!
    const action = target.actions!.find((a) => a.consequential)! as ObjectAction
    const head = raw.at(-1)!
    const unscoped = {
      ...head,
      cursor: nextCursor(head.cursor),
      timestamp: new Date(Date.parse(head.timestamp) + 1000).toISOString(),
    } as XSnapshot
    delete (unscoped as Partial<XSnapshot>).case_id
    let dispatchCount = 0
    let snapshotCount = 0
    const port = {
      async dispatchIntent() {
        dispatchCount++
        return staleResponse()
      },
      async getSnapshot() {
        snapshotCount++
        return unscoped
      },
    } as unknown as Pick<CaseworkPort, 'dispatchIntent' | 'getSnapshot'>

    const result = await createIntentPath(store, port).submit(ACTOR, target.id, action, { judgment: true })

    expect(dispatchCount).toBe(1)
    expect(snapshotCount).toBe(1)
    expect(result.state).toBe('refused')
    expect(result.code).toBe('STALE_PROJECTION')
    expect(result.note).toContain('This projection is stale.')
    expect(result.note).toContain('Current projection refresh failed: snapshot refresh returned case')
    expect(store.getState().history.snapshots[unscoped.cursor]).toBeUndefined()
  })

  it('rejects a refresh response with a non-string case identity', async () => {
    const { raw, history } = snapshots()
    const store = createStore(initialState(history))
    const target = Object.values(snapshotOf(store.getState()).objects).find((o) => o.actions?.some((a) => a.consequential))!
    const action = target.actions!.find((a) => a.consequential)! as ObjectAction
    const head = raw.at(-1)!
    const malformed = {
      ...head,
      case_id: null,
      cursor: nextCursor(head.cursor),
      timestamp: new Date(Date.parse(head.timestamp) + 1000).toISOString(),
    } as unknown as XSnapshot
    let dispatchCount = 0
    let snapshotCount = 0
    const port = {
      async dispatchIntent() {
        dispatchCount++
        return staleResponse()
      },
      async getSnapshot() {
        snapshotCount++
        return malformed
      },
    } as unknown as Pick<CaseworkPort, 'dispatchIntent' | 'getSnapshot'>

    const result = await createIntentPath(store, port).submit(ACTOR, target.id, action, { judgment: true })

    expect(dispatchCount).toBe(1)
    expect(snapshotCount).toBe(1)
    expect(result.state).toBe('refused')
    expect(result.code).toBe('STALE_PROJECTION')
    expect(result.note).toContain('This projection is stale.')
    expect(result.note).toContain('Current projection refresh failed: snapshot refresh returned case')
    expect(store.getState().history.snapshots[malformed.cursor]).toBeUndefined()
  })
})
