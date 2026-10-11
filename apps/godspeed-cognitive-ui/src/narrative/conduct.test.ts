import { describe, test, expect, beforeEach } from 'bun:test'
import type { NarrationBeat } from '../ports/contract'
import { beatFromNarration } from './conduct'
import { explain } from './conduct'
import { createLocalAgent } from './localAgent'
import { createStore, initialState } from '../model/store'
import { projectHistory } from '../ports/project'
import { buildNorthstarHistory } from '../adapters/local/northstarData'
import type { UiState } from '../model/types'

describe('beatFromNarration', () => {
  let state: UiState

  beforeEach(() => {
    const snaps = buildNorthstarHistory({ actor_id: 'usr-sam', role: 'case_architect' })
    const history = projectHistory(snaps, {}, 'local-contract', { withCore: false })
    state = initialState(history)
  })

  test('maps focus directive', () => {
    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Focus on northstar',
      evidenceCitations: [],
      directives: [{ focus: 'ns-goal' }],
    }

    const { beat, cursor } = beatFromNarration(nb, state, state.revision)

    expect(beat.focus).toBe('ns-goal')
    expect(beat.caption).toBe('Focus on northstar')
    expect(cursor).toBe(state.revision)
  })

  test('maps highlight directive', () => {
    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Highlight items',
      evidenceCitations: [],
      directives: [{ highlight: ['ns-impl', 'ns-failure'] }],
    }

    const { beat } = beatFromNarration(nb, state, state.revision)

    expect(beat.highlight).toEqual(['ns-impl', 'ns-failure'])
  })

  test('maps annotate directive', () => {
    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Annotate',
      evidenceCitations: [],
      directives: [{ annotate: [{ target: 'ns-impl', label: 'Verified' }] }],
    }

    const { beat } = beatFromNarration(nb, state, state.revision)

    expect(beat.annotate).toBeDefined()
    expect(beat.annotate?.[0]?.target).toBe('ns-impl')
    expect(beat.annotate?.[0]?.label).toBe('Verified')
  })

  test('openArtifact increases holdMs to ARTIFACT_BEAT_MS', () => {
    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Show artifact',
      evidenceCitations: [],
      directives: [{ openArtifact: { ref: 'evi-assumption', level: 'summary' } }],
    }

    const { beat } = beatFromNarration(nb, state, state.revision)

    expect(beat.artifacts).toContain('evi-assumption')
    expect(beat.holdMs).toBe(4200) // ARTIFACT_BEAT_MS
  })

  test('x.arrange maps to surface', () => {
    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Arrange',
      evidenceCitations: [],
      directives: [{ x: { arrange: 'orbital' } }],
    }

    const { beat } = beatFromNarration(nb, state, state.revision)

    expect(beat.surface).toBe('orbital')
  })

  test('x.reveal adds to reveal array', () => {
    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Reveal items',
      evidenceCitations: [],
      directives: [{ x: { reveal: ['ns-impl', 'ns-failure'] } }],
    }

    const { beat } = beatFromNarration(nb, state, state.revision)

    expect(beat.reveal).toContain('ns-impl')
    expect(beat.reveal).toContain('ns-failure')
  })

  test('x.dim sets dim array', () => {
    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Dim items',
      evidenceCitations: [],
      directives: [{ x: { dim: ['ns-goal'] } }],
    }

    const { beat } = beatFromNarration(nb, state, state.revision)

    expect(beat.dim).toContain('ns-goal')
  })

  test('x.compare sets up comparison', () => {
    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Compare',
      evidenceCitations: [],
      directives: [{ x: { compare: { a: '1.0', b: '1.1' } } }],
    }

    const { beat } = beatFromNarration(nb, state, state.revision)

    expect(beat.compare).toBeDefined()
    expect(beat.compare?.a).toBe('1.0')
    expect(beat.compare?.b).toBe('1.1')
    expect(beat.holdMs).toBe(4200) // ARTIFACT_BEAT_MS for comparison
  })

  test('temporalStep moves relative to cursor', () => {
    const revisions = state.history.revisions
    const startCursor = revisions[2]!.id

    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Go forward',
      evidenceCitations: [],
      directives: [{ temporalStep: 1 }],
    }

    const { beat, cursor } = beatFromNarration(nb, state, startCursor)

    expect(beat.time).toBe(cursor)
    const newIdx = revisions.findIndex((r) => r.id === cursor)
    const oldIdx = revisions.findIndex((r) => r.id === startCursor)
    expect(newIdx - oldIdx).toBe(1)
  })

  test('temporalStep clamps at ends', () => {
    const revisions = state.history.revisions
    const lastCursor = revisions[revisions.length - 1]!.id

    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Go beyond end',
      evidenceCitations: [],
      directives: [{ temporalStep: 1 }],
    }

    const { cursor } = beatFromNarration(nb, state, lastCursor)

    expect(cursor).toBe(lastCursor)
  })

  test('x.temporalTo live resolves to newest cursor', () => {
    const newestCursor = state.history.revisions[state.history.revisions.length - 1]!.id

    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Go to live',
      evidenceCitations: [],
      directives: [{ x: { temporalTo: 'live' } }],
    }

    const { cursor } = beatFromNarration(nb, state, state.history.revisions[0]!.id)

    expect(cursor).toBe(newestCursor)
  })

  test('x.temporalTo specific cursor sets cursor', () => {
    const targetCursor = state.history.revisions[2]!.id

    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Go to specific',
      evidenceCitations: [],
      directives: [{ x: { temporalTo: targetCursor } }],
    }

    const { cursor } = beatFromNarration(nb, state, state.history.revisions[0]!.id)

    expect(cursor).toBe(targetCursor)
  })

  test('camera.target core clears focus', () => {
    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Camera to core',
      evidenceCitations: [],
      directives: [{ camera: { target: 'core' } }],
    }

    const { beat } = beatFromNarration(nb, state, state.revision)

    expect(beat.focus).toBeUndefined()
  })

  test('multiple directives stack effects', () => {
    const nb: NarrationBeat = {
      index: 0,
      thoughtText: 'Multi',
      evidenceCitations: [],
      directives: [
        { focus: 'ns-goal' },
        { highlight: ['ns-impl'] },
        { x: { arrange: 'orbital', reveal: ['ns-failure'] } },
      ],
    }

    const { beat } = beatFromNarration(nb, state, state.revision)

    expect(beat.focus).toBe('ns-goal')
    expect(beat.highlight).toContain('ns-impl')
    expect(beat.surface).toBe('orbital')
    expect(beat.reveal).toContain('ns-failure')
  })
})

