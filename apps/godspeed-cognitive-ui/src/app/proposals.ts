import { type Store } from '../model/store'
import type { Actor, Id } from '../model/types'
import type { CaseworkPort, InteractionIntent, TemplateSourcePort, XSnapshot } from '../ports/contract'
import { loadCaseHistory } from '../ports/project'
import { uuid } from './intents'

// The case-design proposal flow (T09, CJ04): template picker → parameters → preflight → commit.
// Extracted from App so the journey tests drive exactly the wiring the UI drives: the real port,
// the real store actions, and the submit gate (PROPOSE_CASE goes out only after a preflight for
// the CURRENT parameters passed, and carries that preflight's digest).

export interface ProposalFlowDeps {
  store: Store
  port: CaseworkPort
  /** The actor the session resolved (the commit acts as this identity). */
  actor(): Actor
  /** Loads a committed case's history and focuses it (App supplies the live wiring). */
  focusCase(caseId: Id): Promise<unknown>
}

const templateSource = (port: CaseworkPort): TemplateSourcePort | null =>
  'getTemplates' in port && 'preflightTemplate' in port ? (port as CaseworkPort & TemplateSourcePort) : null

export function createProposalFlow(deps: ProposalFlowDeps) {
  const { store, port } = deps
  const source = templateSource(port)

  /** Opens the picker and loads the template list through the port. */
  const open = async (): Promise<void> => {
    store.dispatch({ type: 'openProposals' })
    if (!source) {
      store.dispatch({ type: 'proposalsUnavailable', error: 'This connection offers no template source.' })
      return
    }
    try {
      const templates = await source.getTemplates()
      store.dispatch({ type: 'proposalsLoaded', templates })
    } catch (e) {
      store.dispatch({ type: 'proposalsUnavailable', error: e instanceof Error ? e.message : String(e) })
    }
  }

  const select = (templateRef: string | null): void => store.dispatch({ type: 'selectTemplate', templateRef })

  const setParam = (name: string, value: string): void => store.dispatch({ type: 'setProposalParam', name, value })

  /** Runs preflight for the current parameters; the digest binds the eventual commit. */
  const preflight = async (): Promise<void> => {
    const pr = store.getState().proposal
    if (!pr?.selected || !source) return
    store.dispatch({ type: 'preflightStarted' })
    try {
      const result = await source.preflightTemplate(pr.selected, { ...pr.params })
      store.dispatch({ type: 'preflightResult', result })
    } catch (e) {
      store.dispatch({ type: 'preflightResult', result: { passed: false, reasons: [e instanceof Error ? e.message : String(e)] } })
    }
  }

  /** Commits with PROPOSE_CASE. Refuses without a passing preflight; focuses the new case. */
  const submit = async (): Promise<{ accepted: boolean; caseId?: Id }> => {
    const pr = store.getState().proposal
    if (!pr?.selected || pr.submitting) return { accepted: false }
    // The gate: only a passing preflight for the current parameters unlocks the commit.
    if (!pr.preflight?.passed || !pr.preflight.digest) return { accepted: false }
    store.dispatch({ type: 'proposalSubmitStarted' })
    const actor = deps.actor()
    const intent: InteractionIntent = {
      intent_id: uuid(),
      kind: 'CONSEQUENTIAL_CASE',
      action_name: 'PROPOSE_CASE',
      target_object_id: pr.selected,
      case_id: '',
      // PROPOSE_CASE creates a case: no prior projection to be stale against.
      client_cursor: '',
      actor: { actor_id: actor.id, role: actor.role as InteractionIntent['actor']['role'] },
      parameters: { template_ref: pr.selected, params: { ...pr.params }, preflight_digest: pr.preflight.digest },
    }
    try {
      const r = await port.dispatchIntent(intent)
      if (!r.success) {
        store.dispatch({ type: 'proposalError', error: r.refusal?.message ?? r.error_message ?? 'The commit was refused.', code: r.refusal?.refusal_kind ?? r.error_code })
        return { accepted: false }
      }
      const newId = r.resulting_object?.id
      if (!newId) {
        store.dispatch({ type: 'proposalError', error: 'The commit was accepted but the kernel returned no case to open.', code: 'INVALID' })
        return { accepted: false }
      }
      store.dispatch({ type: 'proposalSubmitted', caseId: newId })
      await deps.focusCase(newId)
      store.dispatch({ type: 'closeProposals' })
      return { accepted: true, caseId: newId }
    } catch (e) {
      store.dispatch({ type: 'proposalError', error: e instanceof Error ? e.message : String(e), code: 'UNAVAILABLE' })
      return { accepted: false }
    }
  }

  const close = (): void => store.dispatch({ type: 'closeProposals' })

  return { open, close, select, setParam, preflight, submit }
}

/** Loads a case's history, focuses its case object, and returns the raw snapshots (for the
 * event-stream re-attachment the App does when the shown case changes). */
export async function loadAndFocusCase(
  port: CaseworkPort,
  store: Store,
  caseId: Id,
  actor: Actor,
): Promise<XSnapshot[]> {
  const { history, raw } = await loadCaseHistory(
    port,
    caseId,
    { actor_id: actor.id, role: actor.role as never, display_name: actor.name, kind: 'human' },
    store.getState().history.provenance,
    { withCore: true },
  )
  store.dispatch({ type: 'loadHistory', history })
  store.dispatch({ type: 'focus', id: caseId })
  return raw
}
