import type { ObjectChange } from '../model/view'
import type {
  BeatOverrides,
  Camera,
  Id,
  LayoutResult,
  Link,
  Orbit,
  Placement,
  Status,
  SurfaceId,
  Vec3,
  WorldObject,
  WorldSnapshot,
} from '../model/types'

// Pure layout: (world, surface, focus, overrides) → placements keyed by object id.
//
// Geometry is authored in "mock pixels" (the 1672×941 reference images) relative to the
// current center of gravity, then converted to world units with the parent's frame scale.
// Every level of the hierarchy is 10× smaller than its parent, so entering an object is a
// real camera zoom and zooming without clicking reveals the same nested world.

export const DESIGN_W = 1672
export const DESIGN_H = 941
/** Focal point of the viewport as a fraction of height (the camera target lands here). */
export const FOCAL_Y = 0.46
const FRAME_RATIO = 0.1

export interface LayoutInput {
  snap: WorldSnapshot
  focus: Id | null
  surface: SurfaceId
  overrides: BeatOverrides
  /** Screen px per mock px (fit of the world viewport to the design size). */
  fit: number
  /** Object awaiting a human decision (judgment surface) and the action that asked for it. */
  judgment?: { object: Id; label: string } | null
  /** Dense workbench mode: the center carries its title inside the void (mock 11). */
  workbench?: boolean
  /** Case design: the template world, glyph nodes around an in-void title (mock 12). */
  design?: boolean
  /** An explanation is playing: consequences in the causal chain wait until a beat reveals them. */
  narrating?: boolean
  /** Comparison representation: per-object change between side A and side B. */
  compare?: { changes: Record<Id, ObjectChange>; aLabel: string; bLabel: string } | null
}

// ---------------------------------------------------------------------------
// Presentation constants (mock px). Positions of particular objects are data: each object's
// contract `spatial_layout` is its slot inside its parent's frame. Nothing here knows ids.

interface Slot {
  x: number
  y: number
  size: number
  label?: Placement['labelSide']
}

const HOME_CORE_SIZE = 194
const HOME_CENTER_Y = 435
/** Default diameters when the data gives a position but no radius. */
const DEFAULT_SIZE: Record<string, number> = { category: 12, case: 60, facet: 66, item: 56, person: 40, run: 50, core: 194 }

/** Members without an authored slot take these open spots around a case (then a far ring). */
const EXTRA_CASE_SLOTS: Slot[] = [
  { x: -610, y: 40, size: 58 },
  { x: 610, y: -250, size: 58 },
  { x: 30, y: 300, size: 58 },
  { x: -300, y: -300, size: 56 },
]
const CASE_CENTER_SIZE = 165
const CASE_CENTER_Y = 415

/** Causal representation (mock 09): the chain reads left to right along a shallow arch. */
const CAUSAL_CENTER_Y = 495
const CAUSAL_SPAN = { x0: -620, x1: 707 }
const CAUSAL_ROLE: Record<string, string> = {
  expected: 'Intended path',
  assumption: 'Key assumption',
  condition: 'Real-world condition',
  missing: 'Unmet requirement',
  observed: 'Failure observed',
  dependency: 'Dependency',
  consequence: 'Consequence',
}

/** Workbench rails compress the world to ~0.655 (see runtime view()); design offsets are authored for the final screen. */
const WORKBENCH_COMPRESS = 0.655

/** Mock 10: the decision comes forward right of the center; everything else fades but stays legible. */
const JUDGMENT_CENTER = { x: 565, y: 440 }
const JUDGED_SLOT: Slot = { x: 410, y: 15, size: 100, label: 'above' }
const JUDGMENT_FADED: Slot[] = [
  { x: -125, y: -262, size: 46 },
  { x: -405, y: -150, size: 46 },
  { x: -488, y: 60, size: 46 },
  { x: -297, y: 280, size: 46 },
  { x: 253, y: 270, size: 40 },
]

