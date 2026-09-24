import { describe, test, expect, beforeEach } from 'bun:test'
import type { Beat } from '../model/types'
import type { Clock } from './player'
import { createStore, initialState } from '../model/store'
import { startNarrativePlayer, DEFAULT_HOLD_MS } from './player'
import { projectHistory } from '../ports/project'
import { buildNorthstarHistory } from '../adapters/local/northstarData'

// Fake clock for testing
class FakeClock implements Clock {
  private now_ = 0
  private timers: Array<{ at: number; fn: () => void; id: number }> = []
  private nextId = 1

  now(): number {
    return this.now_
  }

  setTimeout(fn: () => void, ms: number): unknown {
    const id = this.nextId++
    const at = this.now_ + ms
    this.timers.push({ at, fn, id })
    this.timers.sort((a, b) => a.at - b.at)
    return id
  }

  clearTimeout(h: unknown): void {
    const id = h as number
    this.timers = this.timers.filter((t) => t.id !== id)
  }

  advance(ms: number): void {
    this.now_ += ms
    while (this.timers.length > 0 && this.timers[0]!.at <= this.now_) {
      const { fn } = this.timers.shift()!
      fn()
    }
  }
}

function createTestStore() {
  const snaps = buildNorthstarHistory({ actor_id: 'usr-sam', role: 'case_architect' })
  const history = projectHistory(snaps, {}, 'local-contract', { withCore: false })
  return createStore(initialState(history))
}

