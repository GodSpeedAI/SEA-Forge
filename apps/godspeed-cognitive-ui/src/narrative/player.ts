import type { Store } from '../model/store'
import type { UiState } from '../model/types'

// Plays the current narrative's beats in order as they stream in. Each beat holds for its
// holdMs, then the next one applies. Pausing (any user interaction) keeps a checkpoint with the
// time left on the current beat; resuming continues from that beat instead of restarting. A
// narrative whose agent stream ended (complete, or failed) ends after its last delivered beat.

export interface Clock {
  now(): number
  setTimeout(fn: () => void, ms: number): unknown
  clearTimeout(h: unknown): void
}

export const DEFAULT_HOLD_MS = 3200

const realClock: Clock = {
  now: () => Date.now(),
  setTimeout: (fn, ms) => globalThis.setTimeout(fn, ms),
  clearTimeout: (h) => globalThis.clearTimeout(h as ReturnType<typeof setTimeout>),
}

/** `pace` multiplies every beat's hold (1 = authored timing; E2E on software GL uses >1). */
export function startNarrativePlayer(store: Store, clock: Clock = realClock, pace = 1): () => void {
  let timer: unknown = null
  /** When the current beat's hold ends (clock time), and how long is left when paused. */
  let dueAt = 0
  let remaining: number | null = null
  let playingId: string | null = null

  const clear = () => {
    if (timer !== null) clock.clearTimeout(timer)
    timer = null
  }

  const beatsOf = (s: UiState) => (s.narrative ? s.narratives[s.narrative.id] : undefined)

  /** Apply the next beat if it is due and available; end if the stream is complete. */
  const advance = () => {
    const s = store.getState()
    const n = beatsOf(s)
    if (!s.narrative || !n || s.narrative.status !== 'playing' || timer !== null) return
    const next = s.narrative.index + 1
    const beat = n.beats[next]
    if (beat) {
      store.dispatch({ type: 'applyBeat', beat, index: next })
      schedule((beat.holdMs ?? DEFAULT_HOLD_MS) * pace)
    } else if (n.complete) {
      store.dispatch({ type: 'endNarrative' })
    }
    // Otherwise wait for the next streamed beat (narrativeBeat re-enters advance()).
  }

  const schedule = (ms: number) => {
    clear()
    dueAt = clock.now() + ms
    timer = clock.setTimeout(() => {
      timer = null
      advance()
    }, ms)
  }

  const off = store.listen((s, prev, a) => {
    const id = s.narrative?.id ?? null
    if (id !== playingId) {
      // A different (or no) narrative: drop any pending hold.
      clear()
      remaining = null
      playingId = id
    }
    if (!s.narrative) return
    switch (a.type) {
      case 'narrativeStarted':
      case 'narrativeBeat':
      case 'narrativeComplete':
      case 'narrativeFailed':
        advance()
        break
      case 'pauseNarrative':
        if (prev.narrative?.status === 'playing' && s.narrative.status === 'paused') {
          remaining = timer !== null ? Math.max(0, dueAt - clock.now()) : 0
          clear()
        }
        break
      case 'resumeNarrative':
        if (prev.narrative?.status === 'paused' && s.narrative.status === 'playing') {
          const left = remaining ?? 0
          remaining = null
          // Resume from the checkpoint: finish the current beat's hold (at least a short beat).
          if (s.narrative.index < 0) advance()
          else schedule(Math.max(900, left))
        }
        break
    }
  })

  return () => {
    clear()
    off()
  }
}
