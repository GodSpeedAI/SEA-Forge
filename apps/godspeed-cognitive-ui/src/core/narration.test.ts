/**
 * Narration interruption (T03): the agent seam is interruptible by construction, and interruption
 * leaves an honest record (the beats that arrived, the interrupted flag).
 */
import { describe, expect, test } from 'bun:test'

import type { EnvironmentAdapters } from './engine'
import { CognitiveEnvironment } from './engine'
import type { AgentAdapter, AgentRequest, NarrationStream } from '../../contracts/index'
import {
  FixtureArtifactAdapter,
  FixtureArtifactCatalog,
  FixtureInteractionAdapter,
  FixtureTemporalAdapter,
  FixtureWorldAdapter,
} from '../adapters/fixture/fixture-provider'
import { harborArtifacts, harborDredgingHistory } from '../adapters/fixture/worlds'

/** An agent whose beats are slow enough to interrupt deterministically from the test. */
class SlowAgentAdapter implements AgentAdapter {
  readonly available = true

  async ask(_request: AgentRequest): Promise<NarrationStream> {
    let interrupted = false
    async function* beats() {
      for (let i = 0; i < 5; i++) {
        if (interrupted) return
        await new Promise((r) => setTimeout(r, 5))
        yield { index: i, text: `beat ${i}`, evidenceRefs: [] }
      }
    }
    return {
      beats: beats(),
      interrupt() {
        interrupted = true
      },
    }
  }
}

function adaptersWithAgent(agent: AgentAdapter): EnvironmentAdapters {
  return {
    world: new FixtureWorldAdapter(harborDredgingHistory),
    temporal: new FixtureTemporalAdapter(harborDredgingHistory),
    artifact: new FixtureArtifactAdapter(harborArtifacts),
    catalog: new FixtureArtifactCatalog(harborArtifacts),
    agent,
    interaction: new FixtureInteractionAdapter(),
  }
}

describe('narration is interruptible by construction', () => {
  test('interrupt stops the stream and records the interruption', async () => {
    const agent = new SlowAgentAdapter()
    const env = new CognitiveEnvironment(adaptersWithAgent(agent), () => 1_000)
    await env.start()

    const streaming = env.requestExplanation('explain the evidence')
    await new Promise((r) => setTimeout(r, 8)) // let at least one beat arrive
    expect(env.store.get().narration.available).toBe(true)
    expect(env.store.get().narration.beats.length).toBeGreaterThanOrEqual(1)

    env.interruptNarration()
    await streaming
    const after = env.store.get().narration
    expect(after.interrupted).toBe(true)
    const countAtInterrupt = after.beats.length
    await new Promise((r) => setTimeout(r, 25)) // a non-interruptible stream would keep producing
    expect(env.store.get().narration.beats.length).toBe(countAtInterrupt)
    env.dispose()
  })
})
