import { designSnapshot, focusOf, nowRevision, snapshotOf } from './store'
import type { CognitiveArtifact } from '../ports/contract'
import type { CompareState, Id, Status, SurfaceId, UiState, WorldHistory, WorldObject, WorldSnapshot } from './types'

// What the scene shows, derived from UI state: which world snapshot (live, historical, the case
// template, or the comparison of two snapshots), what is in focus, and which representation.
// Every representation is built from the same source objects; comparison never copies content,
// it pairs the A and B states of each object by id.

export interface CompareSide {
  label: string
  snap: WorldSnapshot
}

export interface ObjectChange {
  change: 'added' | 'removed' | 'changed' | 'same'
  a?: WorldObject
  b?: WorldObject
  /** Human list of what differs, e.g. ["status", "subtitle"]. */
  fields: string[]
}

export interface WorldView {
  snap: WorldSnapshot
  focus: Id | null
  surface: SurfaceId
  design: boolean
  compare: { a: CompareSide; b: CompareSide; changes: Record<Id, ObjectChange> } | null
}

export function sideLabel(h: WorldHistory, cursor: string): string {
  if (cursor === 'draft') return 'Local draft'
  const r = h.revisions.find((x) => x.id === cursor)
  if (!r) return cursor
  return cursor === nowRevision(h) ? 'Live' : r.label
}

export function templateRoot(snap: WorldSnapshot): Id | null {
  return Object.values(snap.objects).find((o) => !o.parent || !snap.objects[o.parent])?.id ?? null
}

function compareSides(s: UiState, c: CompareState): { a: CompareSide; b: CompareSide } | null {
  if (c.source === 'design') {
    const d = s.design
    if (!d) return null
    const get = (cur: string) => (cur === 'draft' ? d.draft : d.history.snapshots[cur]) ?? null
    const a = get(c.a)
    const b = get(c.b)
    return a && b ? { a: { label: sideLabel(d.history, c.a), snap: a }, b: { label: sideLabel(d.history, c.b), snap: b } } : null
  }
  const a = s.history.snapshots[c.a]
  const b = s.history.snapshots[c.b]
  return a && b ? { a: { label: sideLabel(s.history, c.a), snap: a }, b: { label: sideLabel(s.history, c.b), snap: b } } : null
}

const statusKey = (st?: Status) => (st ? `${st.label}|${st.tone}` : '')

export function diffObjects(a: WorldSnapshot, b: WorldSnapshot): Record<Id, ObjectChange> {
  const out: Record<Id, ObjectChange> = {}
  const ids = new Set([...Object.keys(a.objects), ...Object.keys(b.objects)])
  for (const id of ids) {
    const oa = a.objects[id]
    const ob = b.objects[id]
    if (!oa) {
      out[id] = { change: 'added', b: ob, fields: [] }
      continue
    }
    if (!ob) {
      out[id] = { change: 'removed', a: oa, fields: [] }
      continue
    }
    const fields: string[] = []
    if (oa.title !== ob.title) fields.push('name')
    if (oa.subtitle !== ob.subtitle) fields.push('detail')
    if (statusKey(oa.status) !== statusKey(ob.status)) fields.push('status')
    if (oa.metric !== ob.metric) fields.push('count')
    if (oa.parent !== ob.parent) fields.push('place')
    if (!!oa.ghost !== !!ob.ghost || !!oa.dormant !== !!ob.dormant) fields.push('state')
    if ((oa.designItems ?? []).map((i) => i.label).join('|') !== (ob.designItems ?? []).map((i) => i.label).join('|')) fields.push('items')
    out[id] = { change: fields.length ? 'changed' : 'same', a: oa, b: ob, fields }
  }
  return out
}

/** B's world plus A-only objects as ghosts, so removed objects keep a place (identity is never duplicated). */
export function mergeForCompare(a: WorldSnapshot, b: WorldSnapshot): WorldSnapshot {
  const objects: Record<Id, WorldObject> = { ...b.objects }
  for (const [id, o] of Object.entries(a.objects)) {
    if (objects[id]) continue
    const parent = o.parent && objects[o.parent] ? o.parent : o.parent && a.objects[o.parent] ? o.parent : null
    objects[id] = { ...o, parent, ghost: true }
  }
  return { ...b, objects, artifacts: { ...a.artifacts, ...b.artifacts } }
}

export function worldView(s: UiState): WorldView {
  const design = s.mode === 'case-design' && !!s.design
  const base = design ? designSnapshot(s)! : snapshotOf(s)
  const sides = s.compare ? compareSides(s, s.compare) : null
  const compare = sides ? { ...sides, changes: diffObjects(sides.a.snap, sides.b.snap) } : null
  const snap = compare ? mergeForCompare(compare.a.snap, compare.b.snap) : base
  const focus = design ? templateRoot(snap) : focusOf(s)
  return { snap, focus, surface: compare ? 'compare' : design ? 'orbital' : s.surface, design, compare }
}

/** Artifact descriptor by ref: the shown world first, then history (newest first), then the design world. */
export function findArtifact(s: UiState, ref: string): CognitiveArtifact | null {
  const shown = worldView(s).snap.artifacts[ref]
  if (shown) return shown
  for (let i = s.history.revisions.length - 1; i >= 0; i--) {
    const a = s.history.snapshots[s.history.revisions[i]!.id]?.artifacts[ref]
    if (a) return a
  }
  return s.design ? (Object.values(s.design.history.snapshots).find((x) => x.artifacts[ref])?.artifacts[ref] ?? null) : null
}
