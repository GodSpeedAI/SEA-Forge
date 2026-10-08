import type {
  ActorRole,
  ArtifactPayload,
  CaseworkPort,
  ExecutionProgressPayload,
  IntentResponse,
  InteractionIntent,
  OperationalSettlement,
  SessionIdentity,
  SessionPort,
  StreamEvent,
  TemplateEntryOption,
  TemplateSourcePort,
  TemporalCheckpoint,
  TemporalTrajectoryResponse,
  XObject,
  XSnapshot,
} from '../../ports/contract'
import { buildNorthstarHistory, NORTHSTAR_CASE_ID, NORTHSTAR_TRAJECTORY } from './northstarData'

// Re-exported so the boot path can learn the fixture case id through this module alone —
// importing northstarData directly from main would ship the fixture as its own prod chunk.
export { NORTHSTAR_CASE_ID }
import { LOCAL_TEMPLATES, localPreflight, templateByRef, type LocalTemplate } from './localTemplates'
import { PAYLOADS } from './payloads'
import { buildTemplateHistory, TEMPLATE_CASE_ID } from './templateData'

// A local, contract-conformant stand-in for the Go system front end. It implements the contract
// CaseworkAdapter surface (CaseworkPort) plus the additive session (SessionPort) and template
// (TemplateSourcePort) surfaces over authored Northstar data. It decides authority the
// way the contract describes: role-aware, stale-projection and duplicate checks. It simulates
// governed execution as contract events: execution_progress, then snapshot, then
// settlement_recorded. This is NOT backend integration. Every screen that uses it is labelled
// "Local contract adapter".

export interface LocalAdapterOptions {
  /** Execution speed multiplier (1 = ~1.2 s per phase). */
  speed?: number
  /** Artifact refs whose resolution fails (recovery tests). */
  failArtifacts?: string[]
  /** Artifact refs that resolve to a malformed payload (recovery tests). */
  corruptArtifacts?: string[]
  /** Network latency in ms for reads. */
  latency?: number
  /**
   * Session identity the adapter self-reports (T09). Default: the fixture operator.
   * Pass null to start logged out (login tests): `login()` then establishes a session.
   */
  identity?: SessionIdentity | null
}

interface CaseRecord {
  snaps: XSnapshot[]
  points: TemporalCheckpoint[]
}

const APPROVER_ROLES: ActorRole[] = ['case_architect', 'security_officer']
const PLACEHOLDER = { actor_id: 'projection', role: 'developer' as ActorRole }

/** The fixture operator the local adapter self-reports as the session identity. */
export const LOCAL_IDENTITY: SessionIdentity = {
  actor_id: 'usr-sam',
  role: 'case_architect',
  display_name: 'Sam Prime',
  kind: 'human',
}

const clone = <T,>(v: T): T => structuredClone(v)
const pad = (n: number) => String(n).padStart(10, '0')

export class LocalContractAdapter implements CaseworkPort, SessionPort, TemplateSourcePort {
  private cases = new Map<string, CaseRecord>()
  private listeners = new Map<string, Set<(e: StreamEvent) => void>>()
  private seen = new Set<string>()
  private payloads: Record<string, ArtifactPayload> = { ...PAYLOADS }
  private identity: SessionIdentity | null
  private opts: Required<Omit<LocalAdapterOptions, 'identity'>>

  constructor(opts: LocalAdapterOptions = {}) {
    this.opts = { speed: 1, failArtifacts: [], corruptArtifacts: [], latency: 40, ...opts }
    this.identity = opts.identity === undefined ? { ...LOCAL_IDENTITY } : opts.identity
    const ns = buildNorthstarHistory(PLACEHOLDER).map((s) => withTemplateLink(s))
    this.cases.set(NORTHSTAR_CASE_ID, { snaps: ns, points: [...NORTHSTAR_TRAJECTORY] })
    const tpl = buildTemplateHistory(PLACEHOLDER)
    this.cases.set(TEMPLATE_CASE_ID, {
      snaps: tpl,
      points: tpl.map((s) => ({
        cursor: s.cursor,
        timestamp: s.timestamp,
        event_type: 'template_published',
        summary: s.x?.label ?? s.summary.phase,
        actor_id: 'usr-sam',
        actor_role: 'case_architect',
        consequential: true,
        completed_plan_items_count: 0,
        total_plan_items_count: 0,
      })),
    })
  }

  // --- session (T09, trivial fixture identity) ---------------------------------

