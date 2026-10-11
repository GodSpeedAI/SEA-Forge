// Tooth (2) support: shared-session detection for the two-principal journeys.
//
// L5 only means something if the operator and the R-SO are different authenticated principals in
// different browser sessions. A shared agent-browser session leaks the cookie jar from one user to
// the other, so "the R-SO approved" could silently be "the operator approved again". This pure
// logic compares what each side observes and names every way the two collapse into one.

import { createHash } from 'node:crypto'

export interface SessionFingerprint {
  /** Which principal this side is meant to be ("operator", "rso"). */
  label: string
  /** The agent-browser --session name driving it. */
  agentSession: string
  /** sha256 of the casework_session cookie value, or null before login / when it cannot be read. */
  cookieDigest: string | null
  /** actor_id the gateway reports for that cookie (GET /api/session), or null when signed out. */
  actorId: string | null
}

/**
 * Every reason the two sides are not two independent sessions. Empty means they are distinct as far
 * as the checks that CAN run can tell (fields still null are skipped, never treated as distinct
 * evidence of isolation; callers run this again after both logins).
 */
export function sharedSessionFindings(a: SessionFingerprint, b: SessionFingerprint): string[] {
  const out: string[] = []
  if (a.agentSession === b.agentSession) {
    out.push(`shared session identity: "${a.label}" and "${b.label}" drive the same agent-browser session "${a.agentSession}"`)
  }
  if (a.cookieDigest !== null && a.cookieDigest === b.cookieDigest) {
    out.push(`shared session identity: "${a.label}" and "${b.label}" present the same casework_session cookie`)
  }
  if (a.actorId !== null && a.actorId === b.actorId) {
    out.push(`shared session identity: "${a.label}" and "${b.label}" are both signed in as ${a.actorId}`)
  }
  return out
}

/** The casework_session value out of `agent-browser cookies get` output (any of its text/JSON layouts). */
export function sessionCookieValue(raw: string): string | null {
  const m = /casework_session["'\s:=,]+(?:value["'\s:=,]+)?["']?([A-Za-z0-9._~+/=-]{8,})/.exec(raw)
  return m ? m[1]! : null
}

/** Cookie values are credentials: only a digest ever leaves this module. */
export function digestOf(value: string | null): string | null {
  return value === null ? null : createHash('sha256').update(value).digest('hex').slice(0, 16)
}

/** Arguments for `agent-browser cookies set` that put a session cookie into another session (tooth only). */
export function cookieSetArgs(value: string, base: string): string[] {
  return ['cookies', 'set', 'casework_session', value, '--url', base, '--httpOnly', '--sameSite', 'Lax']
}

interface StepLike {
  name: string
  ok: boolean
  error?: string
}
interface ResultLike {
  journey_id: string
  status: string
  steps: StepLike[]
}

/** The post-login guard step in L5; the shared-cookie tooth must fail exactly here. */
export const POST_LOGIN_GUARD_STEP = 'guard: after both sign in'

/**
 * Tooth (2b), judged on results.json: distinct agent-browser session names, but the R-SO session was
 * handed the operator's cookie. L5 MUST fail at the post-login guard step (the last recorded step,
 * every earlier step green), and the message must name BOTH collapse signals the guard computes from
 * what the browsers hold (same casework_session cookie digest, same actor_id), proving it failed for
 * the right reason and not at a login or a timeout. Returns null when the tooth bit, else why not.
 */
export function sharedCookieToothProblem(results: ResultLike[]): string | null {
  const l5 = results.find((r) => r.journey_id === 'L5')
  if (!l5) return 'L5 did not run'
  const failed = l5.steps.find((s) => !s.ok)
  if (l5.status !== 'FAIL' || !failed) return `L5 status=${l5.status} with no failed step`
  if (!failed.name.startsWith(POST_LOGIN_GUARD_STEP)) return `L5 failed at "${failed.name}" (${failed.error}), not at the post-login guard`
  if (l5.steps[l5.steps.length - 1] !== failed || l5.steps.filter((s) => !s.ok).length !== 1) return 'L5 recorded steps after the guard failure or more than one failure'
  const err = failed.error ?? ''
  if (!/shared session identity/.test(err)) return `guard message lacks "shared session identity": ${err}`
  if (!/present the same casework_session cookie/.test(err)) return `guard message does not name the shared cookie: ${err}`
  if (!/both signed in as operator_local/.test(err)) return `guard message does not name the shared actor: ${err}`
  if (/drive the same agent-browser session/.test(err)) return `the sessions were not distinct (name equality fired): ${err}`
  return null
}
