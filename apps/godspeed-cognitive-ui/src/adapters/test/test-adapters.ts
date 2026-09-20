/**
 * Test and no-op adapters (T03 teeth).
 *
 * These stand in for the adapters a full environment would have: an always-unavailable agent, a
 * scriptable agent for interruption tests, and a scene renderer that only records the states it is
 * shown. The core must run to completion with these in place — rendering and narration are seams,
 * not requirements.
 */
import type { AgentAdapter, AgentRequest, NarrationBeat, NarrationStream } from '../../../contracts/index'
import type { UiState } from '../../core/model'
import type { SceneRendererPort } from '../../core/ports'

export class UnavailableAgentAdapter implements AgentAdapter {
  readonly available = false

  async ask(_request: AgentRequest): Promise<NarrationStream> {
    throw new Error('no agent adapter is configured')
  }
}

export class ScriptedAgentAdapter implements AgentAdapter {
  readonly available = true

  constructor(
    private readonly script: readonly NarrationBeat[],
    private readonly onInterrupt?: () => void,
  ) {}

  async ask(_request: AgentRequest): Promise<NarrationStream> {
    const script = this.script
    let interrupted = false
    const self = this
    async function* beats(): AsyncIterable<NarrationBeat> {
      for (const beat of script) {
        if (interrupted) return
        yield beat
      }
    }
    return {
      beats: beats(),
      interrupt() {
        interrupted = true
        self.onInterrupt?.()
      },
    }
  }
}

/** A scene renderer that does nothing but remember what it saw — the no-op seam the tooth uses. */
export class RecordingSceneAdapter implements SceneRendererPort {
  private readonly seen: UiState[] = []
  private unsub: (() => void) | null = null

  get rendered(): readonly UiState[] {
    return this.seen
  }

  mount(state: UiState, observe: (listener: (state: UiState) => void) => () => void): void {
    this.seen.push(state)
    this.unsub = observe((next) => this.seen.push(next))
  }

  dispose(): void {
    this.unsub?.()
    this.unsub = null
  }
}