  /** The self-reported identity, or null while logged out. */
  async session(): Promise<SessionIdentity | null> {
    await this.wait(10)
    return this.identity ? { ...this.identity } : null
  }

  /** Fixture login: any non-empty username is accepted as the case-architect operator. */
  async login(username: string, _password: string): Promise<SessionIdentity> {
    await this.wait(60)
    if (!username.trim()) {
      const err = new Error('A username is required.') as Error & { code?: string }
      err.code = 'INVALID'
      throw err
    }
    this.identity = { actor_id: username.trim(), role: 'case_architect', display_name: username.trim(), kind: 'human' }
    return { ...this.identity }
  }

  async logout(): Promise<void> {
    await this.wait(20)
    this.identity = null
  }

  // --- templates (T09 fixture surface; the live gateway serves the kernel's) ----------------

  async getTemplates(): Promise<readonly TemplateEntryOption[]> {
    await this.wait()
    return clone(LOCAL_TEMPLATES)
  }

  async preflightTemplate(templateRef: string, params: Record<string, unknown>) {
    await this.wait(80)
    return localPreflight(templateRef, params)
  }

  // --- reads -----------------------------------------------------------------

  async getSnapshot(caseId: string, actorId: string, role: ActorRole): Promise<XSnapshot> {
    await this.wait()
    const c = this.case(caseId)
    return this.stamp(c.snaps[c.snaps.length - 1]!, actorId, role)
  }

  async getSnapshotAt(caseId: string, cursor: string, actorId: string, role: ActorRole): Promise<XSnapshot> {
    await this.wait()
    const s = this.case(caseId).snaps.find((x) => x.cursor === cursor)
    if (!s) throw new Error(`No snapshot at ${cursor} (resync required)`)
    return this.stamp(s, actorId, role)
  }

  async resolveArtifact(evidenceId: string): Promise<ArtifactPayload> {
    await this.wait()
    if (this.opts.failArtifacts.includes(evidenceId)) throw new Error(`Artifact ${evidenceId} is unavailable`)
    const p = this.payloads[evidenceId]
    if (!p) throw new Error(`Unknown artifact ${evidenceId}`)
    if (this.opts.corruptArtifacts.includes(evidenceId)) return { ...clone(p), content: p.content.slice(0, Math.floor(p.content.length / 3)) + '\u0000{' }
    return clone(p)
  }

  async queryTemporalTrajectory(caseId: string): Promise<TemporalTrajectoryResponse> {
    await this.wait()
    const c = this.case(caseId)
    return clone({ case_id: caseId, base_cursor: c.points[0]!.cursor, head_cursor: c.points[c.points.length - 1]!.cursor, points: c.points })
  }

  subscribeEvents(caseId: string, _since: string | undefined, onEvent: (e: StreamEvent) => void): () => void {
    let set = this.listeners.get(caseId)
    if (!set) this.listeners.set(caseId, (set = new Set()))
    set.add(onEvent)
    return () => set!.delete(onEvent)
  }

  // --- authority ---------------------------------------------------------------