/** Where grandchildren go when a beat brings them forward in the orbital arrangement. */
const FORWARD_SLOTS: Slot[] = [
  { x: 120, y: -165, size: 60, label: 'right' },
  { x: -455, y: 45, size: 64, label: 'left' },
  { x: -170, y: 215, size: 60, label: 'left' },
  { x: 150, y: 215, size: 60, label: 'right' },
]

/** Receded objects become out-of-focus spheres near the edges (mock 05/09 bokeh). */
const RECEDE_SPOTS: { x: number; y: number; z: number }[] = [
  { x: -835, y: 150, z: 420 },
  { x: 700, y: -375, z: -260 },
  { x: -680, y: -45, z: -240 },
  { x: 800, y: 0, z: 300 },
  { x: -550, y: 385, z: -200 },
  { x: 690, y: 290, z: -220 },
  { x: -40, y: -390, z: -300 },
  { x: 500, y: 405, z: -200 },
]

// ---------------------------------------------------------------------------
// Hierarchy helpers

export function childrenOf(snap: WorldSnapshot, id: Id): WorldObject[] {
  const out: WorldObject[] = []
  for (const o of Object.values(snap.objects)) if (o.parent === id) out.push(o)
  return out.sort((a, b) => b.salience - a.salience || a.id.localeCompare(b.id))
}

function hash(s: string): number {
  let h = 2166136261
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619)
  return (h >>> 0) / 4294967295
}

/** Mock-px offset of a child inside its parent's frame: the object's own spatial_layout, else a generic spot. */
function slotFor(snap: WorldSnapshot, parent: WorldObject, child: WorldObject, index: number, count: number): Slot {
  if (child.slot) return { ...child.slot, size: child.slot.size || DEFAULT_SIZE[child.kind] || 56 }
  if (parent.kind === 'core') return ringSlot(index, count, 470, 250, 12)
  if (parent.kind === 'case') {
    const free = childrenOf(snap, parent.id).filter((c) => !c.slot)
    const i = free.findIndex((c) => c.id === child.id)
    return EXTRA_CASE_SLOTS[i] ?? ringSlot(i, free.length, 760, 380, 52, 0.4)
  }
  return ringSlot(index, count, 520, 270, parent.kind === 'category' ? 50 : 70, hash(parent.id) * Math.PI)
}

function ringSlot(i: number, n: number, rx: number, ry: number, size: number, phase = -Math.PI / 2): Slot {
  const a = phase + (i / Math.max(n, 1)) * Math.PI * 2
  return { x: Math.cos(a) * rx, y: Math.sin(a) * ry, size }
}

export interface Canonical {
  pos: Vec3
  /** World units per mock px for this object's children. */
  frame: number
  size: number
  label: Placement['labelSide']
  depth: number
}

/** Canonical world positions for every object (the nested world as it exists before any surface rearranges it). */
export function canonical(snap: WorldSnapshot): Map<Id, Canonical> {
  const out = new Map<Id, Canonical>()
  const roots = Object.values(snap.objects).filter((o) => !o.parent || !snap.objects[o.parent])
  const visit = (o: WorldObject, c: Canonical) => {
    out.set(o.id, c)
    const kids = childrenOf(snap, o.id)
    kids.forEach((k, i) => {
      const s = slotFor(snap, o, k, i, kids.length)
      visit(k, {
        pos: { x: c.pos.x + s.x * c.frame, y: c.pos.y + s.y * c.frame, z: 0 },
        frame: c.frame * FRAME_RATIO,
        // A case is sized as the singularity it becomes when entered, so free zoom and focus agree.
        size: k.kind === 'case' ? CASE_CENTER_SIZE * c.frame * FRAME_RATIO : s.size * c.frame,
        label: s.label ?? 'right',
        depth: c.depth + 1,
      })
    })
  }
  roots.forEach((r, i) =>
    visit(r, { pos: { x: i * 4000, y: 0, z: 0 }, frame: 1, size: r.kind === 'core' ? HOME_CORE_SIZE : 40, label: 'right', depth: 0 }),
  )
  return out
}

