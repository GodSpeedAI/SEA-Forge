import {
  clampTilt,
  flightDuration,
  panBy,
  project,
  sampleFlight,
  zoomAt,
  type Flight,
  type Viewport,
} from '../camera/camera'
import { DESIGN_H, DESIGN_W, FOCAL_Y, layout } from '../layout/layout'
import { expandedArtifact, focusOf, type Store } from '../model/store'
import type { Camera, Id, LayoutResult, Link, Lod, Placement, UiState, Vec3 } from '../model/types'
import type { CoreCanvas, Singularity } from './coreCanvas'
import { worldView, type WorldView } from '../model/view'
import { LodTracker } from './lod'

// The scene runtime owns everything that changes every frame: the camera, per-object tweens,
// LOD, and the imperative writes to the DOM world layer, the SVG link layer and the WebGL canvas.
// React renders structure (which nodes exist, which representation) and reads from here.

interface Tween {
  x: number
  y: number
  z: number
  size: number
  emph: number
  /** 0..1 fade for entering/exiting */
  presence: number
  exiting: boolean
  center: number
}

export interface NodeFrame {
  sx: number
  sy: number
  /** apparent diameter px */
  d: number
  opacity: number
  visible: boolean
  lod: Lod
}

const DOCK_FRACTION = 0.34
const MIN_DOCK = 420
const ZOOM_LIMITS = { min: 0.35, max: 4000 }
/** A parent must appear at least this large before its members show as satellites. */
const SATELLITE_PARENT_PX = 100

export class SceneRuntime {
  camera: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  flight: Flight | null = null
  result!: LayoutResult
  readonly lod = new LodTracker()
  readonly frames = new Map<Id, NodeFrame>()
  private tweens = new Map<Id, Tween>()
  private nodeEls = new Map<Id, HTMLElement>()
  private excerptEls = new Map<Id, HTMLElement>()
  private svg: SVGSVGElement | null = null
  private core: CoreCanvas | null = null
  private listeners = new Set<() => void>()
  private version = 0
  private w = DESIGN_W
  private h = DESIGN_H
  private dockT = 0
  private wake = 0
  private lastInput = 0
  private userMoved = false
  private raf = 0
  private last = 0
  private t0 = performance.now()
  private unsub: (() => void) | null = null

  constructor(private store: Store) {
    this.relayout(store.getState())
    this.camera = { ...this.result.camera }
    this.unsub = store.listen((s, prev) => this.onState(s, prev))
  }

  // -------------------------------------------------------------------------
  // Structure subscription for React

  subscribe = (fn: () => void) => {
    this.listeners.add(fn)
    return () => this.listeners.delete(fn)
  }
  getVersion = () => this.version
  private bump() {
    this.version++
    for (const fn of this.listeners) fn()
  }

  /** Ids React should render (placed + still fading out). */
  renderIds(): Id[] {
    return [...this.tweens.keys()]
  }
  placement(id: Id): Placement | undefined {
    return this.result.placements[id]
  }

  registerNode = (id: Id, el: HTMLElement | null) => {
    if (el) this.nodeEls.set(id, el)
    else this.nodeEls.delete(id)
  }
  registerExcerpt = (id: Id, el: HTMLElement | null) => {
    if (el) this.excerptEls.set(id, el)
    else this.excerptEls.delete(id)
  }
  attachSvg(svg: SVGSVGElement | null) {
    this.svg = svg
  }
  attachCore(core: CoreCanvas | null) {
    this.core = core
  }

  // -------------------------------------------------------------------------
  // Viewport

  private fullFit() {
    return Math.min(this.w / DESIGN_W, this.h / DESIGN_H)
  }
  private dockWidth() {
    return Math.max(MIN_DOCK, Math.round(this.w * DOCK_FRACTION))
  }
  /** Viewport and effective camera with screen insets (dock, workbench rails) applied. */
  private view(): { vp: Viewport; cam: Camera } {
    const { l, r, t } = this.inset
    const worldW = this.w - l - r
    const worldH = this.h - t
    // Mock 07: the world compresses to ~0.72 of its size beside the dock; rails compress it similarly.
    const rails = this.inset.l > 1 ? 0.8 : 0.915
    const zoomMul = Math.min(1, worldW / (this.w * rails), worldH / this.h)
    return {
      vp: { w: l + worldW, h: this.h, fx: l + worldW / 2, fy: t + worldH * FOCAL_Y },
      cam: { ...this.camera, zoom: this.camera.zoom * zoomMul },
    }
  }