  async dispatchIntent(intent: InteractionIntent): Promise<IntentResponse> {
    await this.wait(180)
    const params = (intent.parameters ?? {}) as Record<string, unknown>
    // Justification may ride the envelope or the typed payload (golden intent fixtures do both).
    const justification = String(intent.justification ?? params.justification ?? '').trim()
    const REFUSAL_KINDS = new Set(['AUTHORITY_DENIED', 'UNAUTHORIZED_ROLE', 'SOD_VIOLATION', 'STALE_PROJECTION', 'JUSTIFICATION_REQUIRED', 'UNAVAILABLE', 'INVALID'])
    const refuse = (code: string, message: string): IntentResponse => ({
      intent_id: intent.intent_id,
      success: false,
      error_code: code,
      error_message: message,
      refusal: { refusal_kind: (REFUSAL_KINDS.has(code) ? code : 'INVALID') as never, message },
    })
    if (this.seen.has(intent.intent_id)) return refuse('DUPLICATE_IN_FLIGHT', 'This request was already received.')
    this.seen.add(intent.intent_id)

    // PROPOSE_CASE commits a new case from a template; it targets no existing case, so the
    // stale-projection and target checks do not apply (client_cursor is not required). The
    // commit must echo a passing preflight's digest.
    if (intent.action_name === 'PROPOSE_CASE' && intent.kind === 'CONSEQUENTIAL_CASE') {
      if (intent.actor.role === 'agent_operator') return refuse('AUTHORITY_DENIED', 'Agents may propose but not decide. A human approver must make this decision.')
      const payload = params as { template_ref?: string; params?: Record<string, unknown>; preflight_digest?: string }
      if (!payload.template_ref) return refuse('INVALID', 'A template must be chosen before a case can be committed.')
      const check = await localPreflight(payload.template_ref, payload.params ?? {})
      if (!check.passed) return refuse('INVALID', `Preflight no longer passes: ${check.reasons.join('; ')}`)
      if (!payload.preflight_digest) return refuse('INVALID', 'Run preflight and commit the digest it produced.')
      if (payload.preflight_digest !== check.digest) {
        return refuse('STALE_PROJECTION', 'The world has moved since the preflight: run preflight again and commit the fresh digest.')
      }
      return this.commitCase(payload.template_ref, payload.params ?? {}, payload.preflight_digest, intent)
    }

    const c = this.cases.get(intent.case_id)
    if (!c) return refuse('INVALID', `Unknown case ${intent.case_id}`)
    const head = c.snaps[c.snaps.length - 1]!
    const target = head.visible_objects.find((o) => o.id === intent.target_object_id)
    if (!target) return refuse('INVALID', 'The target no longer exists.')
    if (intent.kind !== 'CONSEQUENTIAL_CASE') return { intent_id: intent.intent_id, success: true }

    if (intent.client_cursor !== head.cursor) {
      return refuse('STALE_PROJECTION', 'The case changed since this view was loaded. Review the current state and decide again.')
    }
    const action = target.actions.find((a) => a.intent === intent.action_name)
    if (!action) return refuse('UNAVAILABLE', `“${target.name}” does not offer this action now.`)
    if (intent.actor.role === 'agent_operator') {
      return refuse('AUTHORITY_DENIED', 'Agents may propose but not decide. A human approver must make this decision.')
    }
    if ((action.intent === 'APPROVE_HUMAN_TASK' || action.intent === 'REJECT_HUMAN_TASK') && !APPROVER_ROLES.includes(intent.actor.role)) {
      return refuse('UNAUTHORIZED_ROLE', `Your role (${intent.actor.role}) cannot decide this. It needs a case architect or security officer.`)
    }
    if (action.intent === 'ADD_DISCRETIONARY_WORK' && !String(params.title ?? '').trim()) {
      return refuse('INVALID', 'Discretionary work needs a title.')
    }
    if (action.requires_justification && !justification) {
      return refuse('JUSTIFICATION_REQUIRED', 'This action needs a written justification.')
    }

    let created: XObject | undefined
    const next = this.append(intent.case_id, `${action.intent.toLowerCase()}`, `${action.label} by ${intent.actor.actor_id}`, intent, (objs) => {
      const t = objs.find((o) => o.id === target.id)!
      if (action.intent === 'APPROVE_HUMAN_TASK') {
        set(objs, t.id, { status: 'IN_PROGRESS', badge: 'Approved · rolling out', actions: [], x: { ...t.x, tone: 'progress' } })
        objs.push(runObject(t.id, 'IN_PROGRESS', 'Running'))
      } else if (action.intent === 'REJECT_HUMAN_TASK') {
        set(objs, t.id, { status: 'WAITING_ON_OTHERS', badge: 'Changes requested', actions: [], x: { ...t.x, tone: 'attention' } })
      } else if (action.intent === 'ESCALATE_OR_OVERRIDE') {
        set(objs, t.id, { status: 'WAITING_ON_OTHERS', badge: 'Escalated to owner', actions: [], x: { ...t.x, tone: 'muted' } })
      } else if (action.intent === 'EXECUTE_ITEM') {
        set(objs, t.id, { status: 'IN_PROGRESS', badge: 'Executing', x: { ...t.x, tone: 'progress' } })
        objs.push(runObject(t.id, 'IN_PROGRESS', 'Running'))
      } else if (action.intent === 'COMPLETE_HUMAN_TASK') {
        set(objs, t.id, { status: 'COMPLETED', badge: 'Completed', actions: [], x: { ...t.x, tone: 'ok' } })
      } else if (action.intent === 'REOPEN_CASE') {
        set(objs, t.id, { status: 'IN_PROGRESS', badge: 'Reopened', explanation: `Reopened by ${intent.actor.actor_id}`, actions: [], x: { ...t.x, tone: 'progress' } })
      } else if (action.intent === 'TERMINATE_CASE') {
        set(objs, t.id, { status: 'WAITING_ON_OTHERS', badge: 'Terminated', explanation: `Terminated by ${intent.actor.actor_id}: ${justification}`, actions: [], x: { ...t.x, tone: 'muted' } })
      } else if (action.intent === 'ADD_DISCRETIONARY_WORK') {
        const title = String(params.title)
        created = {
          id: `disc-${Math.random().toString(36).slice(2, 8)}`,
          kind: 'work_item',
          name: title,
          status: 'AVAILABLE_TO_ADD',
          badge: 'Proposed',
          explanation: `Proposed by ${intent.actor.actor_id}: ${justification}`,
          salience: 0.4,
          parent_id: t.id,
          actions: [],
        }
        objs.push(created)
      }
    })
    if (action.intent === 'APPROVE_HUMAN_TASK') this.execute(intent.case_id, target.id)
    if (action.intent === 'EXECUTE_ITEM') this.execute(intent.case_id, target.id)
    const response: IntentResponse = { intent_id: intent.intent_id, success: true, new_cursor: next.cursor }
    if (created) response.resulting_object = clone(created)
    return response
  }

