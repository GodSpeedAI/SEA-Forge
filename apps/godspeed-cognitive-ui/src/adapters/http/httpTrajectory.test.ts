import { describe, expect, test } from 'bun:test'
import { HttpCaseworkAdapter, HttpRefusalError } from './httpCaseworkAdapter'

const jsonResponse = (status: number, body: unknown) =>
  new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } })

const SESSION = { user: 'operator', actor_id: 'operator_local', role: 'operator', roles: ['operator'] }
const VALID_POINT = {
  cursor: '01AAA', timestamp: '2026-09-29T12:00:00Z', event_type: 'case.submitted',
  summary: 'case.submitted', actor_id: '', actor_role: '', consequential: true,
  completed_plan_items_count: 0, total_plan_items_count: 2,
}
const VALID_TRAJECTORY = {
  case_id: 'case_requested', base_cursor: '01AAA', head_cursor: '01AAA', points: [VALID_POINT],
}

function withPoint(point: Record<string, unknown>) {
  return { ...VALID_TRAJECTORY, points: [point] }
}

describe('HttpCaseworkAdapter temporal trajectory', () => {
  test('requests the authenticated case trajectory and returns the canonical response', async () => {
    const trajectory = {
      case_id: 'case one',
      base_cursor: '01AAA',
      head_cursor: '01AAA',
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

  test('rejects every missing or incorrectly typed required trajectory field', async () => {
    const invalidResponses: Array<{ field: string; response: unknown }> = []
    const responseFields = ['case_id', 'base_cursor', 'head_cursor', 'points']
    for (const field of responseFields) {
      const omitted = { ...VALID_TRAJECTORY } as Record<string, unknown>
      delete omitted[field]
      invalidResponses.push({ field: `${field} missing`, response: omitted })
      invalidResponses.push({ field: `${field} wrong type`, response: { ...VALID_TRAJECTORY, [field]: null } })
    }

    const checkpointFields = [
      'cursor', 'timestamp', 'event_type', 'summary', 'actor_id', 'actor_role',
      'consequential', 'completed_plan_items_count', 'total_plan_items_count',
    ]
    for (const field of checkpointFields) {
      const omitted = { ...VALID_POINT } as Record<string, unknown>
      delete omitted[field]
      invalidResponses.push({ field: `point.${field} missing`, response: withPoint(omitted) })
      invalidResponses.push({
        field: `point.${field} wrong type`,
        response: withPoint({ ...VALID_POINT, [field]: field.endsWith('_count') ? '0' : null }),
      })
    }
    invalidResponses.push(
      { field: 'points empty', response: { ...VALID_TRAJECTORY, points: [] } },
      { field: 'base cursor does not match first point', response: { ...VALID_TRAJECTORY, base_cursor: '01BBB' } },
      { field: 'head cursor does not match last point', response: { ...VALID_TRAJECTORY, head_cursor: '01BBB' } },
      { field: 'point.active_stage_id wrong type', response: withPoint({ ...VALID_POINT, active_stage_id: null }) },
      { field: 'completed count negative', response: withPoint({ ...VALID_POINT, completed_plan_items_count: -1 }) },
      { field: 'total count fractional', response: withPoint({ ...VALID_POINT, total_plan_items_count: 1.5 }) },
      { field: 'completed count exceeds total', response: withPoint({ ...VALID_POINT, completed_plan_items_count: 3 }) },
    )

    const original = globalThis.fetch
    for (const { field, response } of invalidResponses) {
      globalThis.fetch = (async (input: string | URL | Request) =>
        String(input).endsWith('/api/session')
          ? jsonResponse(200, SESSION)
          : jsonResponse(200, response)) as typeof fetch
      try {
        const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
        let caught: unknown
        try {
          await adapter.queryTemporalTrajectory('case_requested')
        } catch (error) {
          caught = error
        }
        if (!(caught instanceof HttpRefusalError) || caught.refusalKind !== 'INVALID') {
          throw new Error(`expected INVALID refusal for ${field}`)
        }
      } finally {
        globalThis.fetch = original
      }
    }
  })

  test('accepts empty unknown actor attribution and an omitted optional active stage', async () => {
    const original = globalThis.fetch
    globalThis.fetch = (async (input: string | URL | Request) =>
      String(input).endsWith('/api/session')
        ? jsonResponse(200, SESSION)
        : jsonResponse(200, VALID_TRAJECTORY)) as typeof fetch
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      expect(await adapter.queryTemporalTrajectory('case_requested')).toEqual(VALID_TRAJECTORY)
    } finally {
      globalThis.fetch = original
    }
  })
})
