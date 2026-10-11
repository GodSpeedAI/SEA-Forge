import type { ReactElement } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { buildNorthstarHistory, NORTHSTAR_TRAJECTORY, NORTHSTAR_CASE_ID } from '../adapters/local/northstarData'
import { LocalContractAdapter } from '../adapters/local/localAdapter'
import type { Actor, UiState, WorldHistory } from '../model/types'
import type { Store } from '../model/store'
import { initialState, createStore } from '../model/store'
import { projectHistory } from '../ports/project'
import type { XSnapshot } from '../ports/contract'

// Journey test harness (T09): component-level wiring tests over the REAL local adapter and the
// REAL store, rendered with react-dom/server (no DOM, no new dependencies). Interactions go
// through the same paths the App uses — the intent path, the proposal flow, connectLive — never
// through test-only shims.

export const ACTOR: Actor = { id: 'usr-sam', role: 'case_architect', name: 'Sam Prime', kind: 'human' }
export const DEV_ACTOR: Actor = { id: 'usr-dev', role: 'developer', name: 'Dev', kind: 'human' }

export function render(node: ReactElement): string {
  return renderToStaticMarkup(node)
}

export function northstarHistory(): { history: WorldHistory; raw: XSnapshot[] } {
  const raw = buildNorthstarHistory({ actor_id: 'projection', role: 'case_architect' })
  return { history: projectHistory(raw, {}, 'local-contract', { withCore: true }), raw }
}

export function makeJourneyAdapter(speed = 1000): LocalContractAdapter {
  return new LocalContractAdapter({ speed, latency: 0 })
}

export function makeStore(history: WorldHistory): { store: Store; state: UiState } {
  const store = createStore(initialState(history))
  return { store, state: store.getState() }
}

/** The head snapshot's projected object map (what the UI's world shows at "now"). */
export function headObjects(history: WorldHistory) {
  return history.snapshots[history.revisions[history.revisions.length - 1]!.id]!.objects
}

export const tick = (ms = 30) => new Promise((r) => setTimeout(r, ms))

export { NORTHSTAR_CASE_ID, NORTHSTAR_TRAJECTORY }