  /** PROPOSE_CASE commit: builds the new case's initial world from the local template plan. */
  private commitCase(templateRef: string, params: Record<string, unknown>, _digest: string, intent: InteractionIntent): IntentResponse {
    const template = templateByRef(templateRef) as LocalTemplate
    const caseId = `case-${templateRef.replace(/^tpl-/, '')}-${Math.random().toString(36).slice(2, 6)}`
    const paramNote = Object.entries(params)
      .filter(([, v]) => v !== undefined && String(v).trim() !== '')
      .map(([k, v]) => `${k}=${v}`)
      .join(', ')
    const caseObj: XObject = {
      id: caseId,
      kind: 'work_item',
      name: template.title,
      status: 'READY_TO_BEGIN',
      badge: 'Committed',
      explanation: paramNote ? `From ${templateRef} · ${paramNote}` : `From ${templateRef}`,
      salience: 0.95,
      parent_id: undefined,
      depends_on: [],
      actions: [
        { id: 'add-discretionary', label: 'Add discretionary work', intent: 'ADD_DISCRETIONARY_WORK', consequential: true, variant: 'SECONDARY', requires_justification: true },
      ],
      x: { presentation: 'case', tone: 'progress' },
    }
    const stageObjs: XObject[] = template.plan.stages.map((st, i) => ({
      id: `${caseId}-${st.id}`,
      kind: 'stage',
      name: st.name,
      status: i === 0 ? 'READY_TO_BEGIN' : 'WAITING',
      badge: i === 0 ? 'Ready' : 'Waiting',
      explanation: st.explanation,
      salience: 0.55 - i * 0.05,
      parent_id: caseId,
      depends_on: st.depends_on?.map((d) => `${caseId}-${d}`),
      actions: st.action ? [{ id: `${caseId}-${st.action.id}`, label: st.action.label, intent: st.action.intent, consequential: true, variant: 'PRIMARY', ...(st.action.requires_justification ? { requires_justification: true } : {}) }] : [],
      x: { presentation: 'facet', tone: i === 0 ? 'progress' : 'muted' },
    }))
    const now = new Date().toISOString()
    const snap: XSnapshot = {
      world_id: `world-${caseId}`,
      case_id: caseId,
      cursor: '1.0000000001',
      timestamp: now,
      perspective: { actor_id: intent.actor.actor_id, role: intent.actor.role, display_name: this.identity?.display_name },
      summary: { headline: template.title, phase: 'Case committed', status_phrase: 'Ready to begin', progress_percent: 0 },
      available_actions: [],
      attention_focus: { primary_object_id: caseId },
      visible_objects: [caseObj, ...stageObjs],
      x: { relationships: stageObjs.flatMap((s) => (s.depends_on ?? []).map((d) => ({ id: `dep:${s.id}>${d}`, from: s.id, to: d, kind: 'depends-on' as const }))) },
    }
    this.cases.set(caseId, {
      snaps: [Object.freeze(snap) as XSnapshot],
      points: [
        {
          cursor: snap.cursor,
          timestamp: now,
          event_type: 'case_committed',
          summary: 'Case committed from template',
          actor_id: intent.actor.actor_id,
          actor_role: intent.actor.role,
          consequential: true,
          completed_plan_items_count: 0,
          total_plan_items_count: stageObjs.length,
        },
      ],
    })
    this.emit(caseId, { event_type: 'snapshot', cursor: snap.cursor, timestamp: now, payload: clone(snap) })
    return { intent_id: intent.intent_id, success: true, new_cursor: snap.cursor, resulting_object: clone(caseObj) }
  }

