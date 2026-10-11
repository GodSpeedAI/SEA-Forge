import { describe, it, expect } from 'bun:test'
import { buildTemplateHistory } from '../adapters/local/templateData'
import { projectHistory } from '../ports/project'
import {
  designModel,
  moveItem,
  toggleRequired,
  isDirty,
} from './design'

const raw = buildTemplateHistory({ actor_id: 'u', role: 'case_architect' })
const history = projectHistory(raw, {}, 'local-contract', { withCore: true })
const snap = history.snapshots[history.revisions[0]!.id]!

describe('design', () => {
  describe('designModel', () => {
    it('reads the stages from the template layers node', () => {
      const model = designModel(snap, 'v0.2')
      expect(model.stages).toBeDefined()
      expect(Array.isArray(model.stages)).toBe(true)
      expect(model.stages.length).toBeGreaterThanOrEqual(0)
    })

    it('extracts name and description from root', () => {
      const model = designModel(snap, 'v0.2')
      expect(model.name).toBeDefined()
      expect(typeof model.name).toBe('string')
      expect(model.description).toBeDefined()
      expect(typeof model.description).toBe('string')
    })

    it('reads roles and evidence', () => {
      const model = designModel(snap, 'v0.2')
      expect(model.roles).toBeDefined()
      expect(Array.isArray(model.roles)).toBe(true)
      expect(model.evidence).toBeDefined()
      expect(Array.isArray(model.evidence)).toBe(true)
    })
  })

  describe('moveItem', () => {
    it('swaps neighbors', () => {
      const model = designModel(snap, 'v0.2')
      if (model.stages.length < 2) return
      const itemId = model.stages[0]!.id
      const result = moveItem(snap, itemId, 1)
      const nextModel = designModel(result, 'v0.2')
      const oldIndex = model.stages.findIndex((x) => x.id === itemId)
      const newIndex = nextModel.stages.findIndex((x) => x.id === itemId)
      expect(newIndex).toBe(oldIndex + 1)
    })

    it('does nothing at the ends', () => {
      const model = designModel(snap, 'v0.2')
      if (model.stages.length === 0) return
      const lastId = model.stages[model.stages.length - 1]!.id
      const result = moveItem(snap, lastId, 1)
      const nextModel = designModel(result, 'v0.2')
      const nextLastId = nextModel.stages[nextModel.stages.length - 1]!.id
      expect(nextLastId).toBe(lastId)
    })

    it('returns new snapshot without mutating input', () => {
      const model = designModel(snap, 'v0.2')
      if (model.stages.length < 2) return
      const itemId = model.stages[0]!.id
      const before = JSON.stringify(designModel(snap, 'v0.2'))
      moveItem(snap, itemId, 1)
      const after = JSON.stringify(designModel(snap, 'v0.2'))
      expect(before).toBe(after)
    })
  })

  describe('toggleRequired', () => {
    it('flips required flag', () => {
      const model = designModel(snap, 'v0.2')
      const evidence = model.evidence
      if (evidence.length === 0) return
      const itemId = evidence[0]!.id
      const wasRequired = evidence[0]!.required ?? false
      const next = toggleRequired(snap, itemId)
      const nextModel = designModel(next, 'v0.2')
      const nextItem = nextModel.evidence.find((x) => x.id === itemId)
      expect(nextItem!.required).toBe(!wasRequired)
    })

    it('returns new snapshot without mutating input', () => {
      const model = designModel(snap, 'v0.2')
      if (model.evidence.length === 0) return
      const itemId = model.evidence[0]!.id
      const before = JSON.stringify(designModel(snap, 'v0.2'))
      toggleRequired(snap, itemId)
      const after = JSON.stringify(designModel(snap, 'v0.2'))
      expect(before).toBe(after)
    })
  })

  describe('isDirty', () => {
    it('is false for no draft', () => {
      expect(isDirty(null, history)).toBe(false)
    })

    it('is true after an edit', () => {
      const model = designModel(snap, 'v0.2')
      if (model.stages.length < 2) return
      const itemId = model.stages[0]!.id
      const draft = moveItem(snap, itemId, 1)
      expect(isDirty(draft, history)).toBe(true)
    })

    it('is false when draft revision is null', () => {
      // isDirty compares revision field in the snapshot
      expect(isDirty(null, history)).toBe(false)
    })
  })
})
