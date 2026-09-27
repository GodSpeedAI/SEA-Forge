import { useState } from 'react'
import type { JSX } from 'react'
import './login.css'

// The login screen (T09): shown only while the session port reports no identity. It never
// asserts identity itself — credentials go to the adapter, which forwards them to the gateway
// (T07); what comes back is the verified session the whole UI then acts as.

export interface LoginScreenProps {
  /** Deployment label, e.g. the gateway origin or the local-adapter honesty note. */
  sourceLabel: string
  busy?: boolean
  error?: string
  onLogin(username: string, password: string): void
}

export function LoginScreen(p: LoginScreenProps): JSX.Element {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [local, setLocal] = useState('')
  const submit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!username.trim() || !password) {
      setLocal('Enter a username and a password.')
      return
    }
    setLocal('')
    p.onLogin(username.trim(), password)
  }
  const error = p.error ?? local
  return (
    <div className="login-screen" data-testid="login-screen" role="main" aria-label="Sign in">
      <form className="login-card" onSubmit={submit}>
        <h1 className="login-title">GodSpeed</h1>
        <p className="login-subtitle">Casework in context. Sign in to continue.</p>
        <label className="login-field">
          <span>Username</span>
          <input
            type="text"
            data-testid="login-username"
            autoComplete="username"
            value={username}
            onChange={(e) => setUsername(e.currentTarget.value)}
            disabled={p.busy}
          />
        </label>
        <label className="login-field">
          <span>Password</span>
          <input
            type="password"
            data-testid="login-password"
            autoComplete="current-password"
            value={password}
            onChange={(e) => setPassword(e.currentTarget.value)}
            disabled={p.busy}
          />
        </label>
        {error && (
          <div className="login-error" role="alert" data-testid="login-error">
            {error}
          </div>
        )}
        <button type="submit" className="login-submit" data-testid="login-submit" disabled={p.busy}>
          {p.busy ? 'Signing in…' : 'Sign in'}
        </button>
        <p className="login-source">{p.sourceLabel}</p>
      </form>
    </div>
  )
}
