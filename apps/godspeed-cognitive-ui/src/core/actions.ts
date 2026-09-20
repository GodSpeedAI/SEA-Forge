/**
 * Application actions (T03).
 *
 * Every state transition the environment can perform is an action here, phrased in semantic
 * vocabulary (an object id, a direction, a position index) — never in physical input or renderer
 * terms. Pointer, keyboard, agent and future voice adapters call the same functions; tests call them
 * directly. This is what makes the interaction grammar survive renderer and transport replacement
 * (REQ-ARCH-004).
 *
 * Actions are pure state transitions plus adapter calls; they hold no state of their own.
 */
import type {
  ArtifactAdapter,
  InteractionAdapter,
  InteractionIntent,
  IntentKind,
  ObjectId,
  WorldSnapshot,
} from '../../contracts/index'
import type { ArtifactDisclosure, CognitiveArtifactDescriptor, UiState } from './model'
import { initialState } from './model'
import type { CognitiveStore } from './store'

const DISCLOSURE_CYCLE: Record<ArtifactDisclosure['level'], ArtifactDisclosure['level'] | null> = {
  minimal: 'summary',
  summary: 'source',
  source: null,
}

/** Objects of the current surface, in the surface's own order. */
export function surfaceObjects(world: WorldSnapshot | null, surfaceId: string | null): readonly ObjectId[] {
  if (!world || !surfaceId) return []
  const surface = world.surfaces.find((s) => s.id === surfaceId)
  return surface ? surface.objectIds : []
}

export function objectById(world: WorldSnapshot | null, id: ObjectId | null) {
  if (!world || !id) return undefined
  return world.objects.find((o) => o.id === id)
}

function replaceDisclosure(
  state: UiState,
  objectId: ObjectId,
  disclosure: ArtifactDisclosure,
): UiState {
  return { ...state, disclosures: { ...state.disclosures, [objectId]: disclosure } }
}

export function focus(store: CognitiveStore, id: ObjectId | null): void {
  store.update((state) => {
    if (id !== null && !objectById(state.world, id)) return state
    return { ...state, focusId: id }
  })
}

/**
 * Move focus within the current surface's ordered objects. Unknown positions leave state unchanged,
 * so a renderer can offer the action unconditionally.
 */
export function moveFocus(store: CognitiveStore, direction: 'next' | 'previous'): void {
  store.update((state) => {
    const ids = surfaceObjects(state.world, state.surfaceId)
    if (ids.length === 0) return state
    const current = state.focusId ? ids.indexOf(state.focusId) : -1
    const nextIndex =
      current === -1
        ? direction === 'next'
          ? 0
          : ids.length - 1
        : Math.min(ids.length - 1, Math.max(0, current + (direction === 'next' ? 1 : -1)))
    const nextId = ids[nextIndex]
    return nextId && nextId !== state.focusId ? { ...state, focusId: nextId } : state
  })
}

/** Selecting another surface moves the viewpoint; focus and selection are per-surface, so they clear. */
export function setSurface(store: CognitiveStore, surfaceId: string): void {
  store.update((state) => {
    if (!state.world?.surfaces.some((s) => s.id === surfaceId)) return state
    if (state.surfaceId === surfaceId) return state
    return { ...state, surfaceId, focusId: null, selectedId: null }
  })
}

export function select(store: CognitiveStore, id: ObjectId | null): void {
  store.update((state) => {
    if (id !== null && !objectById(state.world, id)) return state
    return { ...state, selectedId: id, focusId: id ?? state.focusId }
  })
}

/**
 * Cycle the focused object's disclosure one level deeper (minimal → summary → source). The content at
 * each level comes from the artifact adapter; the core only tracks the level. Returns the disclosure
 * after the transition so callers can observe what the interaction reached.
 */
export async function resolve(
  store: CognitiveStore,
  objectId: ObjectId,
  artifact: ArtifactAdapter,
  catalog: Readonly<Record<ObjectId, readonly CognitiveArtifactDescriptor[]>>,
): Promise<ArtifactDisclosure | null> {
  const state = store.get()
  const descriptor = catalog[objectId]?.[0]
  if (!descriptor) return null
  const current = state.disclosures[objectId]?.level ?? 'minimal'
  const next = DISCLOSURE_CYCLE[current]
  if (!next) return state.disclosures[objectId] ?? null
  store.update((s) => replaceDisclosure(s, objectId, { objectId, level: next, pending: true }))
  const resolved = await artifact.resolve(descriptor.ref, next)
  const disclosure: ArtifactDisclosure = {
    objectId,
    level: next,
    title: descriptor.title,
    note: `${resolved.kind} · ${resolved.mediaType} · ${resolved.byteLength} B`,
    pending: false,
  }
  store.update((s) => replaceDisclosure(s, objectId, disclosure))
  return disclosure
}

/**
 * Consequential intent: dispatched through the interaction adapter and never concluded locally. The
 * outcome (including a refusal) becomes part of the state so any renderer shows the same record.
 */
export async function propose(
  store: CognitiveStore,
  intent: { kind: IntentKind; target?: string; parameters?: Readonly<Record<string, string>> },
  dispatch: InteractionAdapter['dispatch'],
  now: () => number,
): Promise<void> {
  const full: InteractionIntent = { id: `intent-${now()}`, ...intent }
  const outcome = await dispatch(full)
  store.update((state) => {
    if (outcome.status === 'accepted') return { ...state, lastRefusal: null }
    return { ...state, lastRefusal: { reason: outcome.reason, note: undefined, at: now() } }
  })
}

export function stepTime(store: CognitiveStore, delta: 1 | -1): void {
  store.update((state) => {
    const { positions, index } = state.time
    if (positions.length === 0) return state
    const base = index ?? positions.length - 1
    const next = Math.min(positions.length - 1, Math.max(0, base + delta))
    if (next === base) return state
    return { ...state, time: { ...state.time, index: next, live: false } }
  })
}

export function setTimePosition(store: CognitiveStore, index: number): void {
  store.update((state) => {
    if (index < 0 || index >= state.time.positions.length) return state
    return { ...state, time: { ...state.time, index, live: false } }
  })
}

/** Return to the present: the projection is live again and the historical position is released. */
export function returnToNow(store: CognitiveStore): void {
  store.update((state) => ({ ...state, time: { ...state.time, index: null, live: true } }))
}

export function clearSelection(store: CognitiveStore): void {
  store.update((state) => ({ ...state, selectedId: null, lastRefusal: null }))
}

export function resetNarration(store: CognitiveStore, available: boolean): void {
  store.update((state) => ({
    ...state,
    narration: { available, beats: [], interrupted: false },
  }))
}

export function appendBeat(
  store: CognitiveStore,
  beat: { index: number; text: string; evidenceRefs: readonly string[] },
): void {
  store.update((state) => ({
    ...state,
    narration: { ...state.narration, beats: [...state.narration.beats, beat] },
  }))
}

export function markInterrupted(store: CognitiveStore): void {
  store.update((state) => ({
    ...state,
    narration: { ...state.narration, interrupted: true, beats: [...state.narration.beats] },
  }))
}

/** The empty state, exposed for tests that build an environment from scratch. */
export function emptyState(): UiState {
  return initialState
}