describe('NarrativePlayer', () => {
  let clock: FakeClock
  let store: ReturnType<typeof createStore>
  let dispose: () => void

  beforeEach(() => {
    clock = new FakeClock()
    store = createTestStore()
    dispose = startNarrativePlayer(store, clock)
  })

  test('narrativeStarted applies beat 0 immediately', () => {
    const beat0: Beat = { caption: 'First beat' }

    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-1',
        question: 'Why?',
        beats: [beat0],
        complete: false,
      },
    })

    const state = store.getState()
    expect(state.narrative?.id).toBe('test-1')
    expect(state.narrative?.index).toBe(0)
    expect(state.overrides.caption).toBe('First beat')
  })

  test('next beat applies after its holdMs when complete is true', () => {
    const beat0: Beat = { caption: 'Beat 0', holdMs: 100 }
    const beat1: Beat = { caption: 'Beat 1', holdMs: 50 }

    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-2',
        question: 'Why?',
        beats: [beat0, beat1],
        complete: true,
      },
    })

    expect(store.getState().narrative?.index).toBe(0)

    clock.advance(100)
    let state = store.getState()
    expect(state.narrative?.index).toBe(1)
    expect(state.overrides.caption).toBe('Beat 1')

    // Last beat is done once hold elapses
    clock.advance(50)
    state = store.getState()
    expect(state.narrative?.status).toBe('done')
  })

  test('waits when beats have not arrived yet', () => {
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-3',
        question: 'Why?',
        beats: [],
        complete: false,
      },
    })

    expect(store.getState().narrative?.index).toBe(-1)

    // Advance time, but no beat arrives
    clock.advance(1000)
    expect(store.getState().narrative?.index).toBe(-1)

    // Now dispatch a beat
    store.dispatch({
      type: 'narrativeBeat',
      id: 'test-3',
      beat: { caption: 'Late beat', holdMs: 100 },
      complete: true,
    })

    expect(store.getState().narrative?.index).toBe(0)

    clock.advance(100)
    expect(store.getState().narrative?.status).toBe('done')
  })

  test('ends after last beat once narrativeComplete arrives', () => {
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-4',
        question: 'Why?',
        beats: [{ caption: 'Only beat', holdMs: 50 }],
        complete: false,
      },
    })

    clock.advance(50)
    expect(store.getState().narrative?.status).toBe('playing')

    store.dispatch({ type: 'narrativeComplete', id: 'test-4' })
    clock.advance(1)

    expect(store.getState().narrative?.status).toBe('done')
  })

  test('pauseNarrative stops advancing and saves remaining hold', () => {
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-5',
        question: 'Why?',
        beats: [
          { caption: 'Beat 0', holdMs: 1000 },
          { caption: 'Beat 1', holdMs: 100 },
        ],
        complete: false,
      },
    })

    clock.advance(300) // 700ms remaining
    store.dispatch({ type: 'pauseNarrative' })

    let state = store.getState()
    expect(state.narrative?.status).toBe('paused')
    expect(state.narrative?.index).toBe(0)

    // Advance time, should not trigger next beat
    clock.advance(500)
    state = store.getState()
    expect(state.narrative?.index).toBe(0)
  })

  test('resumeNarrative continues from same index after minimum 900ms', () => {
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-6',
        question: 'Why?',
        beats: [
          { caption: 'Beat 0', holdMs: 1000 },
          { caption: 'Beat 1', holdMs: 100 },
        ],
        complete: false,
      },
    })

    clock.advance(300)
    store.dispatch({ type: 'pauseNarrative' })
    store.dispatch({ type: 'resumeNarrative' })

    // Should not advance immediately
    expect(store.getState().narrative?.index).toBe(0)

    // But within 900ms it should advance
    clock.advance(900)
    expect(store.getState().narrative?.index).toBe(1)
  })

  test('narrativeFailed after 2 beats ends with error set', () => {
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-7',
        question: 'Why?',
        beats: [
          { caption: 'Beat 0', holdMs: 50 },
          { caption: 'Beat 1', holdMs: 50 },
        ],
        complete: false,
      },
    })

    clock.advance(50)
    expect(store.getState().narrative?.index).toBe(1)

    store.dispatch({ type: 'narrativeFailed', id: 'test-7', error: 'Agent connection lost' })

    let state = store.getState()
    expect(state.narrative?.error).toBe('Agent connection lost')

    // Should end after remaining hold
    clock.advance(50)
    state = store.getState()
    expect(state.narrative?.status).toBe('done')
  })

  test('home cancels narrative', () => {
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-8',
        question: 'Why?',
        beats: [
          { caption: 'Beat 0', holdMs: 1000 },
          { caption: 'Beat 1', holdMs: 100 },
        ],
        complete: false,
      },
    })

    store.dispatch({ type: 'home' })

    expect(store.getState().narrative).toBeNull()

    // Advancing should not trigger next beat
    clock.advance(1000)
    expect(store.getState().narrative).toBeNull()
  })

  test('uses default hold when beat holdMs is undefined', () => {
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-9',
        question: 'Why?',
        beats: [
          { caption: 'Beat 0' },
          { caption: 'Beat 1' },
        ],
        complete: true,
      },
    })

    expect(store.getState().narrative?.index).toBe(0)

    clock.advance(DEFAULT_HOLD_MS)
    expect(store.getState().narrative?.index).toBe(1)

    clock.advance(DEFAULT_HOLD_MS)
    expect(store.getState().narrative?.status).toBe('done')
  })

  test('switching narratives drops previous timer', () => {
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-10',
        question: 'Why?',
        beats: [{ caption: 'Beat 0', holdMs: 1000 }],
        complete: false,
      },
    })

    clock.advance(500)

    // Switch to different narrative
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-11',
        question: 'What?',
        beats: [{ caption: 'Different beat', holdMs: 100 }],
        complete: false,
      },
    })

    // Advance original wait time from first narrative
    clock.advance(500) // Would trigger beat 1 of first narrative
    expect(store.getState().narrative?.id).toBe('test-11')
    expect(store.getState().narrative?.index).toBe(0) // Still on first beat of new narrative
  })

  test('dispose clears timers and stops player', () => {
    dispose()

    // Start with a working player first
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-12',
        question: 'Why?',
        beats: [
          { caption: 'Beat 0', holdMs: 50 },
          { caption: 'Beat 1' },
        ],
        complete: false,
      },
    })

    // After dispose, no beats should be applied automatically
    expect(store.getState().narrative?.index).toBe(-1)

    // Advancing should not trigger beats
    clock.advance(50)
    expect(store.getState().narrative?.index).toBe(-1)
  })

  test('beats with focus, reveal, and other attributes are applied', () => {
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-13',
        question: 'Why?',
        beats: [
          {
            caption: 'Complex beat',
            focus: 'ns-goal',
            reveal: ['ns-impl', 'ns-failure'],
            highlight: ['ns-failure'],
            holdMs: 100,
          },
        ],
        complete: true,
      },
    })

    const state = store.getState()
    expect(state.overrides.caption).toBe('Complex beat')
    expect(state.overrides.reveal).toContain('ns-impl')
    expect(state.overrides.highlight).toContain('ns-failure')
  })

  test('streamed beats come after narrativeStarted', () => {
    store.dispatch({
      type: 'narrativeStarted',
      narrative: {
        id: 'test-14',
        question: 'Why?',
        beats: [],
        complete: false,
      },
    })

    expect(store.getState().narrative?.index).toBe(-1)

    // Stream beat 0
    store.dispatch({
      type: 'narrativeBeat',
      id: 'test-14',
      beat: { caption: 'Streamed beat 0', holdMs: 100 },
      complete: false,
    })

    expect(store.getState().narrative?.index).toBe(0)

    clock.advance(100)

    // Stream beat 1
    store.dispatch({
      type: 'narrativeBeat',
      id: 'test-14',
      beat: { caption: 'Streamed beat 1', holdMs: 100 },
      complete: false,
    })

    expect(store.getState().narrative?.index).toBe(1)
  })
})
