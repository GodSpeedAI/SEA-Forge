/**
 * The environment's local state container (T03).
 *
 * The simplest repository pattern: an immutable state value, a replace-by-function update, and a
 * subscriber set. Renderers, loggers and tests all observe the same state; nothing outside the
 * action vocabulary may call `update` with arbitrary transitions.
 */
import type { UiState } from './model'

export type UiListener = (state: UiState) => void

export class CognitiveStore {
  private state: UiState
  private readonly listeners = new Set<UiListener>()

  constructor(initial: UiState) {
    this.state = initial
  }

  get(): UiState {
    return this.state
  }

  /**
   * The only mutation path. Actions own their transitions; the store owns consistency (subscribers
   * always see a complete state, never a partial patch).
   */
  update(transition: (state: UiState) => UiState): void {
    const next = transition(this.state)
    if (next === this.state) return
    this.state = next
    for (const listener of this.listeners) listener(this.state)
  }

  subscribe(listener: UiListener): () => void {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }
}
