/**
 * The T03 host surface (T03).
 *
 * A deliberately small renderer: it observes the core store and calls the environment's action
 * vocabulary — the SAME vocabulary the tests call. It owns no state of its own beyond React's
 * subscription, and it never interprets the world; it draws it. The banner states plainly what this
 * is and what it is not.
 */
import { useEffect, useSyncExternalStore } from 'react'

import type { CognitiveEnvironment } from '../core/engine'
import type { UiState } from '../core/model'
import { surfaceObjects } from '../core/actions'

const FIXTURE_BANNER =
  'Fixture-backed prototype — NOT connected to SEA Forge, GitHub, Gauntlet, or any agent. ' +
  'No case truth, settlement, or authority lives here.'

export function App({ environment }: { environment: CognitiveEnvironment }) {
  const state = useSyncExternalStore(
    (listener) => environment.store.subscribe(listener),
    () => environment.store.get(),
  )

  const world = state.world
  if (!world) {
    return (
      <main className="frame">
        <Banner />
        <p className="empty">loading the fixture world…</p>
      </main>
    )
  }
  return (
    <main className="frame">
      <Banner />
      <KeysLegend />
      <KeyboardController environment={environment} />
      <section className="surfaces" aria-label="surfaces">
        {world.surfaces.map((surface) => (
          <SurfaceColumn
            key={surface.id}
            environment={environment}
            state={state}
            surfaceId={surface.id}
            label={surface.label}
          />
        ))}
      </section>
      <aside className="panels" aria-label="context panels">
        <RelationshipsPanel environment={environment} state={state} />
        <TimePanel environment={environment} state={state} />
        <NarrationPanel environment={environment} state={state} />
        <RefusalPanel state={state} />
      </aside>
    </main>
  )
}

function Banner() {
  return (
    <header className="banner" role="note">
      <strong>What this is:</strong> a fixture-backed interaction prototype (plan T03).{' '}
      <strong>What it is not:</strong> connected to SEA Forge, GitHub, Gauntlet, CopilotKit, or any
      governed authority. {FIXTURE_BANNER}
    </header>
  )
}

function KeysLegend() {
  return (
    <p className="keys">
      <kbd>←</kbd>/<kbd>→</kbd> switch surface · <kbd>↑</kbd>/<kbd>↓</kbd> move focus ·{' '}
      <kbd>Enter</kbd> select · <kbd>e</kbd> disclose deeper · <kbd>t</kbd> step back in time ·{' '}
      <kbd>n</kbd> return to now · <kbd>x</kbd> propose consequence (refused) · <kbd>Esc</kbd> clear
    </p>
  )
}

/**
 * Physical-input adapter: translates keyboard events into the SAME action vocabulary the tests use.
 * This is the only place keyboard semantics live; the grammar itself is in the core.
 */
function KeyboardController({ environment }: { environment: CognitiveEnvironment }) {
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const state = environment.store.get()
      const focusId = state.focusId
      switch (event.key) {
        case 'ArrowLeft': {
          const target = previousSurface(state)
          if (target) environment.setSurface(target)
          break
        }
        case 'ArrowRight': {
          const target = nextSurface(state)
          if (target) environment.setSurface(target)
          break
        }
        case 'ArrowUp':
          environment.moveFocus('previous')
          break
        case 'ArrowDown':
          environment.moveFocus('next')
          break
        case 'Enter':
          environment.select(focusId)
          break
        case 'e':
          if (focusId) void environment.resolve(focusId)
          break
        case 't':
          environment.stepTime(-1)
          break
        case 'n':
          environment.returnToNow()
          break
        case 'x':
          if (focusId) void environment.propose('propose-consequence', focusId)
          break
        case 'Escape':
          environment.clearSelection()
          break
        default:
          return
      }
      event.preventDefault()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [environment])
  return null
}

function nextSurface(state: UiState): string | null {
  const surfaces = state.world?.surfaces ?? []
  if (surfaces.length === 0) return null
  const i = state.surfaceId ? surfaces.findIndex((s) => s.id === state.surfaceId) : -1
  return surfaces[Math.min(surfaces.length - 1, i + 1)]?.id ?? null
}

function previousSurface(state: UiState): string | null {
  const surfaces = state.world?.surfaces ?? []
  if (surfaces.length === 0) return null
  const i = state.surfaceId ? surfaces.findIndex((s) => s.id === state.surfaceId) : surfaces.length
  return surfaces[Math.max(0, i - 1)]?.id ?? null
}

