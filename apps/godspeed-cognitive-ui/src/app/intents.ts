import { nowRevision, type Store } from '../model/store'
import type { Actor, Id, IntentRecord, ObjectAction } from '../model/types'
import type { CaseworkPort, InteractionIntent } from '../ports/contract'

// The one path consequential work takes out of the UI, for humans and agents alike (VAR-007).
// It builds a contract InteractionIntent from what the backend offered (the object's
// ActionDescriptor), stamps the actor and the cursor the actor was looking at, sends it through
// the port, and records the backend's answer. The UI never decides authority: a refusal is
// shown as the backend phrased it, and world changes arrive only as new snapshots.

export interface IntentPath {
  submit(actor: Actor, object: Id, action: ObjectAction, opts?: { justification?: string; judgment?: boolean }): Promise<IntentRecord>
}

const uuid = () =>
  typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : 'xxxxxxxx-xxxx-4xxx-8xxx-xxxxxxxxxxxx'.replace(/x/g, () => ((Math.random() * 16) | 0).toString(16))

export function createIntentPath(store: Store, port: Pick<CaseworkPort, 'dispatchIntent'>): IntentPath {
  return {
    async submit(actor, object, action, opts = {}) {
      const s = store.getState()
      const intent: InteractionIntent = {
        intent_id: uuid(),
        kind: action.consequential ? 'CONSEQUENTIAL_CASE' : 'BACKEND_INFORMATION',
        action_name: action.intent,
        target_object_id: object,
        case_id: s.history.caseId,
        // The newest state this client has seen; the backend refuses stale projections.
        client_cursor: nowRevision(s.history),
        actor: { actor_id: actor.id, role: actor.role as InteractionIntent['actor']['role'] },
        ...(opts.justification ? { justification: opts.justification } : {}),
      }
      const record: IntentRecord = {
        id: intent.intent_id,
        actionName: action.intent,
        target: object,
        actor,
        state: 'sending',
        justification: opts.justification,
      }
      store.dispatch({ type: 'intentSent', record, judgment: opts.judgment })
      try {
        const r = await port.dispatchIntent(intent)
        store.dispatch({
          type: 'intentSettled',
          id: record.id,
          state: r.success ? 'accepted' : 'refused',
          note: r.success ? 'Accepted by authority' : r.error_message,
          code: r.error_code,
        })
        return { ...record, state: r.success ? 'accepted' : 'refused', note: r.error_message, code: r.error_code }
      } catch (e) {
        const note = e instanceof Error ? e.message : String(e)
        store.dispatch({ type: 'intentSettled', id: record.id, state: 'refused', note: `Not delivered: ${note}`, code: 'UNAVAILABLE' })
        return { ...record, state: 'refused', note, code: 'UNAVAILABLE' }
      }
    },
  }
}