function ancestors(snap: WorldSnapshot, id: Id): Id[] {
  const out: Id[] = []
  let p = snap.objects[id]?.parent
  while (p && snap.objects[p]) {
    out.push(p)
    p = snap.objects[p]!.parent
  }
  return out
}

function isAttention(s?: Status) {
  return s?.tone === 'attention' || s?.tone === 'critical'
}

// ---------------------------------------------------------------------------

export function layout(input: LayoutInput): LayoutResult {
  const { snap, surface, overrides, fit } = input
  const canon = canonical(snap)
  const coreId = Object.values(snap.objects).find((o) => o.kind === 'core')?.id ?? Object.keys(snap.objects)[0]!
  const focus = input.focus && snap.objects[input.focus] ? input.focus : null
  const centerId = focus ?? coreId
  const center = canon.get(centerId)!
  const frame = center.frame
  const placements: Record<Id, Placement> = {}
  const links: Link[] = []
  const orbits: Orbit[] = []
  const dim = new Set(overrides.dim)
  const reveal = new Set(overrides.reveal)

  const at = (dx: number, dy: number, z = 0): Vec3 => ({ x: center.pos.x + dx * frame, y: center.pos.y + dy * frame, z })

  // The center of gravity.
  const centerObj = snap.objects[centerId]!
  const centerSize = centerObj.kind === 'core' ? HOME_CORE_SIZE : CASE_CENTER_SIZE
  placements[centerId] = input.workbench
    ? { pos: center.pos, size: centerSize * 1.55 * frame, emphasis: 1, center: true, labelInside: true }
    : { pos: center.pos, size: centerSize * frame, emphasis: 1, center: true }

  let centerScreenY = centerObj.kind === 'core' ? HOME_CENTER_Y : CASE_CENTER_Y

  if (input.design) {
    const k = frame / WORKBENCH_COMPRESS
    placements[centerId] = { pos: center.pos, size: 205 * k, emphasis: 1, center: true, labelInside: true }
    const kids = childrenOf(snap, centerId)
    kids.forEach((c, i) => {
      const slot = c.slot ? { ...c.slot, size: c.slot.size || 58 } : ringSlot(i, kids.length, 360, 240, 58)
      placements[c.id] = { pos: { x: center.pos.x + slot.x * k, y: center.pos.y + slot.y * k, z: 0 }, size: slot.size * k, emphasis: 1, labelSide: 'right' }
      links.push({ id: `spoke:${c.id}`, from: centerId, to: c.id, style: 'spoke', tone: c.accent === 'orange' ? 'attention' : undefined })
    })
    orbits.push(
      { cx: center.pos.x, cy: center.pos.y, rx: 410 * k, ry: 330 * k, rot: 0, opacity: 0.35 },
      { cx: center.pos.x, cy: center.pos.y, rx: 250 * k, ry: 220 * k, rot: 0, opacity: 0.25 },
    )
    applyCompare(input, placements)
    // Sit higher than the default focal point, clear of the composer (mock 12).
    return { placements, orbits, links, camera: cameraFor(center.pos, frame, fit, FOCAL_Y * DESIGN_H - 95), center: centerId }
  }
  const judged = input.judgment && snap.objects[input.judgment.object] ? input.judgment.object : null
  const causal = surface === 'causal' ? causalMembers(snap, centerId) : []
  if (surface === 'judgment' && judged && judged !== centerId) {
    const j = snap.objects[judged]!
    placements[judged] = {
      pos: at(JUDGED_SLOT.x, JUDGED_SLOT.y, 40),
      size: JUDGED_SLOT.size * frame,
      emphasis: 1,
      labelSide: 'above',
      role: `1. ${input.judgment!.label}`,
      title: j.title,
      subtitle: j.subtitle,
      status: { label: 'Needs your judgment', tone: 'attention' },
    }
    links.push({ id: `spoke:${judged}`, from: centerId, to: judged, style: 'spoke', tone: 'attention' })
    childrenOf(snap, centerId)
      .filter((k) => k.id !== judged)
      .slice(0, JUDGMENT_FADED.length)
      .forEach((k, i) => {
        const slot = JUDGMENT_FADED[i]!
        placements[k.id] = { pos: at(slot.x, slot.y, -40), size: slot.size * frame, emphasis: 0.42, labelSide: 'right', role: 'faded' }
      })
    orbits.push(
      { cx: center.pos.x, cy: center.pos.y, rx: 470 * frame, ry: 330 * frame, rot: 0.1, opacity: 0.25 },
      { cx: center.pos.x + 410 * frame, cy: center.pos.y + 15 * frame, rx: 62 * frame, ry: 44 * frame, rot: -0.3, opacity: 0.5 },
    )
    const camera = cameraFor(center.pos, frame, fit, JUDGMENT_CENTER.y)
    camera.x += ((DESIGN_W / 2 - JUDGMENT_CENTER.x) * fit) / camera.zoom
    return { placements, orbits, links, camera, center: centerId }
  } else if (causal.length >= 2) {
    // The same source objects, re-read as a causal chain (their causal representation from the data).
    centerScreenY = CAUSAL_CENTER_Y
    const shown = causal.filter((o) => !(input.narrating && o.causal!.role === 'observed' && !reveal.has(o.id)))
    const n = causal.length
    shown.forEach((o) => {
      const c = o.causal!
      const idx = causal.indexOf(o)
      const t = n > 1 ? idx / (n - 1) : 0.5
      const x = CAUSAL_SPAN.x0 + (CAUSAL_SPAN.x1 - CAUSAL_SPAN.x0) * t
      const y = 40 - 290 * Math.sin(Math.PI * t)
      placements[o.id] = {
        pos: at(x, y),
        size: (c.role === 'observed' ? 104 : 88) * frame,
        emphasis: dim.has(o.id) ? 0.3 : 1,
        labelSide: 'above',
        role: `${c.order}. ${CAUSAL_ROLE[c.role] ?? c.role}`,
        title: c.label ?? o.title,
        subtitle: c.explanation ?? o.subtitle,
        status: c.badge ? { label: c.badge, tone: c.tone ?? o.status?.tone ?? 'muted' } : o.status,
      }
    })
    // Links: the backend's relationships between chain members, else the reading order.
    for (let i = 0; i + 1 < shown.length; i++) {
      const from = shown[i]!.id
      const to = shown[i + 1]!.id
      const rel = snap.relationships.find((r) => r.from === from && r.to === to) ?? snap.relationships.find((r) => r.from === to && r.to === from)
      const target = snap.objects[to]!
      const critical = isAttention(target.status) || target.causal?.tone === 'critical'
      links.push({ id: rel?.id ?? `causal:${from}>${to}`, from, to, label: rel?.label, style: 'causal', tone: critical ? 'critical' : undefined })
    }
    orbits.push(
      { cx: center.pos.x - 60 * frame, cy: center.pos.y + 20 * frame, rx: 820 * frame, ry: 360 * frame, rot: 0.05, opacity: 0.35 },
      { cx: center.pos.x, cy: center.pos.y, rx: 330 * frame, ry: 150 * frame, rot: 0, opacity: 0.25 },
    )
  } else {
    // Orbital (default). Children of the center on their authored slots.
    const kids = childrenOf(snap, centerId)
    let forwardUsed = 0
    for (const k of kids) {
      const c = canon.get(k.id)!
      let pos = c.pos
      let size = c.size
      let emphasis = 1
      if (reveal.has(k.id) && centerObj.kind !== 'core') {
        // "Moves forward": toward the center and the viewer.
        pos = { x: lerp(c.pos.x, center.pos.x, 0.22), y: lerp(c.pos.y, center.pos.y, 0.22), z: 90 }
        size *= 1.22
      }
      if (dim.has(k.id)) {
        pos = { x: lerp(c.pos.x, center.pos.x, -0.12), y: lerp(c.pos.y, center.pos.y, -0.12), z: -120 }
        emphasis = 0.28
      }
      if (input.workbench) {
        // Between rails the arrangement tightens horizontally (mock 11).
        pos = { x: center.pos.x + (pos.x - center.pos.x) * 0.74, y: center.pos.y + (pos.y - center.pos.y) * 1.0, z: pos.z }
      }
      placements[k.id] = { pos, size, emphasis, labelSide: c.label }
      if (centerObj.kind !== 'core') {
        links.push({ id: `spoke:${k.id}`, from: centerId, to: k.id, style: 'spoke', tone: isAttention(k.status) ? 'critical' : undefined })
      }
      // Grandchildren: satellites (the renderer shows them once the parent is large enough).
      for (const g of childrenOf(snap, k.id)) {
        const gc = canon.get(g.id)!
        if (reveal.has(g.id)) {
          // Brought forward out of its parent into a free slot near the center (identity kept).
          const slot = FORWARD_SLOTS[forwardUsed++ % FORWARD_SLOTS.length]!
          placements[g.id] = {
            pos: at(slot.x, slot.y, 60),
            size: slot.size * frame,
            emphasis: dim.has(g.id) ? 0.3 : 1,
            labelSide: slot.label,
          }
        } else {
          placements[g.id] = { pos: gc.pos, size: gc.size, emphasis: dim.has(g.id) ? 0.3 : 1, satellite: true, labelSide: gc.label }
        }
        // Great-grandchildren only matter when zoomed deep without focusing.
        for (const gg of childrenOf(snap, g.id)) {
          const ggc = canon.get(gg.id)!
          placements[gg.id] = { pos: ggc.pos, size: ggc.size, emphasis: 1, satellite: true, labelSide: ggc.label }
        }
      }
    }
    if (centerObj.kind === 'core') {
      orbits.push(
        { cx: 10, cy: 25, rx: 485, ry: 235, rot: -0.2, opacity: 0.5 },
        { cx: -20, cy: 10, rx: 370, ry: 300, rot: 0.62, opacity: 0.4 },
        { cx: 300, cy: -130, rx: 250, ry: 150, rot: 0.5, opacity: 0.25 },
      )
    } else {
      orbits.push(
        { cx: center.pos.x - 40 * frame, cy: center.pos.y + 30 * frame, rx: 570 * frame, ry: 280 * frame, rot: -0.03, opacity: 0.55 },
        { cx: center.pos.x, cy: center.pos.y + 10 * frame, rx: 300 * frame, ry: 145 * frame, rot: 0, opacity: 0.3 },
        { cx: center.pos.x + 20 * frame, cy: center.pos.y + 30 * frame, rx: 900 * frame, ry: 430 * frame, rot: 0.12, opacity: 0.3 },
      )
    }
  }

  // Revealed relationships (beats) on top of the arrangement.
  for (const rid of overrides.relationships) {
    const rel = snap.relationships.find((r) => r.id === rid)
    if (!rel || links.some((l) => l.id === rid) || !placements[rel.from] || !placements[rel.to]) continue
    links.push({ id: rel.id, from: rel.from, to: rel.to, label: rel.label, style: 'relation' })
  }

  // Everything outside the focus subtree recedes to out-of-focus spheres at the edges.
  if (focus) {
    const anc = ancestors(snap, focus)
    const receders: Id[] = []
    // Siblings first, then ancestors' siblings, then any remaining focus-level facets not placed.
    const parent = snap.objects[focus]!.parent
    if (parent) for (const s of childrenOf(snap, parent)) if (s.id !== focus) receders.push(s.id)
    for (const a of anc) {
      const ap = snap.objects[a]!.parent
      if (ap) for (const s of childrenOf(snap, ap)) if (s.id !== a && !receders.includes(s.id)) receders.push(s.id)
    }
    if (causal.length >= 2) for (const k of childrenOf(snap, centerId)) if (!placements[k.id]) receders.unshift(k.id)
    receders.slice(0, RECEDE_SPOTS.length).forEach((id, i) => {
      const spot = RECEDE_SPOTS[i]!
      const o = snap.objects[id]!
      placements[id] = {
        pos: at(spot.x, spot.y, spot.z),
        size: (o.kind === 'category' ? 50 : 42) * frame,
        emphasis: 0.55,
        satellite: false,
        labelSide: 'right',
      }
      // Receded objects never show labels: the renderer reads this flag.
      placements[id]!.role = 'receded'
    })
  }

  // Narration highlights and annotations are read by the renderer from the placement.
  for (const id of overrides.highlight) if (placements[id]) placements[id] = { ...placements[id]!, highlight: true }
  for (const a of overrides.annotations) if (placements[a.target]) placements[a.target] = { ...placements[a.target]!, annotation: a.label }
  applyCompare(input, placements)

  const camera = cameraFor(center.pos, frame, fit, centerScreenY)
  return { placements, orbits, links, camera, center: centerId }
}

