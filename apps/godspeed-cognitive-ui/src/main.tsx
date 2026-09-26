import { createRoot } from 'react-dom/client'
import { App } from './app/App'
import type { Actor } from './model/types'
import { createLocalAgent } from './narrative/localAgent'
import { loadCaseHistory } from './ports/project'
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
// Test/recovery switches (local source): ?speed=N, ?failArtifact=<ref>, ?corruptArtifact=<ref>,
// ?agent=off, ?agentFailAfter=N, ?failRenderer=<kind>.
const params = new URLSearchParams(location.search)
const list = (k: string) => params.getAll(k).flatMap((v) => v.split(',')).filter(Boolean)

// Vite replaces this at build time; `import.meta.env.DEV` is true for dev servers and false for
// production builds. A production build CANNOT select the local source (the plan's guardrail:
// fixtures stay only as test doubles).
const requestedSource = import.meta.env.VITE_CASEWORK_SOURCE ?? (import.meta.env.DEV ? 'local' : 'live')
const source = requestedSource === 'live' || requestedSource === 'local' ? requestedSource : 'local'

let portPromise: Promise<import('./ports/contract').CaseworkPort>
if (source === 'live') {
  const { HttpCaseworkAdapter } = await import('./adapters/http/httpCaseworkAdapter')
  portPromise = Promise.resolve(new HttpCaseworkAdapter())
} else {
  const { LocalContractAdapter } = await import('./adapters/local/localAdapter')
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

const human: Actor = { id: 'usr-sam', role: 'case_architect', name: 'Sam P.', kind: 'human' }
const agentActor: Actor = { id: 'agt-guide', role: 'agent_operator', name: 'GodSpeed agent', kind: 'agent' }

const caseId =
  params.get('case') ??
  (source === 'live' ? '' : (await import('./adapters/local/northstarData')).NORTHSTAR_CASE_ID)
const { history, raw } = await loadCaseHistory(port, caseId, { actor_id: human.id, role: 'case_architect', display_name: human.name, kind: 'human' }, 'local-contract', { withCore: true })
const summaries = Object.fromEntries(history.revisions.map((r) => [r.id, r.summary ?? '']))

createRoot(document.getElementById('root')!).render(
  <App port={port} agent={agent} history={history} raw={raw} summaries={summaries} human={human} agentActor={agentActor} />,
)
