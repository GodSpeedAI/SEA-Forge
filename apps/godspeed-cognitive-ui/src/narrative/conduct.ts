import { cursorOf, focusOf, type Store } from '../model/store'
import type { Beat, UiState } from '../model/types'
import type { NarrationBeat, NarrationPort } from '../ports/contract'

// Turns an agent's narration stream into semantic beats over the existing grammar. A directive
// can only select, focus, arrange, reveal, highlight, annotate, open an artifact, move through
// time or open a comparison: it cannot create UI or change case data. Losing the agent adapter
// ends the explanation where it is and leaves direct interaction untouched.

/** Translates one narration beat, given the history cursor the previous beat left the world at. */
export function beatFromNarration(nb: NarrationBeat, s: UiState, fromCursor: string): { beat: Beat; cursor: string } {
  const beat: Beat = { caption: nb.thoughtText, citations: [...nb.evidenceCitations], holdMs: DEFAULT_BEAT_MS }
  if (nb.grounded_answer) beat.grounded = nb.grounded_answer
  let cursor = fromCursor
  for (const d of nb.directives ?? []) {
    if (d.focus) beat.focus = d.focus
    if (d.camera) beat.focus = d.camera.target === 'core' ? undefined : d.camera.target
    if (d.highlight) beat.highlight = [...d.highlight]
    if (d.annotate) beat.annotate = d.annotate.map((x) => ({ target: x.target, label: x.label }))
    if (d.openArtifact) {
      beat.artifacts = [...(beat.artifacts ?? []), d.openArtifact.ref]
      beat.holdMs = ARTIFACT_BEAT_MS
    }
    if (d.temporalStep) {
      const revs = s.history.revisions
      const i = revs.findIndex((r) => r.id === cursor)
      const j = Math.max(0, Math.min(revs.length - 1, i + d.temporalStep))
      cursor = revs[j]!.id
      beat.time = cursor
    }
    const x = d.x
    if (!x) continue
    if (x.arrange && x.arrange !== 'compare') beat.surface = x.arrange
    if (x.temporalTo) {
      cursor = cursorOf(s.history, x.temporalTo)
      beat.time = cursor
    }
    if (x.compare) {
      beat.compare = { a: x.compare.a, b: x.compare.b }
      beat.holdMs = ARTIFACT_BEAT_MS
    }
    if (x.reveal) beat.reveal = [...(beat.reveal ?? []), ...x.reveal]
    if (x.dim) beat.dim = [...x.dim]
  }
  return { beat, cursor }
}

const DEFAULT_BEAT_MS = 3400
const ARTIFACT_BEAT_MS = 4200

let seq = 0

/** Asks the agent; returns false when there is no agent or it has nothing to say here. */
export function explain(store: Store, agent: NarrationPort | null, question: string): boolean {
  if (!agent) return false
  const s = store.getState()
  let stream
  try {
    stream = agent.explain(question, { caseId: s.history.caseId, focus: focusOf(s), cursor: s.revision })
  } catch (e) {
    store.dispatch({ type: 'agentAvailability', available: false })
    return false
  }
  if (!stream) return false
  const id = `${(stream as { id?: string }).id ?? 'explanation'}#${++seq}`
  store.dispatch({ type: 'narrativeStarted', narrative: { id, question, beats: [], complete: false } })
  void (async () => {
    let cursor = store.getState().revision
    try {
      for await (const nb of stream.beats) {
        const now = store.getState()
        if (now.narrative?.id !== id) {
          stream.interrupt()
          return
        }
        const t = beatFromNarration(nb, now, cursor)
        cursor = t.cursor
        store.dispatch({ type: 'narrativeBeat', id, beat: t.beat, complete: false })
      }
      store.dispatch({ type: 'narrativeComplete', id })
    } catch (e) {
      store.dispatch({ type: 'narrativeFailed', id, error: e instanceof Error ? e.message : String(e) })
      store.dispatch({ type: 'agentAvailability', available: false })
    }
  })()
  return true
}
