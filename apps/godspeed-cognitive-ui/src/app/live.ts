import type { Store } from '../model/store'
import type { ExecutionState } from '../model/types'
import type { CaseworkPort, ExecutionProgressPayload, OperationalSettlement, XSnapshot } from '../ports/contract'
import { projectHistory } from '../ports/project'

// Keeps the UI's world in step with the port's event stream. Snapshots append history (never
// rewrite it). Execution progress is projected for the execution pill and panel. Whether work
// is executed or settled is read from the snapshots only: a `settlement_recorded` event is
// shown only once the snapshot that carries the settlement arrives.

export function connectLive(
  port: CaseworkPort,
  store: Store,
  caseId: string,
  raw: XSnapshot[],
  summaries: Record<string, string>,
): () => void {
  const snaps = [...raw]
  const logs = new Map<string, { phase: string; progress: number; log: string[] }>()
  const project = () => store.dispatch({ type: 'loadHistory', history: projectHistory(snaps, summaries, store.getState().history.provenance, { withCore: true }) })

  const refreshExecutions = () => {
    const s = store.getState()
    const now = s.history.snapshots[s.history.revisions.at(-1)!.id]!
    for (const run of Object.values(now.objects)) {
      if (run.kind !== 'run' || !run.parent) continue
      const target = now.objects[run.parent]
      const ev = logs.get(run.id)
      const state: ExecutionState['state'] = target?.settlement
        ? target.settlement.decision === 'ACCEPTED' ? 'settled' : 'rejected'
        : run.contractStatus === 'COMPLETED' ? 'executed' : 'running'
      // Progress is shown only when the stream reported it. Snapshot standing alone never
      // produces a percentage: an honest pill shows the phase, not a guessed number (T09).
      const progress: number | null = state === 'settled' ? 1 : ev?.progress ?? null
      const exec: ExecutionState = {
        object: run.parent,
        runId: run.id,
        phase: state === 'settled' ? 'settled' : ev?.phase ?? 'running',
        progress,
        log: ev?.log ?? [],
        state,
      }
      const prev = s.executions[run.parent]
      if (!prev || prev.state !== exec.state || prev.progress !== exec.progress || prev.log.length !== exec.log.length || prev.phase !== exec.phase) {
        store.dispatch({ type: 'execution', exec })
      }
    }
  }

  return port.subscribeEvents(
    caseId,
    raw.at(-1)?.cursor,
    (e) => {
      // Any delivered event is proof the stream is alive again (resume from the cursor).
      if (store.getState().connection !== 'live') store.dispatch({ type: 'connectionState', connection: 'live' })
      if (e.event_type === 'snapshot' || e.event_type === 'patch') {
        const snap = e.payload as XSnapshot
        if (!snaps.some((x) => x.cursor === snap.cursor)) {
          snaps.push(snap)
          summaries[snap.cursor] = snap.summary.phase
          project()
        }
        refreshExecutions()
      } else if (e.event_type === 'execution_progress') {
        const p = e.payload as ExecutionProgressPayload
        const cur = logs.get(p.run_id) ?? { phase: p.phase, progress: 0, log: [] }
        logs.set(p.run_id, { phase: p.phase, progress: p.progress_percent, log: [...cur.log, p.log_line].slice(-40) })
        refreshExecutions()
      } else if (e.event_type === 'settlement_recorded') {
        const st = e.payload as OperationalSettlement
        const s = store.getState()
        const now = s.history.snapshots[s.history.revisions.at(-1)!.id]!
        for (const run of Object.values(now.objects)) {
          const cur = run.kind === 'run' && run.parent === st.plan_item_id ? logs.get(run.id) : undefined
          if (cur) cur.log = [...cur.log, `Settlement ${st.decision.toLowerCase()}: ${st.consequence_summary}`]
        }
        refreshExecutions()
      }
    },
    (err) => {
      // Connection interrupted: the UI shows Reconnecting and keeps the last snapshot standing.
      // No progress is fabricated while offline; the subscription resumes from its cursor.
      store.dispatch({ type: 'connectionState', connection: 'reconnecting' })
      console.warn('[event stream]', err)
    },
  )
}
