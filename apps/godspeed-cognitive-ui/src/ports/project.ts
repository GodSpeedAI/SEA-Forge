import type { Actor, CaseworkPort, CognitiveRelationship, XObject, XSnapshot } from './contract'
import type { ObjectKind, Relationship, RelationKind, Revision, Status, WorldHistory, WorldObject, WorldSnapshot } from '../model/types'
import { presentationOf, sentryExplanation, statusPhrase, toneOf } from '../model/derive'

// Contract snapshot → internal world. This is the only place contract shapes are read; the
// layouts, scene and panels only ever see the projection. It is generic: nothing here (or
// above it) recognises particular ids or names.
//
// Presentation (kind class, tone, phrase, sentry explanation) is DERIVED locally from kernel
// fields (src/model/derive.ts, T09): live snapshots carry no `x` extensions. Authored `x`
// extensions (the local fixture) win over derivation wherever present.

export const CORE_ID = 'core'

const RELATION_KIND: Record<CognitiveRelationship['kind'], RelationKind> = {
  'depends-on': 'depends',
  produces: 'leads-to',
  attests: 'evidences',
  'governed-by': 'relates',
  contradicts: 'relates',
}

/** Presentation class for the authored `x.presentation` names, else derived from the kernel kind. */
function presentationOfX(xPresentation: string | undefined, kind: string): ObjectKind {
  const byX: Record<string, ObjectKind> = {
    core: 'core',
    region: 'category',
    case: 'case',
    facet: 'facet',
    item: 'item',
    person: 'person',
    run: 'run',
  }
  return (xPresentation && byX[xPresentation]) || presentationOf(kind)
}

export function projectObject(o: XObject, parentFallback: string | null): WorldObject {
  const x = o.x ?? {}
  const status: Status = { label: o.badge || statusPhrase(o.status), tone: toneOf(o) }
  const kind = presentationOfX(x.presentation, o.kind)
  const obj: WorldObject = {
    id: o.id,
    kind,
    parent: o.parent_id ?? parentFallback,
    title: o.name,
    subtitle: o.explanation,
    metric: x.metric,
    status,
    salience: o.salience,
    ghost: x.ghost,
    dormant: x.dormant,
    actions: o.actions.map((a) => ({
      id: a.id,
      label: a.label,
      intent: a.intent,
      consequential: a.consequential,
      variant: a.variant,
      requiresJustification: a.requires_justification,
    })),
    artifacts: x.artifacts?.map((a) => a.ref),
    residue: x.residue ? { label: x.residue.label, tone: x.residue.tone } : undefined,
    icon: x.design?.icon,
    accent: x.design?.accent,
    designItems: x.design?.items,
    contractKind: o.kind,
    contractStatus: o.status,
    causal: x.representations?.causal,
    settlement: x.settlement,
    templateCaseId: x.template_case_id,
  }
  if (o.spatial_layout) {
    obj.slot = {
      x: o.spatial_layout.x,
      y: o.spatial_layout.y,
      size: o.spatial_layout.radius ? o.spatial_layout.radius * 2 : 0,
      label: x.label_side,
    }
  }
  return obj
}

export interface ProjectOptions {
  /** Add the persistent Core as the root of the world (the system surface). */
  withCore: boolean
}

