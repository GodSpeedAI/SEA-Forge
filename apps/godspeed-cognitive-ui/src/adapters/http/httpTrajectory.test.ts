import { describe, expect, test } from 'bun:test'
import { HttpCaseworkAdapter, HttpRefusalError } from './httpCaseworkAdapter'

const jsonResponse = (status: number, body: unknown) =>
  new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } })

const SESSION = { user: 'operator', actor_id: 'operator_local', role: 'operator', roles: ['operator'] }

describe('HttpCaseworkAdapter temporal trajectory', () => {
  test('requests the authenticated case trajectory and returns the canonical response', async () => {
    const trajectory = {
      case_id: 'case one',
      base_cursor: '01AAA',
      head_cursor: '01BBB',
      points: [{
        cursor: '01AAA', timestamp: '2026-09-29T12:00:00Z', event_type: 'case.submitted',
        summary: 'case.submitted', actor_id: '', actor_role: '', consequential: true,
        completed_plan_items_count: 0, total_plan_items_count: 2,
      }],
    }
    const calls: string[] = []
    const original = globalThis.fetch
    globalThis.fetch = (async (input: string | URL | Request) => {
      const url = String(input)
      calls.push(url)
      return url.endsWith('/api/session') ? jsonResponse(200, SESSION) : jsonResponse(200, trajectory)
    }) as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      expect(await adapter.queryTemporalTrajectory('case one')).toEqual(trajectory)
      expect(calls).toEqual([
        'http://gw.test/api/session',
        'http://gw.test/api/trajectory?case_id=case%20one',
      ])
    } finally {
      globalThis.fetch = original
    }
  })

  test('surfaces typed unknown or unavailable retained history refusals', async () => {
    const original = globalThis.fetch
    globalThis.fetch = (async (input: string | URL | Request) =>
      String(input).endsWith('/api/session')
        ? jsonResponse(200, SESSION)
        : jsonResponse(404, { error: { kind: 'not_found', note: 'no retained trajectory is available' } })) as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      let caught: unknown
      try {
        await adapter.queryTemporalTrajectory('case_1')
      } catch (error) {
        caught = error
      }
      expect(caught).toBeInstanceOf(HttpRefusalError)
      expect((caught as HttpRefusalError).refusalKind).toBe('not_found')
    } finally {
      globalThis.fetch = original
    }
  })

  test('rejects a trajectory for a different case with a typed invalid refusal', async () => {
    const original = globalThis.fetch
    globalThis.fetch = (async (input: string | URL | Request) =>
      String(input).endsWith('/api/session')
        ? jsonResponse(200, SESSION)
        : jsonResponse(200, {
            case_id: 'case_other', base_cursor: '01AAA', head_cursor: '01AAA',
            points: [{ cursor: '01AAA' }],
          })) as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      await expect(adapter.queryTemporalTrajectory('case_requested')).rejects.toMatchObject({
        name: 'HttpRefusalError',
        refusalKind: 'INVALID',
      })
    } finally {
      globalThis.fetch = original
    }
  })

  test('rejects missing trajectory points or cursors with a typed invalid refusal', async () => {
    const original = globalThis.fetch
    globalThis.fetch = (async (input: string | URL | Request) =>
      String(input).endsWith('/api/session')
        ? jsonResponse(200, SESSION)
        : jsonResponse(200, {
            case_id: 'case_requested', base_cursor: '01AAA', head_cursor: '01AAA',
            points: [{ summary: 'missing cursor' }],
          })) as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      await expect(adapter.queryTemporalTrajectory('case_requested')).rejects.toMatchObject({
        name: 'HttpRefusalError',
        refusalKind: 'INVALID',
      })
    } finally {
      globalThis.fetch = original
    }
  })
})
