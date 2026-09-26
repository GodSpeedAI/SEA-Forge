// HttpCaseworkAdapter unit tests: the fetch/EventSource mapping logic, mocked at the network
// boundary. The live conformance run against the real gateway is separate (live-tagged; see the
// T08 evidence for the recorded run against the T07 gateway).
import { describe, expect, test } from 'bun:test'
import { HttpCaseworkAdapter, HttpRefusalError } from './httpCaseworkAdapter'

/** A fetch mock serving a scripted queue of responses; records every request. */
function mockFetch(handler: (url: string, init: RequestInit | undefined) => Response | Promise<Response>) {
  const requests: { url: string; init: RequestInit | undefined }[] = []
  const impl = async (input: string | URL | Request, init?: RequestInit): Promise<Response> => {
    const url = String(input)
    requests.push({ url, init })
    return handler(url, init)
  }
  ;(impl as typeof fetch & { requests: typeof requests }).requests = requests
  return impl as typeof fetch
}

const jsonResponse = (status: number, body: unknown) =>
  new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } })

const SESSION = { user: 'operator', actor_id: 'operator_local', role: 'operator', roles: ['operator'] }

describe('HttpCaseworkAdapter', () => {
  test('session() learns identity from /api/session and caches it', async () => {
    const fetchMock = mockFetch((url) => {
      if (url.endsWith('/api/session')) return jsonResponse(200, SESSION)
      return jsonResponse(404, {})
    })
    const original = globalThis.fetch
    globalThis.fetch = fetchMock as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      const first = await adapter.session()
      expect(first?.actor_id).toBe('operator_local')
      const second = await adapter.session()
      expect(second).toEqual(first)
      expect((fetchMock as unknown as { requests: { url: string }[] }).requests.filter((r) => r.url.endsWith('/api/session')).length).toBe(1)
    } finally {
      globalThis.fetch = original
    }
  })

  test('getSnapshot ignores client-asserted actors and stamps the server perspective', async () => {
    const snapshot = {
      cursor: '01M3',
      perspective: { actor_id: 'operator_local', role: 'operator' },
      visible_objects: [{ id: 'item_1', kind: 'case', name: 'A case', status: 'ACTIVE', badge: '', salience: 1, actions: [] }],
    }
    const fetchMock = mockFetch((url) => {
      if (url.includes('/api/world')) {
        expect(url).toContain('case_id=case_1')
        expect(url).not.toContain('actor=')
        return jsonResponse(200, { snapshot })
      }
      return jsonResponse(404, {})
    })
    const original = globalThis.fetch
    globalThis.fetch = fetchMock as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      const snap = await adapter.getSnapshot('case_1', 'mallory', 'security_officer' as never)
      expect(snap.perspective.actor_id).toBe('operator_local')
      expect(snap.visible_objects[0]!.id).toBe('item_1')
    } finally {
      globalThis.fetch = original
    }
  })

  test('dispatchIntent sends the CSRF header and surfaces typed refusals verbatim', async () => {
    const doc = { cookie: 'casework_csrf=test-token-123' }
    ;(globalThis as { document?: unknown }).document = doc
    const fetchMock = mockFetch((_url, init) => {
      expect((init?.headers as Record<string, string>)['X-CSRF-Token']).toBe('test-token-123')
      return jsonResponse(200, {
        intent_id: 'i-1',
        success: false,
        refusal: { refusal_kind: 'STALE_PROJECTION', message: 'The case changed since this view was loaded.' },
      })
    })
    const original = globalThis.fetch
    globalThis.fetch = fetchMock as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      const response = await adapter.dispatchIntent({
        intent_id: 'i-1',
        kind: 'CONSEQUENTIAL_CASE',
        action_name: 'EXECUTE_ITEM',
        target_object_id: 'item_1',
        case_id: 'case_1',
        client_cursor: 'old',
        actor: { actor_id: 'operator_local', role: 'operator' },
        parameters: { item_id: 'item_1' },
      } as never)
      expect(response.success).toBe(false)
      expect(response.refusal?.refusal_kind).toBe('STALE_PROJECTION')
    } finally {
      globalThis.fetch = original
      delete (globalThis as { document?: unknown }).document
    }
  })

  test('pre-dispatch gateway refusals (rate limit) throw typed, never retry', async () => {
    ;(globalThis as { document?: unknown }).document = { cookie: 'casework_csrf=t' }
    const fetchMock = mockFetch(() => jsonResponse(429, { error: 'rate limited', error_class: 'rate_limited' }))
    const original = globalThis.fetch
    globalThis.fetch = fetchMock as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      let caught: unknown
      try {
        await adapter.dispatchIntent({
          intent_id: 'i-2',
          kind: 'CONSEQUENTIAL_CASE',
          action_name: 'EXECUTE_ITEM',
          target_object_id: 'x',
          case_id: 'c',
          client_cursor: '',
          actor: { actor_id: 'a', role: 'operator' },
          parameters: {},
        } as never)
      } catch (err) {
        caught = err
      }
      expect(caught).toBeInstanceOf(HttpRefusalError)
      expect((caught as HttpRefusalError).refusalKind).toBe('rate_limited')
      expect((fetchMock as unknown as { requests: { url: string }[] }).requests.length).toBe(1)
    } finally {
      globalThis.fetch = original
      delete (globalThis as { document?: unknown }).document
    }
  })

  test('unauthenticated reads surface a sign-in refusal, not a leak', async () => {
    const fetchMock = mockFetch(() => jsonResponse(401, { error: 'authentication required' }))
    const original = globalThis.fetch
    globalThis.fetch = fetchMock as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      let caught: unknown
      try {
        await adapter.getSnapshot('case_1', 'x', 'operator' as never)
      } catch (err) {
        caught = err
      }
      expect(caught).toBeInstanceOf(HttpRefusalError)
      expect((caught as HttpRefusalError).message).toContain('Sign in')
    } finally {
      globalThis.fetch = original
    }
  })

  test('subscribeEvents filters other cases and forwards same-case revisions', () => {
    const events: MessageEvent[] = []
    class FakeEventSource {
      static instances: FakeEventSource[] = []
      url: string
      withCredentials: boolean
      onerror: ((ev: unknown) => void) | null = null
      constructor(url: string, opts?: { withCredentials?: boolean }) {
        this.url = url
        this.withCredentials = !!opts?.withCredentials
        FakeEventSource.instances.push(this)
      }
      addEventListener(type: string, cb: (ev: MessageEvent) => void) {
        if (type === 'snapshot') (this as unknown as { snapshotCb: (ev: MessageEvent) => void }).snapshotCb = cb
      }
      close() {}
      emit(data: Record<string, unknown>, lastEventId: string) {
        events.push(new MessageEvent('snapshot', { data: JSON.stringify(data), lastEventId }))
        ;(this as unknown as { snapshotCb: (ev: MessageEvent) => void }).snapshotCb(events[events.length - 1]!)
      }
    }
    const original = globalThis.EventSource
    globalThis.EventSource = FakeEventSource as unknown as typeof EventSource
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      const received: unknown[] = []
      const unsubscribe = adapter.subscribeEvents('case_a', '01M0', (e) => received.push(e), () => {})
      const es = FakeEventSource.instances[0]!
      expect(es.url).toContain('/api/events?last=01M0')
      expect(es.withCredentials).toBe(true)
      es.emit({ event_type: 'snapshot', cursor: '01M1', payload: { case_id: 'case_b' } }, '01M1')
      es.emit({ event_type: 'snapshot', cursor: '01M2', payload: { case_id: 'case_a' } }, '01M2')
      expect(received.length).toBe(1)
      expect((received[0] as { cursor: string }).cursor).toBe('01M2')
      unsubscribe()
    } finally {
      globalThis.EventSource = original
    }
  })

  test('queryTemporalTrajectory is honestly unsupported', async () => {
    const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
    let message = ''
    try {
      await adapter.queryTemporalTrajectory('case_1')
    } catch (err) {
      message = (err as Error).message
    }
    expect(message).toContain('not served by this deployment')
  })
})
