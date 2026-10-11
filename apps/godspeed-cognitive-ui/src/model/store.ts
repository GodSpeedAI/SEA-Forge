import type { Action, BeatOverrides, Id, UiState, WorldHistory, WorldObject, WorldSnapshot } from './types'

// A tiny external store. Semantic state lives here; per-frame state (camera position, tweens)
// lives in the scene runtime so React never re-renders at 60fps.
//
// The reducer never talks to the port. Consequential intents are sent by the app's intent
// path (src/app/intents.ts), which records them here with `intentSent`/`intentSettled`; world
// changes only ever arrive as contract snapshots (`loadHistory`).

export const emptyOverrides = (): BeatOverrides => ({ reveal: [], dim: [], relationships: [], highlight: [], annotations: [] })

export function initialState(history: WorldHistory): UiState {
  return {
    theme: 'light',
    mode: 'world',
    history,
    revision: nowRevision(history),
    focusStack: [],
    surface: 'orbital',
    selection: null,
    hover: null,
    artifacts: [],
    expandedFrom: null,
    narrative: null,
    narratives: {},
    overrides: emptyOverrides(),
    judgment: null,
    drawer: null,
    proposal: null,
    intents: [],
    timeline: false,
    timeMarks: [],
    compare: null,
    executions: {},
    design: null,
    agent: 'available',
    connection: 'live',
    cameraRequest: 0,
    cameraRestore: null,
    awake: false,
  }
}

// ---------------------------------------------------------------------------
// Selectors

export const nowRevision = (h: WorldHistory): string => h.revisions[h.revisions.length - 1]!.id
export const isPast = (s: UiState): boolean => s.revision !== nowRevision(s.history)
export const snapshotOf = (s: UiState): WorldSnapshot => s.history.snapshots[s.revision]!
export const focusOf = (s: UiState): Id | null => s.focusStack[s.focusStack.length - 1] ?? null
export const objectOf = (s: UiState, id: Id): WorldObject | undefined => snapshotOf(s).objects[id]
export const expandedArtifact = (s: UiState) => s.artifacts.find((a) => a.phase === 'expanded') ?? null
/** Resolves 'live' to the newest cursor. */
export const cursorOf = (h: WorldHistory, c: string): string => (c === 'live' ? nowRevision(h) : c)

/** The snapshot the design surface shows: a published version or the local draft. */
export function designSnapshot(s: UiState): WorldSnapshot | null {
  const d = s.design
  if (!d) return null
  if (d.revision === 'draft') return d.draft ?? d.history.snapshots[nowRevision(d.history)]!
  return d.history.snapshots[d.revision] ?? d.history.snapshots[nowRevision(d.history)]!
}

/** Focusable ancestry of an object, outermost first (categories and the core are not focus levels). */
export function focusPath(snap: WorldSnapshot, id: Id): Id[] {
  const path: Id[] = []
  let cur: WorldObject | undefined = snap.objects[id]
  while (cur) {
    if (cur.kind !== 'core' && (cur.kind !== 'category' || cur.id === id)) path.unshift(cur.id)
    cur = cur.parent ? snap.objects[cur.parent] : undefined
  }
  return path
}

// ---------------------------------------------------------------------------
// Reducer (pure)