function SurfaceColumn({
  environment,
  state,
  surfaceId,
  label,
}: {
  environment: CognitiveEnvironment
  state: UiState
  surfaceId: string
  label: string
}) {
  const objects = state.world?.objects ?? []
  const ids = surfaceObjects(state.world, surfaceId)
  return (
    <div className={'surface' + (state.surfaceId === surfaceId ? ' current' : '')}>
      <button
        className="surface-name"
        onClick={() => environment.setSurface(surfaceId)}
        title="make this the current surface"
      >
        {label}
      </button>
      {ids.map((id) => {
        const object = objects.find((o) => o.id === id)
        if (!object) return null
        const disclosure = state.disclosures[id]
        const descriptors = state.artifactCatalog[id] ?? []
        return (
          <div
            key={id}
            className={
              'object' +
              (state.focusId === id ? ' focused' : '') +
              (state.selectedId === id ? ' selected' : '')
            }
            tabIndex={0}
            onClick={() => environment.focus(id)}
            onDoubleClick={() => environment.select(id)}
          >
            <div className="object-head">
              <span className="kind">{object.kind}</span>
              <span className="label">{object.label}</span>
            </div>
            <div className="salience" title={`salience ${object.salience}`}>
              <div className="salience-bar" style={{ width: `${Math.round(object.salience * 100)}%` }} />
            </div>
            {descriptors.length > 0 && (
              <div className="disclosure">
                {disclosure ? (
                  <>
                    <span className="level">{disclosure.pending ? '…' : disclosure.level}</span>
                    {!disclosure.pending && disclosure.note && <span className="note">{disclosure.note}</span>}
                  </>
                ) : (
                  <span className="level muted">minimal</span>
                )}
              </div>
            )}
          </div>
        )
      })}
    </div>
  )
}

function RelationshipsPanel({
  environment,
  state,
}: {
  environment: CognitiveEnvironment
  state: UiState
}) {
  const focusId = state.focusId
  const world = state.world
  const related =
    focusId && world
      ? world.relationships.filter((r) => r.from === focusId || r.to === focusId)
      : []
  const labelOf = (id: string) => world?.objects.find((o) => o.id === id)?.label ?? id
  return (
    <div className="panel">
      <h2>Relationships</h2>
      {!focusId && <p className="muted">focus an object to see its relationships</p>}
      {focusId && related.length === 0 && <p className="muted">no relationships for the focused object</p>}
      {related.map((r, i) => (
        <p key={i} className="relationship">
          <button className="link" onClick={() => environment.focus(r.from)}>{labelOf(r.from)}</button>
          <span className="kind"> {r.kind} </span>
          <button className="link" onClick={() => environment.focus(r.to)}>{labelOf(r.to)}</button>
        </p>
      ))}
    </div>
  )
}

function TimePanel({ environment, state }: { environment: CognitiveEnvironment; state: UiState }) {
  const { positions, index, live, truncated } = state.time
  return (
    <div className="panel">
      <h2>Time {live ? '· now (live)' : `· position ${index! + 1}/${positions.length}`}</h2>
      {truncated && <p className="muted">older history exists beyond this window</p>}
      {positions.map((p, i) => (
        <p key={p.cursor}>
          <button
            className={'link' + (index === i && !live ? ' active' : '')}
            onClick={() => environment.setTimePosition(i)}
          >
            {p.at} — {p.summary}
          </button>
        </p>
      ))}
      <button className="action" onClick={() => environment.returnToNow()} disabled={live}>
        return to now
      </button>
    </div>
  )
}

function NarrationPanel({ environment, state }: { environment: CognitiveEnvironment; state: UiState }) {
  const narration = state.narration
  return (
    <div className="panel">
      <h2>Narration</h2>
      {!narration.available && (
        <p className="muted">
          no agent adapter is configured; narration is unavailable, not failed (fixture demo)
        </p>
      )}
      {narration.beats.map((b) => (
        <p key={b.index} className="beat">
          {b.text}
          {b.evidenceRefs.length > 0 && <span className="kind"> [{b.evidenceRefs.join(', ')}]</span>}
        </p>
      ))}
      {narration.interrupted && <p className="muted">narration was interrupted</p>}
      {narration.available && (
        <div className="row">
          <button className="action" onClick={() => void environment.requestExplanation('explain the focus')}>
            ask
          </button>
          <button className="action" onClick={() => environment.interruptNarration()}>
            interrupt
          </button>
        </div>
      )}
    </div>
  )
}

function RefusalPanel({ state }: { state: UiState }) {
  const refusal = state.lastRefusal
  return (
    <div className="panel">
      <h2>Consequential requests</h2>
      {!refusal && (
        <p className="muted">
          nothing consequential has been requested. pressing <kbd>x</kbd> proposes a consequence; a
          fixture adapter refuses it — nothing is concluded locally.
        </p>
      )}
      {refusal && (
        <p className="refusal">
          refused: <span className="kind">{refusal.reason}</span> — the environment never concludes
          consequential actions locally.
        </p>
      )}
    </div>
  )
}
