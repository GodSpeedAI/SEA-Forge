import { nowRevision, type Store } from '../model/store'
import type { Actor, Id, IntentRecord, ObjectAction } from '../model/types'
import type { CaseworkPort, InteractionIntent } from '../ports/contract'
import { compareCursor, projectSnapshot } from '../ports/project'

// The one path consequential work takes out of the UI, for humans and agents alike (VAR-007).
// It builds a contract InteractionIntent from what the backend offered (the object's
// ActionDescriptor), stamps the actor and the cursor the actor was looking at, sends it through
// the port, and records the backend's answer. The UI never decides authority: a refusal is
// shown as the backend phrased it (the typed refusal envelope when present), and world changes
// arrive only as new snapshots.
//
// Typed payloads (T01): the payload-carrying intents build their wire payloads here so every
// call site stays honest — EXECUTE_ITEM, COMPLETE_HUMAN_TASK, REOPEN_CASE and TERMINATE_CASE
// from the envelope, ADD_DISCRETIONARY_WORK from the drawer's typed payload.

export interface IntentPath {
  submit(
    actor: Actor,
    object: Id,
    action: ObjectAction,
    opts?: {
      justification?: string
      judgment?: boolean
      /** Typed payload (T01) riding in InteractionIntent.parameters; built here when omitted. */
      parameters?: Record<string, unknown>
    },
  ): Promise<IntentRecord>
}

export const uuid = () =>
  typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : 'xxxxxxxx-xxxx-4xxx-8xxx-xxxxxxxxxxxx'.replace(/x/g, () => ((Math.random() * 16) | 0).toString(16))

export function createIntentPath(store: Store, port: Pick<CaseworkPort, 'dispatchIntent' | 'getSnapshot'>): IntentPath {
  return {
    async submit(actor, object, action, opts = {}) {
      const s = store.getState()
      const caseId = s.history.caseId
      const justification = opts.justification
      // Typed payload per action (types.ts IntentRequestPayloads). PROPOSE_CASE is dispatched by
      // the dedicated design flow; the drawer supplies the full ADD_DISCRETIONARY_WORK payload.
      const parameters: Record<string, unknown> | undefined =
        opts.parameters ??
        (action.intent === 'EXECUTE_ITEM'
          ? { item_id: object }
          : action.intent === 'COMPLETE_HUMAN_TASK'
            ? { item_id: object, result: { note: justification }, justification }
            : action.intent === 'REOPEN_CASE' || action.intent === 'TERMINATE_CASE'
              ? { case_id: caseId, reason: justification }
              : undefined)
      const intent: InteractionIntent = {
        intent_id: uuid(),
        kind: action.consequential ? 'CONSEQUENTIAL_CASE' : 'BACKEND_INFORMATION',
        action_name: action.intent,
        target_object_id: object,
        case_id: caseId,
        // The newest state this client has seen; the backend refuses stale projections.
        client_cursor: nowRevision(s.history),
        actor: { actor_id: actor.id, role: actor.role as InteractionIntent['actor']['role'] },
        ...(justification ? { justification } : {}),
        ...(parameters ? { parameters } : {}),
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
        const refusal = r.success ? undefined : r.refusal
        let note = r.success ? 'Accepted by authority' : refusal?.message ?? r.error_message
        const code = refusal?.refusal_kind ?? r.error_code
        if (!r.success && code === 'STALE_PROJECTION') {
          try {
            // Refresh standing once after the refusal. The intent itself is never replayed.
            const fresh = await port.getSnapshot(caseId, intent.actor.actor_id, intent.actor.role)
            if (fresh.case_id !== caseId) {
              throw new Error(`snapshot refresh returned case ${String(fresh.case_id)} for requested case ${caseId}`)
            }
            const latest = store.getState().history
            // The user may have switched cases while the one refresh was in flight. Keep the
            // refusal, but never merge the old case's snapshot into the newly selected history.
            if (latest.caseId === caseId) {
              const projected = projectSnapshot(fresh, { withCore: true })
              const revisions = latest.revisions.filter((revision) => revision.id !== fresh.cursor).map((revision) => ({ ...revision }))
              revisions.push({ id: fresh.cursor, at: fresh.timestamp, label: '', summary: fresh.summary.phase })
              revisions.sort((a, b) => compareCursor(a.id, b.id))
              revisions.forEach((revision, index) => {
                revisions[index] = {
                  ...revision,
                  label: index === revisions.length - 1 ? 'Now' : revision.id === fresh.cursor ? new Date(fresh.timestamp).toLocaleDateString() : revision.label === 'Now' ? new Date(revision.at).toLocaleDateString() : revision.label,
                }
              })
              store.dispatch({
                type: 'loadHistory',
                history: {
                  ...latest,
                  revisions,
                  snapshots: { ...latest.snapshots, [fresh.cursor]: projected },
                },
              })
            }
          } catch (error) {
            const detail = error instanceof Error ? error.message : String(error)
            note = `${note ?? 'The projection was stale.'} Current projection refresh failed: ${detail}`
          }
        }
        store.dispatch({
          type: 'intentSettled',
          id: record.id,
          state: r.success ? 'accepted' : 'refused',
          note,
          code,
        })
        return {
          ...record,
          state: r.success ? 'accepted' : 'refused',
          note,
          code,
        }
      } catch (e) {
        const note = e instanceof Error ? e.message : String(e)
        store.dispatch({ type: 'intentSettled', id: record.id, state: 'refused', note: `Not delivered: ${note}`, code: 'UNAVAILABLE' })
        return { ...record, state: 'refused', note, code: 'UNAVAILABLE' }
      }
    },
  }
}
