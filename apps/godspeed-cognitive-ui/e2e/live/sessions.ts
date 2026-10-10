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
