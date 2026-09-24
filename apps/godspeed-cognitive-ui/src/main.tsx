import { createRoot } from 'react-dom/client'
import { LocalContractAdapter } from './adapters/local/localAdapter'
import { NORTHSTAR_CASE_ID } from './adapters/local/northstarData'
import { App } from './app/App'
import type { Actor } from './model/types'
import { createLocalAgent } from './narrative/localAgent'
import { loadCaseHistory } from './ports/project'
import './ui/theme.css'

// GodSpeed cognitive environment. Spec: .agents/specs/cognitive-environment/GodSpeed_Cognitive_Environment_DESIGN.md (§0.1 + Appendix A).
//
// The UI talks to the world only through the contract port (src/ports/contract.ts). In this phase
// the port is served by a local, contract-conformant adapter; the Go system front end is
// deliberately not connected. The status bar says "Local contract adapter".
//
// Test/recovery switches: ?speed=N (execution pace), ?failArtifact=<ref>, ?corruptArtifact=<ref>,
// ?agent=off, ?agentFailAfter=N, ?failRenderer=<kind>.
const params = new URLSearchParams(location.search)
const list = (k: string) => params.getAll(k).flatMap((v) => v.split(',')).filter(Boolean)

const port = new LocalContractAdapter({
  speed: Number(params.get('speed') ?? 1) || 1,
  failArtifacts: list('failArtifact'),
  corruptArtifacts: list('corruptArtifact'),
})
const agentParam = params.get('agent')
const failAfter = params.get('agentFailAfter')
const agent = agentParam === 'off' ? null : createLocalAgent(failAfter !== null ? { failAfter: Number(failAfter) } : {})

const human: Actor = { id: 'usr-sam', role: 'case_architect', name: 'Sam P.', kind: 'human' }
const agentActor: Actor = { id: 'agt-guide', role: 'agent_operator', name: 'GodSpeed agent', kind: 'agent' }

const caseId = params.get('case') ?? NORTHSTAR_CASE_ID
const { history, raw } = await loadCaseHistory(port, caseId, { actor_id: human.id, role: 'case_architect', display_name: human.name, kind: 'human' }, 'local-contract', { withCore: true })
const summaries = Object.fromEntries(history.revisions.map((r) => [r.id, r.summary ?? '']))

createRoot(document.getElementById('root')!).render(
  <App port={port} agent={agent} history={history} raw={raw} summaries={summaries} human={human} agentActor={agentActor} />,
)