  // --- governed execution (simulated as contract events) -----------------------

  private execute(caseId: string, targetId: string) {
    const runId = `run-${targetId}-1`
    const step = 1200 / this.opts.speed
    const phases: [ExecutionProgressPayload['phase'], number, string][] = [
      ['orchestrator', 0.1, 'Lease granted; sandbox profile gauntlet-default'],
      ['builder', 0.35, 'Applying release v0.3.0 to 10% cohort'],
      ['critic', 0.6, 'Coverage guard: 1,204 claims checked, 0 rejected'],
      ['verifier', 0.85, 'Evidence bundle sealed'],
    ]
    phases.forEach(([phase, p, log], i) => setTimeout(() => this.progress(caseId, runId, phase, p, log), step * (i + 1)))
    setTimeout(() => {
      this.payloads['evi-rollout-checks'] = rolloutChecks(caseId, targetId, runId)
      this.append(caseId, 'execution_completed', 'Rollout executed; awaiting settlement', null, (objs) => {
        const t = objs.find((o) => o.id === targetId)!
        set(objs, runId, { status: 'COMPLETED', badge: 'Executed' })
        set(objs, targetId, { status: 'COMPLETED', badge: 'Executed · awaiting settlement', x: { ...t.x, tone: 'hypothesis' } })
        objs.push(evidenceObject(targetId, runId))
      })
    }, step * 5)
    setTimeout(() => this.progress(caseId, runId, 'settling', 0.95, 'Settlement evaluating evidence'), step * 6)
    setTimeout(() => {
      const settlement: OperationalSettlement = {
        settlement_id: `stl-${targetId}-1`,
        case_id: caseId,
        plan_item_id: targetId,
        invocation_id: `inv-${targetId}-1`,
        decision: 'ACCEPTED',
        evidence_id: 'evi-rollout-checks',
        settled_at: new Date().toISOString(),
        consequence_summary: 'Release is live; the coverage guard held in production.',
      }
      const snap = this.append(caseId, 'settlement_recorded', 'Release settled & verified', null, (objs) => {
        const t = objs.find((o) => o.id === targetId)!
        set(objs, targetId, { status: 'COMPLETED', badge: 'Settled & verified', salience: Math.min(t.salience, 0.35), x: { ...t.x, tone: 'ok', settlement } })
        // The settled release resolves what it was blocking: settled work goes quiet.
        for (const o of objs) {
          if (o.depends_on?.includes(targetId) || (t.depends_on ?? []).includes(o.id)) continue
          if (o.x?.residue) set(objs, o.id, { x: { ...o.x, residue: undefined } })
        }
      })
      this.emit(caseId, { event_type: 'settlement_recorded', cursor: snap.cursor, timestamp: settlement.settled_at, payload: settlement })
    }, step * 7.5)
  }

  private progress(caseId: string, run_id: string, phase: ExecutionProgressPayload['phase'], progress_percent: number, log_line: string) {
    const c = this.case(caseId)
    this.emit(caseId, {
      event_type: 'execution_progress',
      cursor: c.snaps[c.snaps.length - 1]!.cursor,
      timestamp: new Date().toISOString(),
      payload: { run_id, phase, progress_percent, log_line } satisfies ExecutionProgressPayload,
    })
  }

  /** Appends an immutable new snapshot (history is never rewritten) and emits it. */
  private append(caseId: string, eventType: string, summary: string, intent: InteractionIntent | null, edit: (objs: XObject[]) => void): XSnapshot {
    const c = this.case(caseId)
    const head = c.snaps[c.snaps.length - 1]!
    const [epoch, seq] = head.cursor.split('.').map(Number) as [number, number]
    const objs = clone(head.visible_objects) as XObject[]
    edit(objs)
    const now = new Date().toISOString()
    const snap: XSnapshot = {
      ...clone(head),
      cursor: `${epoch}.${pad(seq + 1)}`,
      timestamp: now,
      visible_objects: objs,
      summary: { ...head.summary, phase: summary },
      x: { ...head.x, label: undefined },
    }
    c.snaps.push(Object.freeze(snap) as XSnapshot)
    c.points.push({
      cursor: snap.cursor,
      timestamp: now,
      event_type: eventType,
      summary,
      actor_id: intent?.actor.actor_id ?? 'gauntlet',
      actor_role: intent?.actor.role ?? 'agent_operator',
      consequential: !!intent,
      completed_plan_items_count: objs.filter((o) => o.status === 'COMPLETED').length,
      total_plan_items_count: objs.length,
    })
    this.emit(caseId, { event_type: 'snapshot', cursor: snap.cursor, timestamp: now, payload: clone(snap) })
    return snap
  }

