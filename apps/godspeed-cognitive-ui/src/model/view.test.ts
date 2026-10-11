import { describe, it, expect } from 'bun:test'
import { buildNorthstarHistory } from '../adapters/local/northstarData'
import { projectHistory } from '../ports/project'
import {
  diffObjects,
  mergeForCompare,
  worldView,
  findArtifact,
} from './view'
import { initialState, snapshotOf, reduce } from './store'

const raw = buildNorthstarHistory({ actor_id: 'u', role: 'case_architect' })
const history = projectHistory(raw, {}, 'local-contract', { withCore: true })
const state = initialState(history)

describe('view', () => {
  describe('diffObjects', () => {
    it('classifies added objects', () => {
      const snap1 = snapshotOf(state)
      const snap2 = { ...snap1, objects: { ...snap1.objects, 'new-obj': { id: 'new-obj', kind: 'item' as const, title: 'New', parent: null, salience: 1 } } }
      const diff = diffObjects(snap1, snap2)
      expect(diff['new-obj']).toEqual({
        change: 'added',
        b: snap2.objects['new-obj'],
        fields: [],
      })
    })

    it('classifies removed objects', () => {
      const snap1 = snapshotOf(state)
      const objId = Object.keys(snap1.objects)[0]!
      const snap2 = { ...snap1, objects: Object.fromEntries(Object.entries(snap1.objects).filter(([k]) => k !== objId)) }
      const diff = diffObjects(snap1, snap2)
      expect(diff[objId]!.change).toBe('removed')
      expect(diff[objId]!.a).toBeTruthy()
    })

    it('detects changed title', () => {
      const snap1 = snapshotOf(state)
      const objId = Object.keys(snap1.objects)[0]!
      const obj = snap1.objects[objId]!
      const snap2 = { ...snap1, objects: { ...snap1.objects, [objId]: { ...obj, title: 'Changed Title' } } }
      const diff = diffObjects(snap1, snap2)
      expect(diff[objId]!.change).toBe('changed')
      expect(diff[objId]!.fields).toContain('name')
    })

    it('detects changed status', () => {
      const snap1 = snapshotOf(state)
      const objId = Object.keys(snap1.objects)[0]!
      const obj = snap1.objects[objId]!
      const snap2 = {
        ...snap1,
        objects: {
          ...snap1.objects,
          [objId]: { ...obj, status: { label: 'Changed', tone: 'ok' as const } },
        },
      }
      const diff = diffObjects(snap1, snap2)
      expect(diff[objId]!.change).toBe('changed')
      expect(diff[objId]!.fields).toContain('status')
    })

    it('marks unchanged objects as same', () => {
      const snap1 = snapshotOf(state)
      const diff = diffObjects(snap1, snap1)
      for (const change of Object.values(diff)) {
        expect(change.change).toBe('same')
        expect(change.fields).toHaveLength(0)
      }
    })
  })

  describe('mergeForCompare', () => {
    it('keeps removed objects as ghosts', () => {
      const snap1 = snapshotOf(state)
      const objId = Object.keys(snap1.objects)[0]!
      const snap2 = { ...snap1, objects: Object.fromEntries(Object.entries(snap1.objects).filter(([k]) => k !== objId)) }
      const merged = mergeForCompare(snap1, snap2)
      expect(merged.objects[objId]).toBeTruthy()
      expect(merged.objects[objId]?.ghost).toBe(true)
    })

    it('never duplicates ids', () => {
      const snap1 = snapshotOf(state)
      const objId = Object.keys(snap1.objects)[0]!
      const snap2 = { ...snap1, objects: { ...snap1.objects, [objId]: { ...snap1.objects[objId]!, title: 'Changed' } } }
      const merged = mergeForCompare(snap1, snap2)
      const ids = Object.keys(merged.objects)
      const uniqueIds = new Set(ids)
      expect(ids).toHaveLength(uniqueIds.size)
    })

    it('merges artifacts from both sides', () => {
      const snap1 = { ...snapshotOf(state), artifacts: { 'a1': { id: 'a1', name: 'Art1' } as any } }
      const snap2 = { ...snap1, artifacts: { 'a2': { id: 'a2', name: 'Art2' } as any } }
      const merged = mergeForCompare(snap1, snap2)
      expect(merged.artifacts).toHaveProperty('a1')
      expect(merged.artifacts).toHaveProperty('a2')
    })
  })

  describe('worldView', () => {
    it('returns world view in normal mode', () => {
      const view = worldView(state)
      expect(view.snap).toBeTruthy()
      expect(view.surface).toBe('orbital')
      expect(view.design).toBe(false)
      expect(view.compare).toBeNull()
    })

    it('returns compare surface when compare is active', () => {
      if (history.revisions.length < 2) return
      const [a, b] = history.revisions.slice(0, 2).map((r) => r.id)
      let next = reduce(state, { type: 'openCompare', a, b })
      const view = worldView(next)
      expect(view.surface).toBe('compare')
      expect(view.compare).not.toBeNull()
      expect(view.compare!.changes).toBeTruthy()
    })
  })

  describe('findArtifact', () => {
    it('finds artifact in current snapshot', () => {
      const snap = snapshotOf(state)
      const artRef = Object.keys(snap.artifacts)[0]
      if (!artRef) return
      const artifact = findArtifact(state, artRef)
      expect(artifact).not.toBeNull()
      expect(artifact!.ref).toBe(artRef)
    })

    it('finds artifact in history when not in current snapshot', () => {
      if (history.revisions.length < 2) return
      const oldSnap = history.snapshots[history.revisions[0]!.id]!
      const artRef = Object.keys(oldSnap.artifacts)[0]
      if (!artRef) return
      let next = reduce(state, { type: 'setTime', revision: history.revisions[1]!.id })
      const artifact = findArtifact(next, artRef)
      if (oldSnap.artifacts[artRef]) {
        expect(artifact).not.toBeNull()
      }
    })
  })
})