export function projectSnapshot(snap: XSnapshot, opts: ProjectOptions): WorldSnapshot {
  const ids = new Set(snap.visible_objects.map((o) => o.id))
  const objects: Record<string, WorldObject> = {}
  if (opts.withCore) {
    objects[CORE_ID] = { id: CORE_ID, kind: 'core', parent: null, title: 'Core', salience: 1 }
  }
  const artifacts: WorldSnapshot['artifacts'] = {}
  for (const o of snap.visible_objects) {
    const orphan = !o.parent_id || !ids.has(o.parent_id)
    const obj = projectObject(orphan ? { ...o, parent_id: undefined } : o, opts.withCore ? CORE_ID : null)
    objects[o.id] = obj
    for (const a of o.x?.artifacts ?? []) artifacts[a.ref] = a
  }
  const relationships: Relationship[] = (snap.x?.relationships ?? [])
    .filter((r) => objects[r.from] && objects[r.to])
    .map((r) => ({ id: r.id, from: r.from, to: r.to, kind: RELATION_KIND[r.kind] ?? 'relates', label: r.label }))
  for (const o of snap.visible_objects) {
    for (const d of o.depends_on ?? []) {
      if (!objects[d] || relationships.some((r) => r.from === o.id && r.to === d)) continue
      relationships.push({ id: `dep:${o.id}>${d}`, from: o.id, to: d, kind: 'depends' })
    }
  }
  // Sentry transparency: when the kernel sent no explanation, derive one from depends_on plus
  // each dependency's standing, so blocked work says what it is waiting on (T09).
  const byId: Record<string, XObject> = Object.fromEntries(snap.visible_objects.map((o) => [o.id, o]))
  for (const o of snap.visible_objects) {
    if (o.explanation || !o.depends_on?.length) continue
    const sentry = sentryExplanation(o, byId)
    if (sentry && objects[o.id]) objects[o.id] = { ...objects[o.id]!, subtitle: sentry }
  }
  const attention = snap.attention_focus?.primary_object_id
  return { revision: snap.cursor, caseId: snap.case_id, objects, relationships, artifacts, attention: attention && objects[attention] ? attention : undefined }
}

const shortDate = (iso: string) => {
  const d = new Date(iso)
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', timeZone: 'UTC' })
}

/** Builds the internal history from contract snapshots (oldest first) and the trajectory's summaries. */
export function projectHistory(
  snaps: readonly XSnapshot[],
  summaries: Record<string, string>,
  provenance: WorldHistory['provenance'],
  opts: ProjectOptions,
): WorldHistory {
  const sorted = [...snaps].sort((a, b) => compareCursor(a.cursor, b.cursor))
  const revisions: Revision[] = sorted.map((s, i) => ({
    id: s.cursor,
    at: s.timestamp,
    label: i === sorted.length - 1 ? 'Now' : s.x?.label && s.x.label !== 'Now' ? s.x.label : shortDate(s.timestamp),
    summary: summaries[s.cursor] ?? s.summary.phase,
  }))
  const snapshots: WorldHistory['snapshots'] = {}
  for (const s of sorted) snapshots[s.cursor] = projectSnapshot(s, opts)
  return { revisions, snapshots, provenance, caseId: sorted[0]?.case_id ?? '' }
}

/** Contract cursors are `<epoch>.<seq>`; compare numerically by epoch, then seq. */
export function compareCursor(a: string, b: string): number {
  const [ea = 0, sa = 0] = a.split('.').map(Number)
  const [eb = 0, sb = 0] = b.split('.').map(Number)
  return ea - eb || sa - sb
}

/** Loads a case's history through the port: trajectory first, then each checkpoint. When the
 * deployment serves no trajectory (an honest adapter limit), falls back to the live head. */
export async function loadCaseHistory(
  port: CaseworkPort,
  caseId: string,
  actor: Actor,
  provenance: WorldHistory['provenance'],
  opts: ProjectOptions,
): Promise<{ history: WorldHistory; raw: XSnapshot[] }> {
  let raw: XSnapshot[]
  let summaries: Record<string, string>
  try {
    const traj = await port.queryTemporalTrajectory(caseId)
    raw = await Promise.all(traj.points.map((p) => port.getSnapshotAt(caseId, p.cursor, actor.actor_id, actor.role)))
    summaries = Object.fromEntries(traj.points.map((p) => [p.cursor, p.summary]))
  } catch {
    // No trajectory in this deployment: one revision, honestly labelled as the current head.
    const head = await port.getSnapshot(caseId, actor.actor_id, actor.role)
    raw = [head]
    summaries = {}
  }
  return { history: projectHistory(raw, summaries, provenance, opts), raw }
}