  private emit(caseId: string, e: StreamEvent) {
    for (const fn of this.listeners.get(caseId) ?? []) setTimeout(() => fn(e), 0)
  }

  private case(caseId: string): CaseRecord {
    const c = this.cases.get(caseId)
    if (!c) throw new Error(`Unknown case ${caseId}`)
    return c
  }

  /** Role-aware projection: agents never receive consequential affordances. */
  private stamp(s: XSnapshot, actor_id: string, role: ActorRole): XSnapshot {
    const out = clone(s) as XSnapshot & { perspective: XSnapshot['perspective'] }
    ;(out as { perspective: unknown }).perspective = { actor_id, role }
    if (role === 'agent_operator') {
      ;(out as unknown as { visible_objects: XObject[] }).visible_objects = out.visible_objects.map((o) => ({ ...o, actions: o.actions.filter((a) => !a.consequential) }))
    }
    return out
  }

  private wait(ms = this.opts.latency) {
    return new Promise((r) => setTimeout(r, ms))
  }
}

function set(objs: XObject[], id: string, patch: Partial<XObject>) {
  const i = objs.findIndex((o) => o.id === id)
  if (i >= 0) objs[i] = { ...objs[i]!, ...patch }
}

function withTemplateLink(s: XSnapshot): XSnapshot {
  return {
    ...s,
    visible_objects: s.visible_objects.map((o) =>
      o.x?.presentation === 'case' && o.id === 'northstar' ? { ...o, x: { ...o.x, template_case_id: TEMPLATE_CASE_ID } } : o,
    ),
  }
}

function runObject(parent: string, status: XObject['status'], badge: string): XObject {
  return {
    id: `run-${parent}-1`,
    kind: 'execution_trace',
    name: 'Rollout run',
    status,
    badge,
    explanation: 'Gauntlet · sandboxed',
    salience: 0.45,
    parent_id: parent,
    actions: [],
    x: { presentation: 'run', tone: 'progress' },
  }
}

function evidenceObject(parent: string, runId: string): XObject {
  return {
    id: `evo-${parent}-1`,
    kind: 'evidence_record',
    name: 'Rollout evidence',
    status: 'COMPLETED',
    badge: 'Recorded',
    explanation: `From ${runId}`,
    salience: 0.4,
    parent_id: parent,
    actions: [],
    x: {
      presentation: 'item',
      tone: 'ok',
      artifacts: [
        { ref: 'evi-rollout-checks', kind: 'test_report', title: 'Rollout checks', boundObject: `evo-${parent}-1`, currentLevel: 'minimal', mediaType: 'application/json', sourceProvenance: 'sxr:cas://sha256:rollout-checks' },
        { ref: 'evi-release-log', kind: 'document', title: 'Release log', boundObject: `evo-${parent}-1`, currentLevel: 'minimal', mediaType: 'text/plain', sourceProvenance: 'sxr:cas://sha256:release-log' },
      ],
    },
  }
}

function rolloutChecks(caseId: string, target: string, runId: string): ArtifactPayload {
  const content = JSON.stringify({
    schema: 'table',
    columns: ['Check', 'Cohort', 'Result', 'Claims'],
    rows: [
      ['Coverage guard', '10%', 'Pass', '1204'],
      ['Eligibility validation', '10%', 'Pass', '1204'],
      ['Claim acceptance rate', '10%', 'Pass', '99.2%'],
      ['Rollback drill', 'staging', 'Pass', '—'],
    ],
    caption: 'Evidence sealed by the verifier',
  })
  return {
    evidence_id: 'evi-rollout-checks',
    name: 'Rollout checks',
    digest: 'sha256:' + 'c0ffee'.repeat(10) + 'c0ff',
    content_type: 'application/json',
    content,
    provenance: { case_id: caseId, plan_item_id: target, invocation_id: `inv-${target}-1`, run_id: runId },
  }
}
