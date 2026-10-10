import type { Store } from '../model/store'
import type { ExecutionState } from '../model/types'
import type { CaseworkPort, ExecutionProgressPayload, OperationalSettlement, StreamEvent, XSnapshot } from '../ports/contract'
import { compareCursor, projectHistory } from '../ports/project'

export interface LiveConnectionOptions {
  /** Bounded grace period before an outage is shown as interrupted. */
  interruptedAfterMs?: number
}

// Keeps the UI's world in step with the port's event stream. Snapshots append history (never
// rewrite it). Execution progress is projected for the execution pill and panel. Whether work
// is executed or settled is read from the snapshots only: a `settlement_recorded` event is
// shown only once the snapshot that carries the settlement arrives.

/**
 * Where an execution stands. An authoritative settlement object (the local fixture carries one)
 * decides. A live snapshot carries none, so its typed run status decides: the gateway reserves
 * COMPLETED for an accepted settlement and reports an executed-but-unsettled run as
 * WAITING_ON_OTHERS (ACTION_REQUIRED when escalated), so executed never reads as settled.
 */
export function executionStateOf(
  runStatus: string | undefined,
  settlement: { decision: string } | undefined,
  provenance: string,
): ExecutionState['state'] {
  if (settlement) return settlement.decision === 'ACCEPTED' ? 'settled' : 'rejected'
  if (provenance === 'local-contract') return runStatus === 'COMPLETED' ? 'executed' : 'running'
  switch (runStatus) {
    case 'COMPLETED':
      return 'settled'
    case 'REJECTED':
      return 'rejected'
    case 'FAILED':
      // The run failed or was terminated (an escalated run is terminated by the kernel): it is not
      // running, and it must never read as a run still in progress.
      return 'failed'
    case 'WAITING_ON_OTHERS':
    case 'ACTION_REQUIRED':
      return 'executed'
    default:
      return 'running'
  }
}

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
      const state = executionStateOf(run.contractStatus, target?.settlement, store.getState().history.provenance)
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
  const onFrame = (e: StreamEvent) => {
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
  }
  /**
   * The stream (re)opened. When nothing was missed the gateway sends no state frame, and control
   * frames (hello, heartbeat) cannot prove the source is reachable, so an idle recovery would stay
   * "Reconnecting" for ever. An opened stream plus a successful authoritative read of the world
   * proves recovery; that read is delivered as the same source-backed snapshot frame a patch is
   * (history dedupes by cursor, so it never duplicates a revision).
   */
  let probeTimer: ReturnType<typeof setTimeout> | undefined
  const probeSource = () => {
    probeTimer = undefined
    if (disposed || store.getState().connection === 'live') return
    const perspective = snaps.at(-1)?.perspective
    if (!perspective) return
    port.getSnapshot(caseId, perspective.actor_id, perspective.role as never).then(
      (snap) => {
        if (!disposed) onFrame({ event_type: 'snapshot', cursor: snap.cursor, timestamp: snap.timestamp, payload: snap })
      },
      () => {
        // Still unreachable: stay in the degraded state and look again shortly.
        if (!disposed && store.getState().connection !== 'live' && probeTimer === undefined) probeTimer = setTimeout(probeSource, 2000)
      },
    )
  }
  const unsubscribe = port.subscribeEvents(
    caseId,
    raw.at(-1)?.cursor,
    onFrame,
    (err) => {
      // Keep the last snapshot standing and report a bounded reconnect grace period.
      markRecovering()
      console.warn('[event stream]', err)
    },
    () => {
      if (store.getState().connection !== 'live') probeSource()
    },
  )
  // A live world opened after the work happened (a fresh login, a reload) carries the runs'
  // standing in its snapshots already; show it now instead of waiting for the next stream event.
  // The local fixture's history is narrative, not standing: its execution UI stays event-driven.
  if (store.getState().history.provenance !== 'local-contract') refreshExecutions()
  return () => {
    disposed = true
    if (probeTimer !== undefined) clearTimeout(probeTimer)
    if (outageTimer !== undefined) clearTimeout(outageTimer)
    outageTimer = undefined
    unsubscribe()
  }
}
