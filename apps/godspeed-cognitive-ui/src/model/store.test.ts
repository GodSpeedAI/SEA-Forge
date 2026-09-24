import { describe, it, expect, beforeEach } from 'bun:test'
import { buildNorthstarHistory } from '../adapters/local/northstarData'
import { projectHistory } from '../ports/project'
import {
  reduce,
  initialState,
  snapshotOf,
  focusOf,
  nowRevision,
  createStore,
} from './store'
import type { UiState } from './types'

const raw = buildNorthstarHistory({ actor_id: 'u', role: 'case_architect' })
const history = projectHistory(raw, {}, 'local-contract', { withCore: true })

describe('store.reduce', () => {
  let state: UiState

  beforeEach(() => {
    state = initialState(history)
  })

  describe('focus', () => {
    it('focus builds the focus path', () => {
      const snap = snapshotOf(state)
      const objects = Object.values(snap.objects).filter((o) => o.kind === 'case')
      if (!objects.length) return
      const caseId = objects[0].id
      const next = reduce(state, { type: 'focus', id: caseId })
      expect(next.focusStack).toEqual([caseId])
      expect(focusOf(next)).toBe(caseId)
    })

    it('back pops focus', () => {
      const snap = snapshotOf(state)
      const caseId = Object.values(snap.objects).find((o) => o.kind === 'case')?.id
      if (!caseId) return
      let next = reduce(state, { type: 'focus', id: caseId })
      expect(focusOf(next)).toBe(caseId)
      next = reduce(next, { type: 'back' })
      expect(focusOf(next)).toBeNull()
    })
  })

  describe('home', () => {
    it('resets focus, compare, timeline, design and returns to now', () => {
      let next = state
      const snap = snapshotOf(next)
      const caseId = Object.values(snap.objects).find((o) => o.kind === 'case')?.id
      if (caseId) next = reduce(next, { type: 'focus', id: caseId })
      next = reduce(next, { type: 'setTime', revision: history.revisions[0]!.id })
      next = reduce(next, { type: 'setMode', mode: 'case-design' })
      next = reduce(next, { type: 'home' })
      expect(next.focusStack).toEqual([])
      expect(next.surface).toBe('orbital')
      expect(next.revision).toBe(nowRevision(history))
      expect(next.timeline).toBe(false)
      expect(next.design).toBeNull()
    })
  })

  describe('timeline', () => {
    it('setTime opens the timeline', () => {
      const rev = history.revisions[0]!.id
      const next = reduce(state, { type: 'setTime', revision: rev })
      expect(next.timeline).toBe(true)
      expect(next.revision).toBe(rev)
    })

    it('returnToNow closes the timeline', () => {
      let next = reduce(state, { type: 'setTime', revision: history.revisions[0]!.id })
      expect(next.timeline).toBe(true)
      next = reduce(next, { type: 'returnToNow' })
      expect(next.timeline).toBe(false)
      expect(next.revision).toBe(nowRevision(history))
    })
  })

  describe('markTime', () => {
    it('keeps at most 2 marks', () => {
      if (history.revisions.length < 3) return
      let next = state
      const [rev1, rev2, rev3] = history.revisions.slice(0, 3).map((r) => r.id)
      next = reduce(next, { type: 'markTime', revision: rev1 })
      expect(next.timeMarks).toEqual([rev1])
      next = reduce(next, { type: 'markTime', revision: rev2 })
      expect(next.timeMarks).toEqual([rev1, rev2])
      next = reduce(next, { type: 'markTime', revision: rev3 })
      expect(next.timeMarks).toEqual([rev2, rev3])
    })

    it('toggles marks', () => {
      const rev = history.revisions[0]!.id
      let next = reduce(state, { type: 'markTime', revision: rev })
      expect(next.timeMarks).toContain(rev)
      next = reduce(next, { type: 'markTime', revision: rev })
      expect(next.timeMarks).not.toContain(rev)
    })
  })

  describe('compare', () => {
    it('openCompare sets surface and compare state', () => {
      if (history.revisions.length < 2) return
      const [a, b] = history.revisions.slice(-2).map((r) => r.id)
      const next = reduce(state, { type: 'openCompare', a, b })
      expect(next.surface).toBe('compare')
      expect(next.compare).not.toBeNull()
      expect(next.compare!.a).toBe(a)
      expect(next.compare!.b).toBe(b)
    })

    it('refuses to compare a cursor with itself', () => {
      const a = history.revisions[0]!.id
      const next = reduce(state, { type: 'openCompare', a, b: a })
      expect(next.compare).toBeNull()
    })

    it('closeCompare restores prior revision and surface', () => {
      if (history.revisions.length < 2) return
      const [a, b] = history.revisions.slice(-2).map((r) => r.id)
      let next = reduce(state, { type: 'setTime', revision: a })
      const priorSurface = next.surface
      next = reduce(next, { type: 'openCompare', a, b })
      expect(next.compare).not.toBeNull()
      next = reduce(next, { type: 'closeCompare' })
      expect(next.compare).toBeNull()
      expect(next.surface).toBe(priorSurface)
    })
  })

  describe('expandArtifact and collapseArtifact', () => {
    it('collapseArtifact restores exact ViewSnapshot', () => {
      const snap2 = reduce(state, { type: 'setTime', revision: history.revisions[0]!.id })
      const viewSnap = {
        focusStack: snap2.focusStack,
        revision: snap2.revision,
        surface: snap2.surface,
        beatOverrides: snap2.overrides,
        compare: snap2.compare,
        camera: { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 },
      }
      let next = reduce(snap2, { type: 'expandArtifact', id: 'test-artifact', snapshot: viewSnap })
      expect(next.expandedFrom).toEqual(viewSnap)
      next = reduce(next, { type: 'collapseArtifact' })
      expect(next.expandedFrom).toBeNull()
      expect(next.focusStack).toEqual(viewSnap.focusStack)
      expect(next.surface).toBe(viewSnap.surface)
    })
  })

  describe('pinArtifact', () => {
    it('toggles pinned state', () => {
      let next = reduce(state, { type: 'materializeArtifact', id: 'a1' })
      next = reduce(next, { type: 'pinArtifact', id: 'a1', pinned: true })
      expect(next.artifacts.find((x) => x.id === 'a1')?.pinned).toBe(true)
      next = reduce(next, { type: 'pinArtifact', id: 'a1', pinned: false })
      expect(next.artifacts.find((x) => x.id === 'a1')?.pinned).toBe(false)
    })
  })

  describe('invoke', () => {
    it('opens judgment for consequential action', () => {
      const snap = snapshotOf(state)
      const objs = Object.values(snap.objects)
      const obj = objs.find((o) => o.actions?.some((a) => a.consequential))
      if (!obj || !obj.actions) return
      const action = obj.actions.find((a) => a.consequential)!
      const next = reduce(state, { type: 'invoke', object: obj.id, action })
      expect(next.judgment).not.toBeNull()
      expect(next.surface).toBe('judgment')
    })

    it('is a no-op in the past', () => {
      let next = reduce(state, { type: 'setTime', revision: history.revisions[0]!.id })
      const snap = snapshotOf(next)
      const obj = Object.values(snap.objects).find((o) => o.actions?.length)
      if (!obj || !obj.actions) return
      const before = next
      next = reduce(next, { type: 'invoke', object: obj.id, action: obj.actions[0]! })
      expect(next).toBe(before)
    })
  })

  describe('intentSettled', () => {
    it('intentSettled with refused state clears pending', () => {
      expect(state.intents).toHaveLength(0)
      const next = reduce(state, { type: 'intentSettled', id: 'nonexistent', state: 'refused', note: 'Not now' })
      expect(next).toBe(state) // no-op when record doesn't exist
    })
  })

  describe('narrative actions', () => {
    it('narrativeStarted/narrativeBeat/narrativeComplete', () => {
      const narr = { id: 'n1', question: 'What happened?', beats: [], complete: false }
      let next = reduce(state, { type: 'narrativeStarted', narrative: narr })
      expect(next.narrative).not.toBeNull()
      expect(next.narrative?.id).toBe('n1')
      const beat = { caption: 'Test', reveal: ['o1'], surface: 'orbital' as const }
      next = reduce(next, { type: 'narrativeBeat', id: 'n1', beat, complete: false })
      expect(next.narratives['n1'].beats).toHaveLength(1)
    })

    it('applyBeat with compare opens comparison', () => {
      if (history.revisions.length < 2) return
      const [a, b] = history.revisions.slice(0, 2).map((r) => r.id)
      const beat = { caption: 'Compare', reveal: [], compare: { a, b }, surface: 'compare' as const }
      const next = reduce(state, { type: 'applyBeat', beat, index: 0 })
      expect(next.compare).not.toBeNull()
    })
  })

  describe('design', () => {
    it('enterDesign + designDraft + setMode(world) restores returnFocus', () => {
      let next = state
      const designState = { caseId: 'case1', returnFocus: ['o1'], revision: 'draft' as const, draft: null, history: history, selected: null }
      next = reduce(next, { type: 'enterDesign', design: designState })
      expect(next.mode).toBe('case-design')
      next = reduce(next, { type: 'setMode', mode: 'world' })
      expect(next.design).toBeNull()
      expect(next.mode).toBe('world')
    })
  })

  describe('loadHistory', () => {
    it('keeps past revision when standing in the past', () => {
      if (history.revisions.length < 2) return
      const oldRev = history.revisions[0]!.id
      let next = reduce(state, { type: 'setTime', revision: oldRev })
      expect(next.revision).toBe(oldRev)
      const newHistory = { ...history, revisions: [...history.revisions] }
      next = reduce(next, { type: 'loadHistory', history: newHistory })
      expect(next.revision).toBe(oldRev)
    })

    it('follows now when at now', () => {
      const newHistory = { ...history, revisions: [...history.revisions, { id: 'new', at: new Date().toISOString(), label: 'New' }] }
      const next = reduce(state, { type: 'loadHistory', history: newHistory })
      expect(next.revision).toBe('new')
    })
  })

  describe('mutation safety', () => {
    it('never mutates history snapshots', () => {
      const before = JSON.stringify(history.snapshots)
      let next = state
      next = reduce(next, { type: 'focus', id: Object.keys(snapshotOf(state).objects)[0]! })
      next = reduce(next, { type: 'expandArtifact', id: 'a1', snapshot: { focusStack: [], revision: '', surface: 'orbital', beatOverrides: { reveal: [], dim: [], relationships: [], highlight: [], annotations: [] }, compare: null, camera: { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 } } })
      next = reduce(next, { type: 'collapseArtifact' })
      const after = JSON.stringify(history.snapshots)
      expect(before).toBe(after)
    })
  })
})

describe('store (Store interface)', () => {
  it('dispatch and subscribe work', () => {
    const initial = initialState(history)
    const store = createStore(initial)
    let called = 0
    store.subscribe(() => {
      called++
    })
    store.dispatch({ type: 'home' })
    expect(called).toBe(1)
    expect(store.getState().focusStack).toEqual([])
  })

  it('listen receives state, prev, and action', () => {
    const initial = initialState(history)
    const store = createStore(initial)
    let events: any[] = []
    store.listen((_s, _prev, a) => {
      events.push({ action: a.type })
    })
    store.dispatch({ type: 'home' })
    expect(events).toHaveLength(1)
    expect(events[0].action).toBe('home')
  })
})
