import { useEffect, useRef, useSyncExternalStore } from 'react'
import { useArtifact, type ArtifactService } from '../artifacts/service'
import { childrenOf } from '../layout/layout'
import { focusOf, isPast, type Store } from '../model/store'
import type { Id, ObjectAction, OpenArtifact, UiState, ViewSnapshot } from '../model/types'
import { findArtifact } from '../model/view'
import type { CognitiveArtifact } from '../ports/contract'
import { ArtifactExcerpt } from '../ui/ArtifactExcerpt'
import { CoreCanvas } from './coreCanvas'
import { ObjectNode } from './ObjectNode'
import type { SceneRuntime } from './runtime'
import './scene.css'

// The world: one WebGL canvas (Core, particles), one SVG layer (orbits, links), and one DOM
// layer (objects, excerpts). All three are written by the runtime from the same camera.

export interface SceneServices {
  artifacts: ArtifactService
  /** Invoke a backend-listed action on an object (consequential ones open judgment). */
  invoke(object: Id, action: ObjectAction): void
  /** Representation switches offered at the current local center. */
  centerActions(state: UiState): { id: string; label: string; pressed?: boolean; onClick: () => void }[]
}

export function Scene({ store, runtime, state, services }: { store: Store; runtime: SceneRuntime; state: UiState; services: SceneServices }) {
  const rootRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const svgRef = useRef<SVGSVGElement>(null)
  useSyncExternalStore(runtime.subscribe, runtime.getVersion)

  // WebGL + resize
  useEffect(() => {
    const canvas = canvasRef.current!
    let core: CoreCanvas | null = null
    try {
      core = new CoreCanvas(canvas)
    } catch (err) {
      console.warn('WebGL unavailable; Core visual disabled', err)
    }
    runtime.attachCore(core)
    runtime.attachSvg(svgRef.current)
    const onResize = () => runtime.resize(window.innerWidth, window.innerHeight)
    onResize()
    window.addEventListener('resize', onResize)
    runtime.start()
    return () => {
      window.removeEventListener('resize', onResize)
      runtime.dispose()
      core?.dispose()
    }
  }, [runtime])

  // Pointer: drag to pan, shift/right-drag to orbit, wheel/pinch to zoom, click to focus.
  useEffect(() => {
    const root = rootRef.current!
    let drag: { x: number; y: number; sx: number; sy: number; orbit: boolean; id: number; moved: boolean; node: HTMLElement | null } | null = null
    const pauseIfPlaying = () => {
      if (store.getState().narrative?.status === 'playing') store.dispatch({ type: 'pauseNarrative' })
    }
    const onDown = (e: PointerEvent) => {
      // Controls inside the world (chips, pills, excerpt cards) keep their own clicks: no pan capture.
      if ((e.target as HTMLElement).closest('.artifact-excerpt, button, input, textarea, a')) return
      // Remember the node now: pointer capture retargets the matching pointerup to the root.
      const node = (e.target as HTMLElement).closest('[data-node]') as HTMLElement | null
      drag = { x: e.clientX, y: e.clientY, sx: e.clientX, sy: e.clientY, orbit: e.shiftKey || e.button === 2, id: e.pointerId, moved: false, node }
      root.setPointerCapture(e.pointerId)
    }
    const onMove = (e: PointerEvent) => {
      runtime.input.activity()
      if (!drag || e.pointerId !== drag.id) return
      const dx = e.clientX - drag.x
      const dy = e.clientY - drag.y
      drag.x = e.clientX
      drag.y = e.clientY
      if (!drag.moved && Math.hypot(e.clientX - drag.sx, e.clientY - drag.sy) > 4) {
        drag.moved = true
        pauseIfPlaying()
        root.classList.add('dragging')
      }
      if (drag.moved) (drag.orbit ? runtime.input.orbit : runtime.input.pan)(dx, dy)
    }
    const onUp = (e: PointerEvent) => {
      if (!drag || e.pointerId !== drag.id) return
      const wasClick = !drag.moved
      const nodeEl = drag.node
      drag = null
      root.classList.remove('dragging')
      if (!wasClick) return
      const id = nodeEl?.dataset.node
      const s = store.getState()
      if (id && id !== focusOf(s) && !nodeEl!.classList.contains('center')) {
        pauseIfPlaying()
        const v = runtime.worldView(s)
        if (s.mode === 'case-design') {
          store.dispatch({ type: 'designSelect', id })
        } else if ((v.surface === 'causal' || v.surface === 'compare') && s.selection !== id) {
          // In a re-arranged representation a click selects (the selection carries across
          // representations); clicking the selected object again enters it.
          store.dispatch({ type: 'select', id })
        } else if (!v.snap.objects[id]?.ghost || !v.compare) {
          store.dispatch({ type: 'focus', id })
          store.dispatch({ type: 'select', id: null })
        }
      } else if (s.narrative?.status === 'playing') {
        pauseIfPlaying()
      }
    }
    const onWheel = (e: WheelEvent) => {
      e.preventDefault()
      runtime.input.activity()
      pauseIfPlaying()
      const scale = e.deltaMode === 1 ? 16 : 1
      const factor = Math.exp(-e.deltaY * scale * (e.ctrlKey ? 0.01 : 0.0016))
      runtime.input.zoom(factor, e.clientX, e.clientY)
    }
    const onOver = (e: PointerEvent) => {
      const id = ((e.target as HTMLElement).closest('[data-node]') as HTMLElement | null)?.dataset.node ?? null
      if (store.getState().hover !== id) store.dispatch({ type: 'hover', id })
    }
    const onContext = (e: Event) => e.preventDefault()
    root.addEventListener('pointerdown', onDown)
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
    root.addEventListener('pointerover', onOver)
    root.addEventListener('wheel', onWheel, { passive: false })
    root.addEventListener('contextmenu', onContext)
    return () => {
      root.removeEventListener('pointerdown', onDown)
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onUp)
      root.removeEventListener('pointerover', onOver)
      root.removeEventListener('wheel', onWheel)
      root.removeEventListener('contextmenu', onContext)
    }
  }, [runtime, store])

  const snap = runtime.snap(state)
  const home = focusOf(state) === null && state.mode === 'world'
  const past = isPast(state)
  const needs = Object.values(snap.objects).filter((o) => o.kind !== 'category' && o.kind !== 'core' && o.status?.tone === 'critical').length
  const homeResidue = home && state.awake && !state.overrides.caption ? (needs ? `${needs} thing${needs > 1 ? 's' : ''} need${needs > 1 ? '' : 's'} you` : 'All quiet') : null
  const ids = runtime.renderIds()
  const center = runtime.result.center
  const excerpts = state.artifacts
    .filter((a) => a.phase === 'excerpt')
    .map((a) => ({ open: a, descriptor: findArtifact(state, a.id) }))
    .filter((x): x is { open: OpenArtifact; descriptor: CognitiveArtifact } => !!x.descriptor)
  const centerActions = services.centerActions(state)
  // An excerpt rides beside the object it was disclosed from, else beside a visible object it is bound to.
  const anchorOf = (open: OpenArtifact, d: CognitiveArtifact) => {
    if (open.source && snap.objects[open.source]) return open.source
    const bound = Object.values(snap.objects).filter((o) => o.artifacts?.includes(open.id)).map((o) => o.id)
    return bound.find((id) => { const p = runtime.placement(id); return p && !p.satellite && p.role !== 'receded' }) ?? d.boundObject
  }

  return (
    <div ref={rootRef} className={`scene${past ? ' past' : ''}`} aria-label="World">
      <canvas ref={canvasRef} className="core-canvas" aria-hidden />
      <svg ref={svgRef} className="links-layer" aria-hidden>
        <defs>
          <marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M0 1 L9 5 L0 9 z" className="arrowhead" />
          </marker>
        </defs>
        <g className="orbits" />
        <g className="links" />
        <g className="link-labels" />
      </svg>
      <div className="world-layer">
        {ids.map((id) => {
          const obj = snap.objects[id] ?? state.history.snapshots[state.history.revisions.at(-1)!.id]!.objects[id]
          if (!obj) return null
          const f = runtime.frames.get(id)
          return (
            <ObjectNode
              key={id}
              obj={obj}
              placement={runtime.placement(id)}
              lod={f?.lod ?? 0}
              parts={childrenOf(snap, id)}
              isCenter={id === center}
              selected={state.selection === id}
              centerActions={id === center ? centerActions : undefined}
              home={home}
              hovered={state.hover === id}
              overrides={state.overrides}
              homeResidue={homeResidue}
              past={past}
              pastSummary={state.history.revisions.find((r) => r.id === state.revision)?.summary}
              register={(el) => runtime.registerNode(id, el)}
              onInvoke={(action) => services.invoke(id, action)}
              onArtifacts={
                obj.artifacts?.length
                  ? () => {
                      if (store.getState().narrative?.status === 'playing') store.dispatch({ type: 'pauseNarrative' })
                      for (const ref of obj.artifacts!) store.dispatch({ type: 'materializeArtifact', id: ref, source: id })
                    }
                  : undefined
              }
            />
          )
        })}
        {excerpts.map(({ open, descriptor }) => (
          <div key={open.id} className="excerpt-anchor" data-source={anchorOf(open, descriptor)} ref={(el) => runtime.registerExcerpt(open.id, el)}>
            <Excerpt store={store} runtime={runtime} service={services.artifacts} open={open} descriptor={descriptor} />
          </div>
        ))}
      </div>
    </div>
  )
}

function Excerpt(p: { store: Store; runtime: SceneRuntime; service: ArtifactService; open: OpenArtifact; descriptor: CognitiveArtifact }) {
  const st = useArtifact(p.service, p.open.id)
  return (
    <ArtifactExcerpt
      descriptor={p.descriptor}
      state={st}
      pinned={p.open.pinned}
      onExpand={() => {
        if (p.store.getState().narrative?.status === 'playing') p.store.dispatch({ type: 'pauseNarrative' })
        p.store.dispatch({ type: 'expandArtifact', id: p.open.id, snapshot: snapshotView(p.store, p.runtime) })
      }}
      onDismiss={() => p.store.dispatch({ type: 'dismissArtifact', id: p.open.id })}
      onPin={(pinned) => p.store.dispatch({ type: 'pinArtifact', id: p.open.id, pinned })}
    />
  )
}

export function snapshotView(store: Store, runtime: SceneRuntime): ViewSnapshot {
  const s = store.getState()
  return {
    camera: runtime.currentCamera(),
    focusStack: s.focusStack,
    revision: s.revision,
    surface: s.surface,
    beatOverrides: s.overrides,
    compare: s.compare,
  }
}
