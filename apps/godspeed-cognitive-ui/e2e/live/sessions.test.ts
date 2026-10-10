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
