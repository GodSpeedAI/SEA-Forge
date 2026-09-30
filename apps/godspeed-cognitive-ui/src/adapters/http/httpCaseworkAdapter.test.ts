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

  test('resolveArtifact returns the canonical authenticated payload with Unicode intact', async () => {
    const digest = 'sha256:' + 'a'.repeat(64)
    const payload = {
      evidence_id: 'evi-17',
      name: 'résumé.md',
      digest,
      content_type: 'text/markdown',
      content: '# Résumé ☃\n',
      provenance: { case_id: '', plan_item_id: '', invocation_id: '', run_id: 'run-17' },
    }
    const fetchMock = mockFetch((url, init) => {
      if (url.endsWith('/api/session')) return jsonResponse(200, SESSION)
      if (url.endsWith(`/api/artifacts/${encodeURIComponent(digest)}`)) {
        expect(init?.credentials).toBe('include')
        return jsonResponse(200, payload)
      }
      return jsonResponse(404, {})
    })
    const original = globalThis.fetch
    globalThis.fetch = fetchMock as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      expect(await adapter.resolveArtifact(digest)).toEqual(payload)
    } finally {
      globalThis.fetch = original
    }
  })

  test('resolveArtifact preserves an empty artifact body', async () => {
    const digest = 'sha256:' + 'b'.repeat(64)
    const payload = {
      evidence_id: 'evi-empty',
      name: 'empty.txt',
      digest,
      content_type: 'text/plain',
      content: '',
      provenance: { case_id: '', plan_item_id: '', invocation_id: '', run_id: 'run-empty' },
    }
    const fetchMock = mockFetch((url) =>
      url.endsWith('/api/session') ? jsonResponse(200, SESSION) : jsonResponse(200, payload),
    )
    const original = globalThis.fetch
    globalThis.fetch = fetchMock as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      expect((await adapter.resolveArtifact(digest)).content).toBe('')
    } finally {
      globalThis.fetch = original
    }
  })

  test('resolveArtifact surfaces typed not-found errors from the gateway', async () => {
    const digest = 'sha256:' + 'c'.repeat(64)
    const fetchMock = mockFetch((url) =>
      url.endsWith('/api/session')
        ? jsonResponse(200, SESSION)
        : jsonResponse(404, { error: { kind: 'not_found', note: 'the requested artifact was not found' } }),
    )
    const original = globalThis.fetch
    globalThis.fetch = fetchMock as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      let caught: unknown
      try {
        await adapter.resolveArtifact(digest)
      } catch (err) {
        caught = err
      }
      expect(caught).toBeInstanceOf(HttpRefusalError)
      expect((caught as HttpRefusalError).refusalKind).toBe('not_found')
      expect((fetchMock as unknown as { requests: { url: string }[] }).requests.filter((r) => r.url.includes('/api/artifacts/')).length).toBe(1)
    } finally {
      globalThis.fetch = original
    }
  })

  test('resolveArtifact rejects a response whose digest differs from the requested digest', async () => {
    const digest = 'sha256:' + 'd'.repeat(64)
    const payload = {
      evidence_id: 'evi-other',
      name: 'other.txt',
      digest: 'sha256:' + 'e'.repeat(64),
      content_type: 'text/plain',
      content: 'other bytes',
      provenance: { case_id: '', plan_item_id: '', invocation_id: '', run_id: 'run-other' },
    }
    const fetchMock = mockFetch((url) =>
      url.endsWith('/api/session') ? jsonResponse(200, SESSION) : jsonResponse(200, payload),
    )
    const original = globalThis.fetch
    globalThis.fetch = fetchMock as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      let caught: unknown
      try {
        await adapter.resolveArtifact(digest)
      } catch (err) {
        caught = err
      }
      expect(caught).toBeInstanceOf(HttpRefusalError)
      expect((caught as HttpRefusalError).refusalKind).toBe('INTEGRITY_MISMATCH')
    } finally {
      globalThis.fetch = original
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

  test('queryTemporalTrajectory calls the authenticated endpoint and preserves a typed not-found refusal', async () => {
    const fetchMock = mockFetch((url, init) => {
      if (url.endsWith('/api/session')) return jsonResponse(200, SESSION)
      if (url.endsWith('/api/trajectory?case_id=case_1')) {
        expect(init?.credentials).toBe('include')
        return jsonResponse(404, { error: { kind: 'not_found', note: 'no retained trajectory is available' } })
      }
      return jsonResponse(500, {})
    })
    const original = globalThis.fetch
    globalThis.fetch = fetchMock as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      let caught: unknown
      try {
        await adapter.queryTemporalTrajectory('case_1')
      } catch (err) {
        caught = err
      }
      expect(caught).toBeInstanceOf(HttpRefusalError)
      expect((caught as HttpRefusalError).refusalKind).toBe('not_found')
      expect((caught as Error).message).toContain('no retained trajectory is available')
      expect((fetchMock as unknown as { requests: { url: string }[] }).requests.map((request) => request.url)).toEqual([
        'http://gw.test/api/session',
        'http://gw.test/api/trajectory?case_id=case_1',
      ])
    } finally {
      globalThis.fetch = original
    }
  })
})