describe('explain', () => {
  let store: ReturnType<typeof createStore>

  beforeEach(() => {
    const snaps = buildNorthstarHistory({ actor_id: 'usr-sam', role: 'case_architect' })
    const history = projectHistory(snaps, {}, 'local-contract', { withCore: false })
    store = createStore(initialState(history))
  })

  test('with localAgent returns true and eventually has beats with complete=true', async () => {
    const agent = createLocalAgent({ pace: 1 })

    const started = explain(store, agent, 'Why did the pilot fail?')
    expect(started).toBe(true)

    // Give time for beats to stream
    await new Promise((r) => setTimeout(r, 200))

    const state = store.getState()
    const narrative = state.narrative?.id ? state.narratives[state.narrative.id] : null

    expect(narrative).toBeDefined()
    expect(narrative?.beats.length).toBeGreaterThanOrEqual(4)
    expect(narrative?.complete).toBe(true)
  })

  test('with failAfter 2 ends with error and sets agent unavailable', async () => {
    const agent = createLocalAgent({ pace: 1, failAfter: 2 })

    const started = explain(store, agent, 'Why did the pilot fail?')
    expect(started).toBe(true)

    // Give time to fail
    await new Promise((r) => setTimeout(r, 200))

    const state = store.getState()

    expect(state.narrative?.error).toBeDefined()
    expect(state.agent).toBe('unavailable')

    const narrative = state.narrative?.id ? state.narratives[state.narrative.id] : null
    expect(narrative?.beats.length).toBe(2)
  })

  test('with null agent returns false and changes nothing', () => {
    const stateBefore = store.getState()

    const started = explain(store, null, 'Why did the pilot fail?')
    expect(started).toBe(false)

    const stateAfter = store.getState()
    expect(stateAfter.narrative).toBe(stateBefore.narrative)
  })

  test('agent returns null for unknown question', async () => {
    const agent = createLocalAgent({ pace: 1 })

    const started = explain(store, agent, 'What is the meaning of life?')
    expect(started).toBe(false)
  })

  test('explain with no agent available returns false', () => {
    const started = explain(store, null, 'Why did the pilot fail?')
    expect(started).toBe(false)
  })
})