/** Objects under the center (any depth ≤ 3) that carry a causal representation, in reading order. */
export function causalMembers(snap: WorldSnapshot, centerId: Id): WorldObject[] {
  const out: WorldObject[] = []
  const walk = (id: Id, depth: number) => {
    for (const k of childrenOf(snap, id)) {
      if (k.causal) out.push(k)
      if (depth < 3) walk(k.id, depth + 1)
    }
  }
  walk(centerId, 1)
  return out.sort((a, b) => a.causal!.order - b.causal!.order)
}

/** Comparison representation: annotate placed objects with their A→B change; unchanged ones recede. */
function applyCompare(input: LayoutInput, placements: Record<Id, Placement>) {
  const cmp = input.compare
  if (!cmp) return
  // Changes below what is shown are summarised on the shown ancestor ("3 changes inside").
  const inside: Record<Id, number> = {}
  for (const [id, ch] of Object.entries(cmp.changes)) {
    if (ch.change === 'same') continue
    const p = placements[id]
    if (p && !p.satellite && !p.center) continue
    let anc = (ch.b ?? ch.a)?.parent
    while (anc && !(placements[anc] && !placements[anc]!.satellite)) anc = (cmp.changes[anc]?.b ?? cmp.changes[anc]?.a)?.parent ?? null
    if (anc && !placements[anc]!.center) inside[anc] = (inside[anc] ?? 0) + 1
  }
  for (const [id, p] of Object.entries(placements)) {
    const ch = cmp.changes[id]
    if (!ch || p.center || p.role === 'receded' || p.satellite) continue
    const n = inside[id] ?? 0
    placements[id] = {
      ...p,
      emphasis: ch.change === 'same' && !n ? Math.min(p.emphasis, 0.45) : ch.change === 'removed' ? 0.6 : 1,
      compare: {
        change: ch.change,
        inside: n || undefined,
        fields: ch.fields,
        a: ch.a?.status,
        b: ch.b?.status,
        aTitle: ch.a?.subtitle ?? ch.a?.metric,
        bTitle: ch.b?.subtitle ?? ch.b?.metric,
      },
    }
  }
}

export function cameraFor(pos: Vec3, frame: number, fit: number, centerScreenY: number): Camera {
  const zoom = fit / frame
  // Put the center at its designed screen height: sy = fy + (y - cam.y) * zoom.
  const dy = (centerScreenY - FOCAL_Y * DESIGN_H) * fit
  return { x: pos.x, y: pos.y - dy / zoom, zoom, tilt: 0, roll: 0 }
}

function lerp(a: number, b: number, t: number) {
  return a + (b - a) * t
}