  /** Animated screen insets taken by screen-space panels. */
  private inset = { l: 0, r: 0, t: 0 }
  private targetInset(s: UiState) {
    const f = this.fullFit()
    if (s.mode !== 'world') return { l: 185 * f, r: 485 * f, t: 150 * f }
    return { l: 0, r: expandedArtifact(s) ? this.dockWidth() : 0, t: 0 }
  }

  resize(w: number, h: number) {
    this.w = w
    this.h = h
    this.core?.resize(w, h)
    this.relayout(this.store.getState())
    if (!this.userMoved && !this.flight) this.camera = { ...this.result.camera }
  }

  /** Snapshot of the camera for artifact expand/collapse. */
  currentCamera(): Camera {
    return { ...(this.flight ? this.flight.to : this.camera) }
  }

  // -------------------------------------------------------------------------
  // State → layout → tween targets

  private viewOf: { s: UiState | null; v: WorldView | null } = { s: null, v: null }
  /** What is shown: live/historical world, the case template, or a comparison (memoised per state). */
  worldView(s: UiState): WorldView {
    if (this.viewOf.s !== s) this.viewOf = { s, v: worldView(s) }
    return this.viewOf.v!
  }
  snap(s: UiState) {
    return this.worldView(s).snap
  }

  private relayout(s: UiState) {
    const fit = this.fullFit()
    const v = this.worldView(s)
    this.result = layout({
      snap: v.snap,
      focus: v.focus,
      surface: v.surface,
      overrides: s.overrides,
      fit,
      judgment: s.judgment ? { object: s.judgment.object, label: s.judgment.action.label } : null,
      workbench: s.mode === 'execution-inspect',
      design: v.design,
      narrating: !!s.narrative && s.narrative.status !== 'done',
      compare: v.compare ? { changes: v.compare.changes, aLabel: v.compare.a.label, bLabel: v.compare.b.label } : null,
    })
    const placements = this.result.placements
    let structural = false
    for (const [id, p] of Object.entries(placements)) {
      const tw = this.tweens.get(id)
      if (!tw) {
        // Enter near the parent's current position so nothing "respawns" out of nowhere.
        const parent = this.snap(s).objects[id]?.parent
        const from = (parent && this.tweens.get(parent)) || null
        this.tweens.set(id, {
          x: from ? from.x : p.pos.x,
          y: from ? from.y : p.pos.y,
          z: p.pos.z,
          size: p.size * 0.4,
          emph: p.emphasis,
          presence: 0,
          exiting: false,
          center: p.center ? 1 : 0,
        })
        structural = true
      } else if (tw.exiting) {
        tw.exiting = false
        structural = true
      }
    }
    for (const [id, tw] of this.tweens) {
      if (!placements[id] && !tw.exiting) {
        tw.exiting = true
        structural = true
      }
    }
    if (structural) this.bump()
    else this.bump() // placements (titles/roles) may have changed
  }

  private onState(s: UiState, prev: UiState) {
    const layoutChanged =
      s.revision !== prev.revision ||
      s.history !== prev.history ||
      s.focusStack !== prev.focusStack ||
      s.surface !== prev.surface ||
      s.overrides !== prev.overrides ||
      s.judgment !== prev.judgment ||
      s.mode !== prev.mode ||
      s.compare !== prev.compare ||
      s.design !== prev.design ||
      (s.narrative?.status === 'done') !== (prev.narrative?.status === 'done') ||
      !s.narrative !== !prev.narrative
    if (layoutChanged) this.relayout(s)
    if (s.cameraRequest !== prev.cameraRequest) {
      if (!layoutChanged) this.relayout(s)
      const to = s.cameraRestore ?? this.result.camera
      this.flyTo(to)
      this.userMoved = false
    }
    if (s.awake && !prev.awake) this.lastInput = performance.now()
  }

  /** Reduced motion: arrive instead of flying; rearrangements settle quickly; the Core barely moves. */
  private reduced = typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches

  flyTo(to: Camera) {
    const from = { ...this.camera }
    if (this.reduced) {
      this.camera = { ...to }
      this.flight = null
      return
    }
    this.flight = { from, to: { ...to }, start: performance.now(), duration: flightDuration(from, to) }
  }

  // -------------------------------------------------------------------------
  // Input

