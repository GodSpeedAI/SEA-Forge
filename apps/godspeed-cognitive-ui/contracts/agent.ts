/**
 * Agent contract.
 *
 * Agents act through the SAME interaction model as humans: they read current UI context and request
 * representational operations through UI-owned actions. Nothing here lets an agent mutate governed
 * case truth directly, and narration is interruptible by construction.
 */

export interface UiContextSnapshot {
  readonly focus: string | null
  readonly visibleObjectIds: readonly string[]
  readonly disclosure: 'minimal' | 'summary' | 'source'
}

export interface AgentRequest {
  readonly requestId: string
  readonly question: string
  readonly context: UiContextSnapshot
}

export interface NarrationBeat {
  readonly index: number
  readonly text: string
  /** Artifact refs the beat cites; never a bare claim without evidence. */
  readonly evidenceRefs: readonly string[]
}

export interface NarrationStream {
  readonly beats: AsyncIterable<NarrationBeat>
  /** Interruption is part of the contract, not an afterthought. */
  interrupt(): void
}

export interface AgentAdapter {
  /** Available only when an agent capability is configured; absence degrades, it does not block. */
  readonly available: boolean
  ask(request: AgentRequest): Promise<NarrationStream>
}