export function reduce(s: UiState, a: Action): UiState {
  const fly = { cameraRequest: s.cameraRequest + 1, cameraRestore: null }
  switch (a.type) {
    case 'focus': {
      const snap = snapshotOf(s)
      if (!snap.objects[a.id]) return s
      const focusStack = focusPath(snap, a.id)
      // Comparison stays open across focus changes (it is a representation of whatever is in focus).
      return { ...s, focusStack, surface: s.compare ? 'compare' : 'orbital', selection: null, judgment: null, ...fly }
    }
    case 'back': {
      if (expandedArtifact(s)) return reduce(s, { type: 'collapseArtifact' })
      if (s.drawer) return { ...s, drawer: null }
      if (s.proposal) return { ...s, proposal: null }
      if (s.judgment) return { ...s, judgment: null, surface: 'orbital', ...fly }
      if (s.compare) return reduce(s, { type: 'closeCompare' })
      if (s.mode !== 'world') return reduce(s, { type: 'setMode', mode: 'world' })
      if (s.narrative || s.surface !== 'orbital' || s.overrides.caption) {
        return { ...s, narrative: null, surface: 'orbital', overrides: emptyOverrides(), artifacts: pinnedOnly(s), ...fly }
      }
      if (s.timeline) return { ...s, timeline: false, timeMarks: [], revision: nowRevision(s.history) }
      if (s.focusStack.length === 0) return s
      return { ...s, focusStack: s.focusStack.slice(0, -1), selection: null, ...fly }
    }
    case 'home':
      return {
        ...s,
        mode: 'world',
        design: null,
        focusStack: [],
        surface: 'orbital',
        selection: null,
        narrative: null,
        overrides: emptyOverrides(),
        judgment: null,
        drawer: null,
        proposal: null,
        artifacts: pinnedOnly(s),
        expandedFrom: null,
        compare: null,
        timeline: false,
        timeMarks: [],
        revision: nowRevision(s.history),
        ...fly,
      }
    case 'select':
      return s.selection === a.id ? s : { ...s, selection: a.id }
    case 'hover':
      return s.hover === a.id ? s : { ...s, hover: a.id }
    case 'reveal':
      return { ...s, overrides: { ...s.overrides, reveal: union(s.overrides.reveal, a.ids) } }
    case 'connect':
      return { ...s, overrides: { ...s.overrides, relationships: union(s.overrides.relationships, a.relationships) } }
    case 'arrange':
      if (a.surface === 'compare' && !s.compare) return s
      if (a.surface !== 'compare' && s.compare) return { ...reduce(s, { type: 'closeCompare' }), surface: a.surface }
      return s.surface === a.surface ? s : { ...s, surface: a.surface, ...fly }
    case 'setTime':
      if (!s.history.snapshots[a.revision]) return s
      return { ...s, revision: a.revision, timeline: true, judgment: null }
    case 'returnToNow':
      return {
        ...s,
        revision: nowRevision(s.history),
        timeline: false,
        timeMarks: [],
        ...(s.compare?.source === 'world' ? { compare: null, surface: s.compare.returnTo.surface, ...fly } : {}),
      }
    case 'openTimeline':
      return a.open ? { ...s, timeline: true } : reduce(s, { type: 'returnToNow' })
    case 'markTime': {
      if (!s.history.snapshots[a.revision]) return s
      const has = s.timeMarks.includes(a.revision)
      const timeMarks = has ? s.timeMarks.filter((m) => m !== a.revision) : [...s.timeMarks, a.revision].slice(-2)
      return { ...s, timeMarks, timeline: true }
    }
    case 'openCompare': {
      const source = a.source ?? 'world'
      const h = source === 'design' ? s.design?.history : s.history
      if (!h) return s
      const ok = (c: string) => c === 'draft' ? source === 'design' && !!s.design?.draft : !!h.snapshots[cursorOf(h, c)]
      if (!ok(a.a) || !ok(a.b) || a.a === a.b) return s
      const returnTo = s.compare?.returnTo ?? { revision: s.revision, surface: s.surface === 'compare' ? 'orbital' : s.surface }
      return {
        ...s,
        compare: { source, a: a.a === 'draft' ? a.a : cursorOf(h, a.a), b: a.b === 'draft' ? a.b : cursorOf(h, a.b), returnTo },
        surface: 'compare',
        judgment: null,
        // The world shown behind the comparison is side B (the later state).
        revision: source === 'world' ? cursorOf(h, a.b) : s.revision,
        ...fly,
      }
    }
    case 'closeCompare': {
      if (!s.compare) return s
      const { returnTo, source } = s.compare
      return {
        ...s,
        compare: null,
        surface: returnTo.surface,
        revision: source === 'world' && s.history.snapshots[returnTo.revision] ? returnTo.revision : s.revision,
        ...fly,
      }
    }
    case 'materializeArtifact':
      return s.artifacts.some((x) => x.id === a.id)
        ? s
        : { ...s, artifacts: [...s.artifacts, { id: a.id, phase: 'excerpt', pinned: false, ...(a.source ? { source: a.source } : {}) }] }
    case 'expandArtifact': {
      const base = s.artifacts.some((x) => x.id === a.id)
        ? s.artifacts
        : [...s.artifacts, { id: a.id, phase: 'excerpt' as const, pinned: false }]
      return {
        ...s,
        artifacts: base.map((x) => ({ ...x, phase: x.id === a.id ? 'expanded' : 'excerpt' })),
        // Keep the original snapshot when switching from one expanded artifact to another.
        expandedFrom: s.expandedFrom ?? a.snapshot,
      }
    }
    case 'collapseArtifact': {
      const snap = s.expandedFrom
      const artifacts = s.artifacts.map((x) => ({ ...x, phase: 'excerpt' as const }))
      if (!snap) return { ...s, artifacts }
      return {
        ...s,
        artifacts,
        expandedFrom: null,
        focusStack: snap.focusStack,
        revision: s.history.snapshots[snap.revision] ? snap.revision : s.revision,
        surface: snap.surface,
        overrides: snap.beatOverrides,
        compare: snap.compare,
        cameraRequest: s.cameraRequest + 1,
        cameraRestore: snap.camera,
      }
    }
    case 'dismissArtifact': {
      const was = s.artifacts.find((x) => x.id === a.id)
      const base = was?.phase === 'expanded' ? reduce(s, { type: 'collapseArtifact' }) : s
      return { ...base, artifacts: base.artifacts.filter((x) => x.id !== a.id) }
    }
    case 'pinArtifact': {
      // Pinning is a local keep-available choice (no contract operation persists pins yet).
      if (!s.artifacts.some((x) => x.id === a.id)) return s
      return { ...s, artifacts: s.artifacts.map((x) => (x.id === a.id ? { ...x, pinned: a.pinned } : x)) }
    }
    case 'invoke': {
      if (isPast(s) || !a.action.consequential) return s // the past is read-only
      // Discretionary work is a typed add-work surface (title + justification), not a choice.
      if (a.action.intent === 'ADD_DISCRETIONARY_WORK') {
        return { ...s, drawer: { object: a.object, action: a.action }, judgment: null, ...fly }
      }
      return { ...s, judgment: { object: a.object, action: a.action }, surface: 'judgment', compare: null, ...fly }
    }
    case 'openDrawer':
      return { ...s, drawer: { object: a.object, action: a.action }, judgment: null, ...fly }
    case 'closeDrawer':
      return s.drawer ? { ...s, drawer: null } : s
    case 'openProposals':
      return {
        ...s,
        proposal: { status: 'loading', templates: [], selected: null, params: {}, preflighting: false, preflight: null, submitting: false },
        judgment: null,
        drawer: null,
      }
    case 'proposalsLoaded':
      return s.proposal ? { ...s, proposal: { ...s.proposal, status: 'ready', templates: a.templates, error: undefined } } : s
    case 'proposalsUnavailable':
      return s.proposal ? { ...s, proposal: { ...s.proposal, status: 'unavailable', error: a.error } } : s
    case 'selectTemplate':
      // A new template resets parameters and any previous preflight: the digest binds the params.
      return s.proposal
        ? { ...s, proposal: { ...s.proposal, selected: a.templateRef, params: {}, preflight: null, submitError: undefined, submitCode: undefined } }
        : s
    case 'setProposalParam': {
      if (!s.proposal) return s
      // Changed parameters invalidate the preflight digest; preflight must run again.
      return { ...s, proposal: { ...s.proposal, params: { ...s.proposal.params, [a.name]: a.value }, preflight: null, submitError: undefined } }
    }
    case 'preflightStarted':
      return s.proposal ? { ...s, proposal: { ...s.proposal, preflighting: true } } : s
    case 'preflightResult':
      return s.proposal ? { ...s, proposal: { ...s.proposal, preflighting: false, preflight: a.result } } : s
    case 'proposalSubmitStarted':
      return s.proposal ? { ...s, proposal: { ...s.proposal, submitting: true, submitError: undefined, submitCode: undefined } } : s
    case 'proposalSubmitted':
      return s.proposal ? { ...s, proposal: { ...s.proposal, submitting: false, result: { caseId: a.caseId } } } : s
    case 'proposalError':
      return s.proposal ? { ...s, proposal: { ...s.proposal, submitting: false, submitError: a.error, submitCode: a.code } } : s
    case 'closeProposals':
      return s.proposal ? { ...s, proposal: null } : s
    case 'connectionState':
      return s.connection === a.connection ? s : { ...s, connection: a.connection }
    case 'intentSent': {
      const intents = [...s.intents, a.record]
      const judgment = a.judgment && s.judgment && !s.judgment.pending ? { ...s.judgment, pending: a.record.id, outcome: undefined } : s.judgment
      return { ...s, intents, judgment }
    }
    case 'closeJudgment':
      return s.judgment ? { ...s, judgment: null, surface: 'orbital', ...fly } : s
    case 'intentSettled': {
      const rec = s.intents.find((r) => r.id === a.id)
      if (!rec) return s
      const intents = s.intents.map((r) => (r.id === a.id ? { ...r, state: a.state, note: a.note, code: a.code } : r))
      const judgment =
        s.judgment?.pending === a.id
          ? { ...s.judgment, pending: undefined, outcome: { state: a.state, note: a.note, code: a.code, by: rec.actor } }
          : s.judgment
      return { ...s, intents, judgment }
    }
    case 'execution':
      return { ...s, executions: { ...s.executions, [a.exec.object]: a.exec } }
    case 'narrativeStarted':
      return {
        ...s,
        narratives: { ...s.narratives, [a.narrative.id]: a.narrative },
        narrative: { id: a.narrative.id, index: -1, status: 'playing' },
        overrides: emptyOverrides(),
        artifacts: pinnedOnly(s),
        expandedFrom: null,
      }
    case 'narrativeBeat': {
      const n = s.narratives[a.id]
      if (!n) return s
      return { ...s, narratives: { ...s.narratives, [a.id]: { ...n, beats: [...n.beats, a.beat], complete: a.complete } } }
    }
    case 'narrativeComplete': {
      const n = s.narratives[a.id]
      return n && !n.complete ? { ...s, narratives: { ...s.narratives, [a.id]: { ...n, complete: true } } } : s
    }
    case 'narrativeFailed': {
      const n = s.narratives[a.id]
      if (!n) return s
      const narratives = { ...s.narratives, [a.id]: { ...n, complete: true } }
      const narrative = s.narrative?.id === a.id ? { ...s.narrative, error: a.error } : s.narrative
      return { ...s, narratives, narrative }
    }
    case 'applyBeat': {
      const b = a.beat
      const snapNow = snapshotOf(s)
      const known = (ids?: Id[]) => (ids ?? []).filter((id) => snapNow.objects[id])
      let next: UiState = {
        ...s,
        narrative: s.narrative ? { ...s.narrative, index: a.index } : s.narrative,
        overrides: {
          reveal: union(s.overrides.reveal, known(b.reveal)),
          dim: b.dim ? known(b.dim) : s.overrides.dim,
          relationships: union(s.overrides.relationships, b.relationships ?? []),
          caption: b.caption,
          subcaption: b.subcaption ?? s.overrides.subcaption,
          residue: b.residue ?? s.overrides.residue,
          highlight: b.highlight ? known(b.highlight) : s.overrides.highlight,
          annotations: b.annotate ? b.annotate.filter((x) => snapNow.objects[x.target]) : s.overrides.annotations,
        },
      }
      if (b.compare) return { ...reduce(next, { type: 'openCompare', a: b.compare.a, b: b.compare.b }) }
      if (next.compare && (b.surface || b.time)) next = reduce(next, { type: 'closeCompare' })
      const bt = b.time ? cursorOf(next.history, b.time) : null
      if (bt && next.history.snapshots[bt]) next = { ...next, revision: bt, timeline: bt !== nowRevision(next.history) }
      let moved = false
      if (b.focus && snapshotOf(next).objects[b.focus]) {
        const path = focusPath(snapshotOf(next), b.focus)
        if (path.join('/') !== next.focusStack.join('/')) {
          next = { ...next, focusStack: path }
          moved = true
        }
      }
      if (b.surface && b.surface !== next.surface) {
        next = { ...next, surface: b.surface }
        moved = true
      }
      for (const id of b.artifacts ?? []) next = reduce(next, { type: 'materializeArtifact', id })
      return moved ? { ...next, cameraRequest: next.cameraRequest + 1, cameraRestore: null } : next
    }
    case 'pauseNarrative':
      return s.narrative?.status === 'playing' ? { ...s, narrative: { ...s.narrative, status: 'paused' } } : s
    case 'resumeNarrative':
      return s.narrative?.status === 'paused' ? { ...s, narrative: { ...s.narrative, status: 'playing' } } : s
    case 'endNarrative':
      return s.narrative ? { ...s, narrative: { ...s.narrative, status: 'done' } } : s
    case 'agentAvailability': {
      const agent = a.available ? 'available' : 'unavailable'
      return s.agent === agent ? s : { ...s, agent }
    }
    case 'setTheme':
      return { ...s, theme: a.theme }
    case 'setMode': {
      if (a.mode === s.mode) return s
      if (s.mode === 'case-design' && a.mode !== 'case-design') {
        // Leaving design returns to where it was entered; the local draft is discarded with it.
        return { ...s, mode: a.mode, focusStack: s.design?.returnFocus ?? s.focusStack, design: null, compare: s.compare?.source === 'design' ? null : s.compare, surface: 'orbital', ...fly }
      }
      if (a.mode === 'case-design' && !s.design) return s
      return { ...s, mode: a.mode, ...fly }
    }
    case 'enterDesign':
      return { ...s, mode: 'case-design', design: a.design, compare: null, judgment: null, surface: 'orbital', ...fly }
    case 'designSelect':
      return s.design ? { ...s, design: { ...s.design, selected: a.id } } : s
    case 'designShow': {
      const d = s.design
      if (!d || (a.revision !== 'draft' && !d.history.snapshots[a.revision]) || (a.revision === 'draft' && !d.draft)) return s
      return { ...s, design: { ...d, revision: a.revision } }
    }
    case 'designDraft':
      return s.design ? { ...s, design: { ...s.design, draft: a.draft, revision: a.draft ? 'draft' : nowRevision(s.design.history) } } : s
    case 'loadHistory': {
      // Standing at "now" follows new revisions as they arrive; standing in the past stays put.
      const wasNow = s.revision === nowRevision(s.history)
      const keep = !wasNow && a.history.snapshots[s.revision] ? s.revision : nowRevision(a.history)
      return { ...s, history: a.history, revision: keep }
    }
    case 'wake':
      return s.awake === a.awake ? s : { ...s, awake: a.awake }
  }
}

function pinnedOnly(s: UiState) {
  return s.artifacts.filter((x) => x.pinned).map((x) => ({ ...x, phase: 'excerpt' as const }))
}

function union(a: Id[], b: Id[]): Id[] {
  if (!b.length) return a
  const set = new Set(a)
  for (const x of b) set.add(x)
  return [...set]
}

// ---------------------------------------------------------------------------
// Store

export type Listener = (s: UiState, prev: UiState, a: Action) => void

export interface Store {
  getState(): UiState
  dispatch(a: Action): void
  subscribe(fn: () => void): () => void
  /** Effects hook: called after every dispatch with the action (used by the narrative player and adapters). */
  listen(fn: Listener): () => void
}

export function createStore(initial: UiState): Store {
  let state = initial
  const subs = new Set<() => void>()
  const listeners = new Set<Listener>()
  return {
    getState: () => state,
    dispatch(a) {
      const prev = state
      state = reduce(state, a)
      if (state !== prev) for (const fn of subs) fn()
      for (const fn of listeners) fn(state, prev, a)
    },
    subscribe(fn) {
      subs.add(fn)
      return () => subs.delete(fn)
    },
    listen(fn) {
      listeners.add(fn)
      return () => listeners.delete(fn)
    },
  }
}
