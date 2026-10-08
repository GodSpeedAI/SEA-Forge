import type { JSX } from 'react'
import type { SessionIdentity } from '../ports/contract'
import './session-badge.css'

// The perspective badge (T09): who the session resolved, shown in the chrome. Identity is never
// asserted by the client — this is the standing the gateway verified (T07) or the identity the
// local adapter self-reports. The badge names the role too, so authority refusals can be read
// against it.

export interface SessionBadgeProps {
  session: SessionIdentity | null
  onLogout?(): void
}

export function SessionBadge(p: SessionBadgeProps): JSX.Element {
  const s = p.session
  if (!s) {
    return (
      <div className="session-badge session-badge--signed-out" data-testid="session-badge" data-state="signed-out">
        <span className="session-badge-role">Signed out</span>
      </div>
    )
  }
  const name = s.display_name ?? s.user ?? s.actor_id
  return (
    <div className="session-badge" data-testid="session-badge" data-state="signed-in" title={`Acting as ${s.actor_id} (${s.role})`}>
      <span className="session-badge-name">{name}</span>
      <span className="session-badge-id">{s.actor_id}</span>
      <span className="session-badge-role">{s.role}</span>
      {p.onLogout && (
        <button type="button" className="session-badge-logout" data-testid="session-logout" onClick={p.onLogout} aria-label="Sign out">
          Sign out
        </button>
      )}
    </div>
  )
}
