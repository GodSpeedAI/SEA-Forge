import type { WorldHistory, WorldObject, WorldSnapshot } from '../model/types'
import { templateRoot } from '../model/view'

// Case Design over the template world: read its structure into the panel model, and make local
// proposals (reorder stages, change required evidence) as a draft snapshot. Proposals never touch
// the published versions; there is no contract operation to submit them yet (seam S4).

type Items = NonNullable<WorldObject['designItems']>
type MutableItems = { id: string; label: string; detail?: string; required?: boolean }[]

/** The node that carries a kind of design structure, identified by its design glyph. */
const nodeWith = (snap: WorldSnapshot, icon: WorldObject['icon']) =>
  Object.values(snap.objects).find((o) => o.icon === icon && o.designItems)

export interface DesignModel {
  name: string
  versionLabel: string
  goal: string
  description: string
  stages: MutableItems
  roles: MutableItems
  evidence: MutableItems
}

export function designModel(snap: WorldSnapshot, versionLabel: string): DesignModel {
  const root = snap.objects[templateRoot(snap) ?? ''] ?? Object.values(snap.objects)[0]
  const goal = Object.values(snap.objects).find((o) => o.icon === 'target')
  return {
    name: root?.title ?? 'Case',
    versionLabel,
    goal: goal?.subtitle ?? '',
    description: root?.subtitle ?? '',
    stages: [...(nodeWith(snap, 'layers')?.designItems ?? [])],
    roles: [...(nodeWith(snap, 'people')?.designItems ?? [])],
    evidence: [...(nodeWith(snap, 'doc')?.designItems ?? [])],
  }
}

function editItems(snap: WorldSnapshot, itemId: string, edit: (items: Items) => Items): WorldSnapshot {
  const owner = Object.values(snap.objects).find((o) => o.designItems?.some((i) => i.id === itemId))
  if (!owner) return snap
  const designItems = edit(owner.designItems!)
  return { ...snap, revision: 'draft', objects: { ...snap.objects, [owner.id]: { ...owner, designItems, subtitle: summarize(owner, designItems) } } }
}

function summarize(owner: WorldObject, items: Items): string | undefined {
  if (owner.icon === 'layers') return `${items.length} stages`
  if (owner.icon === 'doc') return `${items.filter((i) => i.required).length} required artifacts`
  return owner.subtitle
}

export function moveItem(snap: WorldSnapshot, itemId: string, dir: -1 | 1): WorldSnapshot {
  return editItems(snap, itemId, (items) => {
    const i = items.findIndex((x) => x.id === itemId)
    const j = i + dir
    if (i < 0 || j < 0 || j >= items.length) return items
    const next = [...items]
    ;[next[i], next[j]] = [next[j]!, next[i]!]
    return next
  })
}

export function toggleRequired(snap: WorldSnapshot, itemId: string): WorldSnapshot {
  return editItems(snap, itemId, (items) => items.map((x) => (x.id === itemId ? { ...x, required: !x.required } : x)))
}

const signature = (snap: WorldSnapshot) =>
  JSON.stringify(Object.values(snap.objects).map((o) => [o.id, o.designItems?.map((i) => [i.id, !!i.required])]))

/** True when the proposal differs from the latest published version. */
export function isDirty(draft: WorldSnapshot | null, h: WorldHistory): boolean {
  if (!draft) return false
  return signature(draft) !== signature(h.snapshots[h.revisions.at(-1)!.id]!)
}