  input = {
    pan: (dx: number, dy: number) => {
      this.flight = null
      this.userMoved = true
      const { cam } = this.view()
      const moved = panBy({ ...this.camera, zoom: cam.zoom }, -dx, -dy)
      this.camera = { ...this.camera, x: moved.x, y: moved.y }
    },
    zoom: (factor: number, sx: number, sy: number) => {
      this.flight = null
      this.userMoved = true
      const { vp, cam } = this.view()
      const mul = cam.zoom / this.camera.zoom
      const z = zoomAt(cam, factor, sx, sy, vp, { min: ZOOM_LIMITS.min * mul, max: ZOOM_LIMITS.max * mul })
      this.camera = { ...z, zoom: z.zoom / mul }
    },
    orbit: (dx: number, dy: number) => {
      this.flight = null
      this.userMoved = true
      this.camera = {
        ...this.camera,
        tilt: Math.min(clampTilt(this.camera.tilt + dy * 0.004), 0.95),
        roll: Math.max(-0.3, Math.min(0.3, this.camera.roll + dx * 0.0015)),
      }
    },
    activity: () => {
      this.lastInput = performance.now()
      if (!this.store.getState().awake) this.store.dispatch({ type: 'wake', awake: true })
    },
  }

  // -------------------------------------------------------------------------
  // Frame loop

  start() {
    const loop = (now: number) => {
      this.raf = requestAnimationFrame(loop)
      this.tick(now)
    }
    this.raf = requestAnimationFrame(loop)
  }

  dispose() {
    cancelAnimationFrame(this.raf)
    this.unsub?.()
  }

