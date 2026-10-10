import { describe, expect, test } from 'bun:test'
import { digestOf, sessionCookieValue, sharedSessionFindings, type SessionFingerprint } from './sessions'

const fp = (over: Partial<SessionFingerprint>): SessionFingerprint => ({
  label: 'operator',
  agentSession: 'gs-live-x-operator',
  cookieDigest: null,
  actorId: null,
  ...over,
})

describe('sharedSessionFindings', () => {
  test('two distinct sessions, cookies and actors produce no findings', () => {
    const a = fp({ cookieDigest: 'aaaa', actorId: 'operator_local' })
    const b = fp({ label: 'rso', agentSession: 'gs-live-x-rso', cookieDigest: 'bbbb', actorId: 'rso_local' })
    expect(sharedSessionFindings(a, b)).toEqual([])
  })

  test('the same agent-browser session is a shared session even before anyone logs in', () => {
    const a = fp({})
    const b = fp({ label: 'rso' })
    const f = sharedSessionFindings(a, b)
    expect(f).toHaveLength(1)
    expect(f[0]).toMatch(/shared session identity/)
    expect(f[0]).toMatch(/same agent-browser session/)
  })

  test('different session names that hold the same cookie are still shared', () => {
    const a = fp({ cookieDigest: 'cccc' })
    const b = fp({ label: 'rso', agentSession: 'other', cookieDigest: 'cccc' })
    const f = sharedSessionFindings(a, b)
    expect(f.some((x) => /same casework_session cookie/.test(x))).toBe(true)
  })

  test('two sides signed in as one actor are shared even with different cookies', () => {
    const a = fp({ cookieDigest: 'aaaa', actorId: 'operator_local' })
    const b = fp({ label: 'rso', agentSession: 'other', cookieDigest: 'bbbb', actorId: 'operator_local' })
    const f = sharedSessionFindings(a, b)
    expect(f).toHaveLength(1)
    expect(f[0]).toMatch(/both signed in as operator_local/)
  })

  test('unknown (null) cookies and actors are never counted as equal', () => {
    const a = fp({ agentSession: 'a' })
    const b = fp({ label: 'rso', agentSession: 'b' })
    expect(sharedSessionFindings(a, b)).toEqual([])
  })

  test('every collapse is reported, not just the first', () => {
    const a = fp({ cookieDigest: 'dd', actorId: 'operator_local' })
    const b = fp({ label: 'rso', cookieDigest: 'dd', actorId: 'operator_local' })
    expect(sharedSessionFindings(a, b)).toHaveLength(3)
  })
})

describe('cookie helpers', () => {
  test('extracts the session cookie from text and JSON layouts and never returns other cookies', () => {
    expect(sessionCookieValue('casework_csrf=abcdefghijkl\ncasework_session=0123456789abcdef; HttpOnly')).toBe('0123456789abcdef')
    expect(sessionCookieValue('[{"name":"casework_session","value":"ZZZZZZZZZZZZ"}]')).toBe('ZZZZZZZZZZZZ')
    expect(sessionCookieValue('casework_csrf=abcdefghijkl')).toBeNull()
  })

  test('digests are stable, short and do not contain the value', () => {
    const d = digestOf('0123456789abcdef')!
    expect(d).toBe(digestOf('0123456789abcdef')!)
    expect(d).toHaveLength(16)
    expect(d).not.toContain('0123456789')
    expect(digestOf(null)).toBeNull()
  })
})

import { POST_LOGIN_GUARD_STEP, cookieSetArgs, sharedCookieToothProblem } from './sessions'

describe('shared-cookie tooth', () => {
  const guardMsg = 'shared session identity: "operator" and "rso" present the same casework_session cookie; shared session identity: "operator" and "rso" are both signed in as operator_local'
  const l5 = (steps: { name: string; ok: boolean; error?: string }[]) => [{ journey_id: 'L5', status: 'FAIL', steps }]
  const good = [
    { name: 'guard: the operator and the R-SO drive distinct agent-browser sessions', ok: true },
    { name: 'operator designs', ok: true },
    { name: `${POST_LOGIN_GUARD_STEP}, the two sessions hold different cookies and different principals`, ok: false, error: guardMsg },
  ]

  test('distinct names but the same cookie and actor: the guard is the right and only failure', () => {
    expect(sharedCookieToothProblem(l5(good))).toBeNull()
  })
  test('same cookie digest under different session names is a finding even before an actor is known', () => {
    const a = fp({ agentSession: 'x-operator', cookieDigest: 'aa' })
    const b = fp({ label: 'rso', agentSession: 'x-rso', cookieDigest: 'aa' })
    expect(sharedSessionFindings(a, b)).toEqual(['shared session identity: "operator" and "rso" present the same casework_session cookie'])
  })
  test('failing at the name-equality guard is the other tooth, not this one', () => {
    const steps = [{ name: 'guard: the operator and the R-SO drive distinct agent-browser sessions', ok: false, error: 'shared session identity: ... drive the same agent-browser session "x"' }]
    expect(sharedCookieToothProblem(l5(steps))).toMatch(/not at the post-login guard/)
  })
  test('failing at the guard for a different message does not count', () => {
    const steps = [...good.slice(0, 2), { ...good[2]!, error: 'principals a / b' }]
    expect(sharedCookieToothProblem(l5(steps))).toMatch(/lacks "shared session identity"/)
    const onlyCookie = [...good.slice(0, 2), { ...good[2]!, error: 'shared session identity: "operator" and "rso" present the same casework_session cookie' }]
    expect(sharedCookieToothProblem(l5(onlyCookie))).toMatch(/shared actor/)
  })
  test('steps after the failure, a pass, or a missing L5 are not a bite', () => {
    expect(sharedCookieToothProblem(l5([...good, { name: 'later', ok: true }]))).toMatch(/after the guard/)
    expect(sharedCookieToothProblem([{ journey_id: 'L5', status: 'PASS', steps: [] }])).toMatch(/no failed step/)
    expect(sharedCookieToothProblem([])).toMatch(/did not run/)
  })
  test('cookie set args carry the value only as an argument and scope it to the base url', () => {
    expect(cookieSetArgs('v'.repeat(12), 'http://127.0.0.1:4179').slice(0, 4)).toEqual(['cookies', 'set', 'casework_session', 'v'.repeat(12)])
    expect(cookieSetArgs('v'.repeat(12), 'http://127.0.0.1:4179')).toContain('http://127.0.0.1:4179')
  })
})
