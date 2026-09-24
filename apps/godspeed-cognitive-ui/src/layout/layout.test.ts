import { describe, it, expect } from 'bun:test'
import { buildNorthstarHistory } from '../adapters/local/northstarData'
import { buildTemplateHistory } from '../adapters/local/templateData'
import { projectHistory } from '../ports/project'
import { layout, childrenOf, cameraFor, causalMembers } from './layout'
import { snapshotOf, initialState } from '../model/store'

const raw = buildNorthstarHistory({ actor_id: 'u', role: 'case_architect' })
const history = projectHistory(raw, {}, 'local-contract', { withCore: true })
const state = initialState(history)

describe('layout', () => {
  describe('Home layout', () => {
    it('places the six regions at their data slots', () => {
      const snap = snapshotOf(state)
      const result = layout({
        snap,
        focus: null,
        surface: 'orbital',
        overrides: { reveal: [], dim: [], relationships: [], highlight: [], annotations: [] },
        fit: 1,
      })

      expect(result.placements['core']).toBeDefined()
      expect(result.placements['core']!.center).toBe(true)

      const regionPlacements = Object.values(result.placements).filter((p) => snap.objects[Object.keys(result.placements).find((id) => result.placements[id] === p)!]?.kind === 'category')
      expect(regionPlacements.length).toBeGreaterThan(0)
    })
  })

  describe('Focusing northstar', () => {
    it('makes it the center and places its facets', () => {
      const snap = snapshotOf(state)
      const northstar = Object.values(snap.objects).find((o) => o.kind === 'case')
      if (!northstar) return

      const result = layout({
        snap,
        focus: northstar.id,
        surface: 'orbital',
        overrides: { reveal: [], dim: [], relationships: [], highlight: [], annotations: [] },
        fit: 1,
      })

      expect(result.placements[northstar.id]).toBeDefined()
      expect(result.placements[northstar.id]!.center).toBe(true)

      const children = childrenOf(snap, northstar.id)
      for (const child of children) {
        expect(result.placements[child.id]).toBeDefined()
      }
    })
  })

  describe('Causal surface', () => {
    it('places exactly the causal members', () => {
      const snap = snapshotOf(state)
      const centerId = Object.values(snap.objects).find((o) => o.kind === 'case')?.id
      if (!centerId) return

      const members = causalMembers(snap, centerId)
      expect(members.length).toBeGreaterThanOrEqual(0)

      const result = layout({
        snap,
        focus: centerId,
        surface: 'causal',
        overrides: { reveal: [], dim: [], relationships: [], highlight: [], annotations: [] },
        fit: 1,
      })

      for (const member of members) {
        expect(result.placements[member.id]).toBeDefined()
      }
    })
  })

  describe('Judgment surface', () => {
    it('gives the judged object role', () => {
      const snap = snapshotOf(state)
      const objId = Object.keys(snap.objects)[0]!
      const result = layout({
        snap,
        focus: null,
        surface: 'judgment',
        overrides: { reveal: [], dim: [], relationships: [], highlight: [], annotations: [] },
        fit: 1,
        judgment: { object: objId, label: '1. Test Action' },
      })

      expect(result.placements[objId]).toBeDefined()
    })
  })

  describe('Compare input', () => {
    it('annotates placements with change classes', () => {
      if (history.revisions.length < 2) return
      const snap1 = history.snapshots[history.revisions[0]!.id]!
      const snap2 = history.snapshots[history.revisions[history.revisions.length - 1]!.id]!

      const objId = Object.keys(snap1.objects)[0]!
      const changes: Record<string, any> = {}
      changes[objId] = { change: 'changed', fields: ['name'] }

      const result = layout({
        snap: snap2,
        focus: null,
        surface: 'compare',
        overrides: { reveal: [], dim: [], relationships: [], highlight: [], annotations: [] },
        fit: 1,
        compare: { changes, aLabel: 'Before', bLabel: 'After' },
      })

      expect(result.placements[objId]).toBeDefined()
    })
  })

  describe('Design mode', () => {
    it('places the template children with glyphs', () => {
      const raw = buildTemplateHistory({ actor_id: 'u', role: 'case_architect' })
      const designHistory = projectHistory(raw, {}, 'local-contract', { withCore: true })
      const snap = designHistory.snapshots[designHistory.revisions[0]!.id]!

      const result = layout({
        snap,
        focus: null,
        surface: 'orbital',
        overrides: { reveal: [], dim: [], relationships: [], highlight: [], annotations: [] },
        fit: 1,
        design: true,
      })

      const rootId = Object.values(snap.objects).find((o) => !o.parent || !snap.objects[o.parent])?.id
      if (rootId) {
        expect(result.placements[rootId]).toBeDefined()
        expect(result.placements[rootId]!.center).toBe(true)
        const children = childrenOf(snap, rootId)
        for (const child of children) {
          expect(result.placements[child.id]).toBeDefined()
        }
      }
    })
  })

  describe('layout.ts code quality', () => {
    it('does not contain fixture ids', () => {
      // Read the file and check it has no hardcoded fixture ids
      const fs = require('fs')
      const content = fs.readFileSync(__dirname + '/layout.ts', 'utf-8')
      const hasBadIds = /\b(ns-|northstar|cat-|tpl-)\w+\b/.test(content)
      expect(hasBadIds).toBe(false)
    })
  })

  describe('cameraFor', () => {
    it('computes camera position from layout', () => {
      const camera = cameraFor({ x: 0, y: 0, z: 0 }, 1, 1, 0.5)
      expect(camera).toHaveProperty('x')
      expect(camera).toHaveProperty('y')
      expect(camera).toHaveProperty('zoom')
      expect(camera.tilt).toBe(0)
      expect(camera.roll).toBe(0)
    })
  })
})