  private tick(now: number) {
    // Time-based (not frame-based) so the scene converges in wall time even at low frame rates.
    const dt = Math.min(0.25, this.last ? (now - this.last) / 1000 : 0.016)
    this.last = now
    const s = this.store.getState()

    if (this.flight) {
      const f = sampleFlight(this.flight, now)
      this.camera = f.cam
      if (f.done) this.flight = null
    }
    // Idle: the world sleeps again a while after the pointer stops.
    if (s.awake && now - this.lastInput > 9000) this.store.dispatch({ type: 'wake', awake: false })
    this.dockT = approach(this.dockT, expandedArtifact(s) ? 1 : 0, dt, 7)
    const ti = this.targetInset(s)
    this.inset = { l: approach(this.inset.l, ti.l, dt, 7), r: approach(this.inset.r, ti.r, dt, 7), t: approach(this.inset.t, ti.t, dt, 7) }
    this.wake = approach(this.wake, s.awake ? 1 : 0, dt, s.awake ? 3 : 0.8)

    // Tweens toward placements.
    const k = 1 - Math.exp(-dt * (this.reduced ? 14 : 4.2))
    let removed = false
    for (const [id, tw] of this.tweens) {
      const p = this.result.placements[id]
      if (p) {
        tw.x += (p.pos.x - tw.x) * k
        tw.y += (p.pos.y - tw.y) * k
        tw.z += (p.pos.z - tw.z) * k
        tw.size += (p.size - tw.size) * k
        tw.emph += (p.emphasis - tw.emph) * k
        tw.presence = approach(tw.presence, 1, dt, 3.5)
        tw.center = approach(tw.center, p.center ? 1 : 0, dt, 2.6)
      } else {
        // Exit toward the parent, shrinking.
        const parent = this.snap(s).objects[id]?.parent
        const pt = parent ? this.tweens.get(parent) : undefined
        if (pt) {
          tw.x += (pt.x - tw.x) * k * 0.6
          tw.y += (pt.y - tw.y) * k * 0.6
        }
        tw.presence = approach(tw.presence, 0, dt, 3.2)
        tw.center = approach(tw.center, 0, dt, 3.2)
        // The Core is the persistent orientation anchor: its node is never unmounted, only hidden.
        if (tw.presence < 0.01 && tw.center < 0.01 && this.snap(s).objects[id]?.kind !== 'core' && s.history.snapshots[s.revision]?.objects[id]?.kind !== 'core') {
          this.tweens.delete(id)
          this.frames.delete(id)
          this.lod.forget(id)
          removed = true
        }
      }
    }

    const { vp, cam } = this.view()
    const t = ((now - this.t0) / 1000) * (this.reduced ? 0.15 : 1)
    const hover = s.hover
    const objects = this.snap(s).objects
    const singularities: Singularity[] = []

    for (const [id, tw] of this.tweens) {
      const p = this.result.placements[id]
      const obj = objects[id]
      // Slow low-amplitude drift keeps the world alive.
      const bob = (p?.satellite ? 0.6 : 1) * (1 + this.wake * 0.8)
      const ph = hashPhase(id)
      const pos: Vec3 = {
        x: tw.x + Math.sin(t * 0.21 + ph) * 2.2 * bob / cam.zoom,
        y: tw.y + Math.cos(t * 0.17 + ph * 1.7) * 1.8 * bob / cam.zoom,
        z: tw.z,
      }
      const pr = project(pos, cam, vp)
      let d = tw.size * pr.scale
      // Dark Home is designed with luminous spheres, not dots (mock 13): a representation choice by theme.
      if (s.theme === 'dark' && obj?.kind === 'category' && !p?.satellite && !p?.role) d = Math.max(d, 46 * this.fullFit() * tw.presence)
      const parentId = obj?.parent
      const parentFrame = parentId ? this.frames.get(parentId) : undefined
      const isCenter = !!p?.center
      const receded = p?.role === 'receded'

      let visible = true
      let opacity = tw.presence * tw.emph
      if (p?.satellite) {
        const parentBig = !!parentFrame && parentFrame.visible && parentFrame.d >= SATELLITE_PARENT_PX
        const attention = obj?.status?.tone === 'attention' || obj?.status?.tone === 'critical'
        // Only direct members of what is on screen surface on wake; deeper things wait for zoom.
        const parentPlaced = parentId ? this.result.placements[parentId] : undefined
        const surfaced = attention && this.wake > 0.05 && !!parentFrame?.visible && !!parentPlaced && !parentPlaced.satellite
        visible = (parentBig || surfaced || hover === id) && d >= 0.4
        if (!parentBig && surfaced) opacity *= this.wake
      }
      if (d < 0.35 && !isCenter) visible = false
      // Off-screen culling with a margin.
      if (pr.sx < -400 || pr.sx > vp.w + 400 || pr.sy < -400 || pr.sy > vp.h + 400) visible = isCenter && d < 6000
      if (receded) opacity *= 0.9

      const lod = isCenter || receded ? 1 : this.lod.get(id, d)
      this.frames.set(id, { sx: pr.sx, sy: pr.sy, d, opacity, visible, lod })

      // Singularity: the center of gravity, or a case zoomed large enough to become one.
      let strength = tw.center
      if (obj?.kind === 'case' && !isCenter) strength = Math.max(strength, smoothstep(95, 170, d) * tw.presence)
      if (strength > 0.01 && visible !== false) {
        // Face-on when the center carries its title inside (workbench modes), lensed edge-on otherwise.
        const incl = obj?.kind === 'core' || p?.labelInside ? 0.28 + cam.tilt * 0.6 : 1.43 + cam.tilt * 0.12
        singularities.push({ x: pr.sx, y: pr.sy, r: d / 2, incl, strength })
      }

      const el = this.nodeEls.get(id)
      if (el) {
        const show = visible && opacity > 0.01
        el.style.display = show ? '' : 'none'
        if (show) {
          el.style.transform = `translate3d(${pr.sx.toFixed(2)}px, ${pr.sy.toFixed(2)}px, 0)`
          el.style.setProperty('--r', `${(Math.max(d, p?.satellite ? 4 : 2) / 2).toFixed(2)}px`)
          el.style.opacity = opacity.toFixed(3)
          el.style.setProperty('--orb', (1 - Math.min(strength, 1)).toFixed(3))
          el.style.setProperty('--wake', this.wake.toFixed(3))
          if (isCenter) el.classList.toggle('huge', d > 460)
          const blur = receded ? Math.min(14, 3 + Math.abs(pos.z) / 45) : 0
          el.style.filter = blur ? `blur(${blur.toFixed(1)}px)` : ''
          el.style.zIndex = String(Math.round(10 + pos.z / 10 + (hover === id ? 5 : 0)))
        }
      }
    }

    // Excerpts ride beside their source object, nudged apart so several can coexist (mock 04).
    const placedCards: { x: number; y: number; w: number; h: number }[] = []
    // Several artifacts at once take the open zones of the composition (mock 04) instead of crowding their sources.
    const ZONES = [
      { x: 1345, y: 92 },
      { x: 1150, y: 332 },
      { x: 58, y: 372 },
      { x: 70, y: 690 },
    ]
    const visibleCards = [...this.excerptEls.values()].filter((e) => {
      const f = e.dataset.source ? this.frames.get(e.dataset.source) : undefined
      return !!f && f.visible
    })
    const zoned = visibleCards.length >= 2 && this.inset.r < 1
    let zoneIndex = 0
    for (const [aid, el] of this.excerptEls) {
      const src = el.dataset.source
      const f = src ? this.frames.get(src) : undefined
      if (!f || !f.visible) {
        el.style.opacity = '0'
        el.style.pointerEvents = 'none'
        continue
      }
      const r = f.d / 2
      const w = el.offsetWidth || 270
      const h = el.offsetHeight || 90
      // Keep clear of the source's label: above-labels push the card up and right,
      // right-labels put it under the label block.
      const side = this.result.placements[src!]?.labelSide ?? 'right'
      let x: number
      let y: number
      if (side === 'above') {
        x = f.sx + r + 60
        y = f.sy - r - h - 70
      } else if (side === 'left') {
        x = f.sx - r - 18 - w
        y = f.sy + r * 0.35 + 34
      } else {
        x = f.sx + r + 18
        y = f.sy + r * 0.35 + 34
      }
      if (x + w > vp.w - 16) x = f.sx - r - 40 - w
      if (zoned) {
        const z = ZONES[zoneIndex++ % ZONES.length]!
        const fit = this.fullFit()
        x = z.x * fit
        y = z.y * fit
      }
      x = Math.max(16, x)
      y = Math.max(70, Math.min(y, vp.h - h - 150))
      for (let guard = 0; guard < 6; guard++) {
        const hit = placedCards.find((c) => x < c.x + c.w + 12 && x + w + 12 > c.x && y < c.y + c.h + 12 && y + h + 12 > c.y)
        if (!hit) break
        const below = hit.y + hit.h + 14
        y = below + h < vp.h - 150 ? below : hit.y - h - 14
        if (y < 70) {
          y = hit.y
          x = hit.x > vp.w / 2 ? hit.x - w - 16 : hit.x + hit.w + 16
        }
      }
      placedCards.push({ x, y, w, h })
      el.style.transform = `translate3d(${x.toFixed(1)}px, ${y.toFixed(1)}px, 0)`
      el.style.opacity = '1'
      el.style.pointerEvents = ''
      el.dataset.ax = String(x)
      el.dataset.ay = String(y)
      void aid
    }

    this.drawSvg(s, cam, vp)

    if (this.core) {
      // The small Core anchor (top-left) appears once you have left Home.
      const away = focusOf(s) ? 1 : 0
      this.anchorT = approach(this.anchorT, away, dt, 2.5)
      if (this.anchorT > 0.01) singularities.push({ x: 72, y: 150, r: 11, incl: 1.25, strength: this.anchorT })
      const c = this.result.placements[this.result.center]
      const cf = this.frames.get(this.result.center)
      const field =
        c && cf
          ? {
              x: cf.sx,
              y: cf.sy,
              scale: this.fieldScale(cam),
              opacity: (focusOf(s) ? 0.95 : 0.14 + 0.2 * this.wake) * (1 - this.dockT * 0.2),
            }
          : null
      this.core.adapt(dt * 1000, now)
      this.core.render(t, s.theme, this.wake, singularities, field)
    }

    if (removed || this.lod.consumeChanged()) this.bump()
    // Verification hook: screenshots wait until a few frames have rendered.
    this.frameCount++
    if (this.frameCount === 30) (window as unknown as { __sceneReady?: boolean }).__sceneReady = true
  }

