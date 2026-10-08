import type { Store } from '../model/store'
import type { ExecutionState } from '../model/types'
import type { CaseworkPort, ExecutionProgressPayload, OperationalSettlement, XSnapshot } from '../ports/contract'
import { compareCursor, projectHistory } from '../ports/project'

export interface LiveConnectionOptions {
  /** Bounded grace period before an outage is shown as interrupted. */
  interruptedAfterMs?: number
}

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
  options: LiveConnectionOptions = {},
): () => void {
  const snaps = [...raw]
  const logs = new Map<string, { phase: string; progress: number; log: string[] }>()
  const project = () => {
    const current = store.getState().history
    const incoming = projectHistory(snaps, summaries, current.provenance, { withCore: true })
    const revisions = new Map(current.revisions.map((revision) => [revision.id, revision]))
    for (const revision of incoming.revisions) revisions.set(revision.id, revision)
    const ordered = [...revisions.values()].sort((a, b) => compareCursor(a.id, b.id))
    ordered.forEach((revision, index) => {
      ordered[index] = {
        ...revision,
        label: index === ordered.length - 1 ? 'Now' : revision.label === 'Now' ? new Date(revision.at).toLocaleDateString() : revision.label,
      }
    })
    store.dispatch({
      type: 'loadHistory',
      history: { ...incoming, revisions: ordered, snapshots: { ...current.snapshots, ...incoming.snapshots } },
    })
  }

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

  const interruptedAfterMs = options.interruptedAfterMs ?? 15_000
  let outageTimer: ReturnType<typeof setTimeout> | undefined
  let disposed = false
  const markRecovering = () => {
    if (disposed) return
    if (store.getState().connection === 'live') store.dispatch({ type: 'connectionState', connection: 'reconnecting' })
    if (outageTimer === undefined) {
      outageTimer = setTimeout(() => {
        outageTimer = undefined
        if (!disposed && store.getState().connection === 'reconnecting') {
          store.dispatch({ type: 'connectionState', connection: 'interrupted' })
        }
      }, interruptedAfterMs)
    }
  }
  const markRecovered = () => {
    if (outageTimer !== undefined) clearTimeout(outageTimer)
    outageTimer = undefined
    if (!disposed && store.getState().connection !== 'live') store.dispatch({ type: 'connectionState', connection: 'live' })
  }
  const unsubscribe = port.subscribeEvents(
    caseId,
    raw.at(-1)?.cursor,
    (e) => {
      // Only source-backed state frames prove recovery. Heartbeats and diagnostic/control
      // frames cannot make an interrupted connection look live again.
      if (e.event_type === 'snapshot' || e.event_type === 'patch' || e.event_type === 'execution_progress' || e.event_type === 'settlement_recorded' || e.event_type === 'lease_expired') markRecovered()
      else if (e.event_type === 'interrupted') {
        if (outageTimer !== undefined) clearTimeout(outageTimer)
        outageTimer = undefined
        if (!disposed && store.getState().connection !== 'interrupted') store.dispatch({ type: 'connectionState', connection: 'interrupted' })
      } else if (e.event_type === 'error' || e.event_type === 'resync_required') markRecovering()
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
      // Keep the last snapshot standing and report a bounded reconnect grace period.
      markRecovering()
      console.warn('[event stream]', err)
    },
  )
  return () => {
    disposed = true
    if (outageTimer !== undefined) clearTimeout(outageTimer)
    outageTimer = undefined
    unsubscribe()
  }
}
