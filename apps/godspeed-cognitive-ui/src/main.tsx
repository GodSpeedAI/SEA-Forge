import { useEffect, useState } from 'react'
import { createRoot } from 'react-dom/client'
import { App } from './app/App'
import type { Actor } from './model/types'
import { createLocalAgent } from './narrative/localAgent'
import { loadCaseHistory } from './ports/project'
import type { CaseworkPort, SessionIdentity, SessionPort } from './ports/contract'
import { LoginScreen } from './ui/LoginScreen'
import './ui/theme.css'

// GodSpeed cognitive environment. Spec: .agents/specs/cognitive-environment/GodSpeed_Cognitive_Environment_DESIGN.md (§0.1 + Appendix A).
//
// The UI talks to the world only through the contract port (src/ports/contract.ts). Two sources:
// - "live" (default for production builds, VITE_CASEWORK_SOURCE=live): the Go gateway over
//   same-origin HTTP+SSE with session identity (T07). The adapter is imported statically here;
//   the LOCAL adapter is what must stay out of production bundles.
// - "local" (dev/test, VITE_CASEWORK_SOURCE=local or a dev build): the contract-conformant local
//   adapter over authored data, behind a DYNAMIC import so the bundler tree-shakes it from
//   production assets (the T08 bundle check proves it).
//
// Session (T09): identity always comes from the port (SessionPort). The live adapter resolves it
// from the gateway session; unauthenticated sessions get the login screen. The local adapter
// self-reports a fixture identity, so dev/test boot straight into the world.
//
// Test/recovery switches (local source): ?speed=N, ?failArtifact=<ref>, ?corruptArtifact=<ref>,
// ?agent=off, ?agentFailAfter=N, ?failRenderer=<kind>.
const params = new URLSearchParams(location.search)
const list = (k: string) => params.getAll(k).flatMap((v) => v.split(',')).filter(Boolean)

// Vite replaces this at build time; `import.meta.env.DEV` is true for dev servers and false for
// production builds. A production build CANNOT select the local source (the plan's guardrail:
// fixtures stay only as test doubles).
const requestedSource = import.meta.env.VITE_CASEWORK_SOURCE ?? (import.meta.env.DEV ? 'local' : 'live')
const source = requestedSource === 'live' || requestedSource === 'local' ? requestedSource : 'local'

let portPromise: Promise<CaseworkPort>
/** The case a local session boots into (resolved with the local adapter; empty on the live path). */
let localCaseId = ''
if (source === 'live') {
  const { HttpCaseworkAdapter } = await import('./adapters/http/httpCaseworkAdapter')
  portPromise = Promise.resolve(new HttpCaseworkAdapter())
} else {
  const { LocalContractAdapter, NORTHSTAR_CASE_ID } = await import('./adapters/local/localAdapter')
  localCaseId = NORTHSTAR_CASE_ID
  portPromise = Promise.resolve(
    new LocalContractAdapter({
      speed: Number(params.get('speed') ?? 1) || 1,
      failArtifacts: list('failArtifact'),
      corruptArtifacts: list('corruptArtifact'),
    }),
  )
}
const port = await portPromise
const agentParam = params.get('agent')
const failAfter = params.get('agentFailAfter')
const agent = agentParam === 'off' ? null : createLocalAgent(failAfter !== null ? { failAfter: Number(failAfter) } : {})

const sessionPort: SessionPort | null =
  'session' in port && 'login' in port && 'logout' in port ? (port as CaseworkPort & SessionPort) : null
const sourceLabel =
  source === 'live' ? `Live gateway · ${location.origin}` : 'Local contract adapter (not the Go service)'

/** Boots the world for a resolved session: identity first, then history, then the app. */
function SessionApp({ session, onLogout }: { session: SessionIdentity; onLogout(): void }) {
  const human: Actor = {
    id: session.actor_id,
    role: session.role,
    name: session.display_name ?? session.user ?? session.actor_id,
    kind: session.kind ?? 'human',
  }
  const agentActor: Actor = { id: 'agt-guide', role: 'agent_operator', name: 'GodSpeed agent', kind: 'agent' }
  const [loaded, setLoaded] = useState<Awaited<ReturnType<typeof loadCaseHistory>> | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let alive = true
    void (async () => {
      try {
        const caseId = params.get('case') ?? (source === 'live' ? '' : localCaseId)
        const history = await loadCaseHistory(
          port,
          caseId,
          { actor_id: human.id, role: human.role as never, display_name: human.name, kind: 'human' },
          source === 'live' ? 'go' : 'local-contract',
          { withCore: true },
        )
        if (alive) setLoaded(history)
      } catch (e) {
        if (alive) setError(e instanceof Error ? e.message : String(e))
      }
    })()
    return () => {
      alive = false
    }
    // The session identity fully determines the boot (a new identity re-boots the world).
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [session.actor_id])

  if (error) {
    return (
      <div className="boot-error" role="alert">
        The world could not be loaded: {error}
      </div>
    )
  }
  if (!loaded) return null
  const summaries = Object.fromEntries(loaded.history.revisions.map((r) => [r.id, r.summary ?? '']))
  return (
    <App
      port={port}
      agent={agent}
      history={loaded.history}
      raw={loaded.raw}
      summaries={summaries}
      human={human}
      agentActor={agentActor}
      session={session}
      onLogout={onLogout}
    />
  )
}

/** Resolves the session before anything else renders: login screen while unauthenticated. */
function Root() {
  const [session, setSession] = useState<SessionIdentity | null | undefined>(undefined)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | undefined>()

  useEffect(() => {
    let alive = true
    void (sessionPort?.session() ?? Promise.resolve(null)).then((s) => {
      if (alive) setSession(s)
    })
    return () => {
      alive = false
    }
  }, [])

  if (session === undefined) return null
  if (session === null) {
    // No port session: the only way forward is authentication. Identity is never asserted here.
    if (!sessionPort) {
      return (
        <div className="boot-error" role="alert">
          This connection offers no session. Sign-in is unavailable.
        </div>
      )
    }
    const sp = sessionPort
    const submit = async (username: string, password: string) => {
      setBusy(true)
      setError(undefined)
      try {
        setSession(await sp.login(username, password))
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e))
        setBusy(false)
      }
    }
    return <LoginScreen sourceLabel={sourceLabel} busy={busy} error={error} onLogin={submit} />
  }
  if (!sessionPort) {
    return (
      <div className="boot-error" role="alert">
        The session cannot be resolved on this connection.
      </div>
    )
  }
  const sp = sessionPort
  return <SessionApp session={session} onLogout={() => void sp.logout().then(() => setSession(null))} />
}

createRoot(document.getElementById('root')!).render(<Root />)