  private anchorT = 0
  private frameCount = 0

  /** Screen px per mock px at the center's frame (so the dust field zooms with the world). */
  private fieldScale(cam: Camera) {
    const baseZoom = this.result.camera.zoom
    return this.fullFit() * (cam.zoom / baseZoom)
  }

  // -------------------------------------------------------------------------
  // SVG: orbits and links, projected with the same camera.

  private drawSvg(s: UiState, cam: Camera, vp: Viewport) {
    const svg = this.svg
    if (!svg) return
    const orbitsG = svg.querySelector('g.orbits') as SVGGElement
    const linksG = svg.querySelector('g.links') as SVGGElement
    const labelsG = svg.querySelector('g.link-labels') as SVGGElement

    // Orbits
    const orbits = this.result.orbits
    while (orbitsG.childNodes.length < orbits.length) orbitsG.appendChild(svgEl('path'))
    while (orbitsG.childNodes.length > orbits.length) orbitsG.lastChild!.remove()
    orbits.forEach((o, i) => {
      const path = orbitsG.childNodes[i] as SVGPathElement
      let dstr = ''
      const n = 96
      for (let j = 0; j <= n; j++) {
        const a = (j / n) * Math.PI * 2
        const ex = Math.cos(a) * o.rx
        const ey = Math.sin(a) * o.ry
        const x = o.cx + ex * Math.cos(o.rot) - ey * Math.sin(o.rot)
        const y = o.cy + ex * Math.sin(o.rot) + ey * Math.cos(o.rot)
        const pr = project({ x, y, z: 0 }, cam, vp)
        dstr += `${j ? 'L' : 'M'}${pr.sx.toFixed(1)} ${pr.sy.toFixed(1)}`
      }
      path.setAttribute('d', dstr)
      path.setAttribute('class', 'orbit')
      path.style.opacity = String(o.opacity * (0.75 + 0.25 * this.wake))
    })

    // Links: arrangement links + hover reveals + excerpt connectors + dock connector.
    const links: (Link & { x1: number; y1: number; x2: number; y2: number; r1: number; r2: number })[] = []
    const add = (l: Link) => {
      const a = this.frames.get(l.from)
      const b = this.frames.get(l.to)
      if (!a || !b || !a.visible || !b.visible) return
      links.push({ ...l, x1: a.sx, y1: a.sy, x2: b.sx, y2: b.sy, r1: a.d / 2, r2: b.d / 2 })
    }
    for (const l of this.result.links) add(l)
    if (s.hover) {
      for (const r of this.snap(s).relationships) {
        if ((r.from === s.hover || r.to === s.hover) && !links.some((x) => x.id === r.id)) {
          add({ id: r.id, from: r.from, to: r.to, label: r.label, style: 'relation' })
        }
      }
    }
    const extra: string[] = []
    for (const [, el] of this.excerptEls) {
      const src = el.dataset.source
      const f = src ? this.frames.get(src) : undefined
      if (!f || !f.visible || el.style.opacity === '0') continue
      const ax = Number(el.dataset.ax)
      const ay = Number(el.dataset.ay)
      const leftSide = ax < f.sx
      const tx = leftSide ? ax + el.offsetWidth : ax
      const ty = ay + 26
      const sx = f.sx + (leftSide ? -f.d / 2 : f.d / 2) * 0.9
      const sy = f.sy - f.d * 0.2
      extra.push(`M${sx.toFixed(1)} ${sy.toFixed(1)} Q${((sx + tx) / 2).toFixed(1)} ${ty.toFixed(1)} ${tx.toFixed(1)} ${ty.toFixed(1)}`)
    }
    const exp = expandedArtifact(s)
    let dockLine = ''
    if (s.judgment && !exp) {
      const f = this.frames.get(s.judgment.object)
      if (f && f.visible) {
        const px = 1175 * this.fullFit()
        const py = 405 * this.fullFit()
        dockLine = `M${(f.sx + f.d / 2 + 10).toFixed(1)} ${f.sy.toFixed(1)} L${px.toFixed(1)} ${py.toFixed(1)}`
      }
    }
    if (exp && this.dockT > 0.05) {
      const srcId = this.expandedSource?.(exp.id)
      const f = srcId ? this.frames.get(srcId) : undefined
      if (f && f.visible) {
        const edge = vp.w
        const midX = f.sx + (edge - f.sx) * 0.55
        dockLine = `M${(f.sx + f.d / 2).toFixed(1)} ${f.sy.toFixed(1)} L${midX.toFixed(1)} ${(f.sy - 40).toFixed(1)} L${edge.toFixed(1)} ${(f.sy - 70).toFixed(1)}`
      }
    }

    const total = links.length + extra.length + (dockLine ? 1 : 0)
    while (linksG.childNodes.length < total) linksG.appendChild(svgEl('path'))
    while (linksG.childNodes.length > total) linksG.lastChild!.remove()
    const labels: { x: number; y: number; text: string }[] = []
    links.forEach((l, i) => {
      const path = linksG.childNodes[i] as SVGPathElement
      const dx = l.x2 - l.x1
      const dy = l.y2 - l.y1
      const len = Math.hypot(dx, dy) || 1
      const ux = dx / len
      const uy = dy / len
      let d: string
      if (l.style === 'spoke') {
        // From the edge of the center's glow to the part.
        const x1 = l.x1 + ux * l.r1 * 1.05
        const y1 = l.y1 + uy * l.r1 * 1.05
        const x2 = l.x2 - ux * l.r2 * 1.02
        const y2 = l.y2 - uy * l.r2 * 1.02
        d = `M${x1.toFixed(1)} ${y1.toFixed(1)}L${x2.toFixed(1)} ${y2.toFixed(1)}`
      } else {
        const x1 = l.x1 + ux * (l.r1 + 8)
        const y1 = l.y1 + uy * (l.r1 + 8)
        const x2 = l.x2 - ux * (l.r2 + 10)
        const y2 = l.y2 - uy * (l.r2 + 10)
        // Gentle arc bowing upward (perpendicular normal pointing up the screen).
        const nx = uy > 0 || (uy === 0 && ux < 0) ? -uy : uy
        const ny = uy > 0 || (uy === 0 && ux < 0) ? ux : -ux
        const bow = ny <= 0 ? 1 : -1
        const mx = (x1 + x2) / 2 + nx * bow * len * 0.12
        const my = (y1 + y2) / 2 + ny * bow * len * 0.12
        d = `M${x1.toFixed(1)} ${y1.toFixed(1)}Q${mx.toFixed(1)} ${my.toFixed(1)} ${x2.toFixed(1)} ${y2.toFixed(1)}`
        if (l.label) labels.push({ x: 0.25 * x1 + 0.5 * mx + 0.25 * x2, y: 0.25 * y1 + 0.5 * my + 0.25 * y2, text: l.label })
      }
      path.setAttribute('d', d)
      path.setAttribute('class', `link link-${l.style}${l.tone ? ` tone-${l.tone}` : ''}`)
      path.setAttribute('marker-end', l.style === 'causal' ? 'url(#arrow)' : '')
    })
    extra.forEach((d, j) => {
      const path = linksG.childNodes[links.length + j] as SVGPathElement
      path.setAttribute('d', d)
      path.setAttribute('class', 'link link-excerpt')
      path.setAttribute('marker-end', '')
    })
    if (dockLine) {
      const path = linksG.childNodes[total - 1] as SVGPathElement
      path.setAttribute('d', dockLine)
      path.setAttribute('class', 'link link-dock')
      path.setAttribute('marker-end', '')
      path.style.opacity = s.judgment && !exp ? '1' : String(this.dockT)
    }

    while (labelsG.childNodes.length < labels.length) {
      const g = svgEl('g')
      g.appendChild(svgEl('rect'))
      g.appendChild(svgEl('text'))
      labelsG.appendChild(g)
    }
    while (labelsG.childNodes.length > labels.length) labelsG.lastChild!.remove()
    labels.forEach((lb, i) => {
      const g = labelsG.childNodes[i] as SVGGElement
      const text = g.childNodes[1] as SVGTextElement
      if (text.textContent !== lb.text) text.textContent = lb.text
      text.setAttribute('x', lb.x.toFixed(1))
      text.setAttribute('y', (lb.y + 4).toFixed(1))
      const rect = g.childNodes[0] as SVGRectElement
      const w = lb.text.length * 7.2 + 12
      rect.setAttribute('x', (lb.x - w / 2).toFixed(1))
      rect.setAttribute('y', (lb.y - 9).toFixed(1))
      rect.setAttribute('width', w.toFixed(1))
      rect.setAttribute('height', '18')
    })
  }

  /** Set by the app: artifact id → source object id. */
  expandedSource: ((artifactId: Id) => Id | undefined) | null = null
}

const SVG_NS = 'http://www.w3.org/2000/svg'
function svgEl<K extends keyof SVGElementTagNameMap>(tag: K): SVGElementTagNameMap[K] {
  return document.createElementNS(SVG_NS, tag)
}

function approach(v: number, target: number, dt: number, rate: number) {
  return v + (target - v) * (1 - Math.exp(-dt * rate))
}

function smoothstep(a: number, b: number, x: number) {
  const t = Math.max(0, Math.min(1, (x - a) / (b - a)))
  return t * t * (3 - 2 * t)
}

function hashPhase(id: string) {
  let h = 0
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) | 0
  return (h % 1000) / 159.2
}
