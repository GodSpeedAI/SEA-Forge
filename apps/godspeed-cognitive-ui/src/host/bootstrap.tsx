/**
 * Host bootstrap (T03): compose the environment from the FIXTURE provider set.
 *
 * This file is the only place that decides which adapters the demo runs with. The core is composed
 * identically in tests; swapping this provider set for the real Go-transport set (a later task) must
 * not touch the interaction grammar.
 */
import { CognitiveEnvironment } from '../core/engine'
import {
  FixtureArtifactAdapter,
  FixtureArtifactCatalog,
  FixtureInteractionAdapter,
  FixtureTemporalAdapter,
  FixtureWorldAdapter,
} from '../adapters/fixture/fixture-provider'
import { harborArtifacts, harborDredgingHistory } from '../adapters/fixture/worlds'
import { UnavailableAgentAdapter } from '../adapters/test/test-adapters'

export function buildFixtureEnvironment(): CognitiveEnvironment {
  const revisions = harborDredgingHistory
  return new CognitiveEnvironment({
    world: new FixtureWorldAdapter(revisions),
    temporal: new FixtureTemporalAdapter(revisions),
    artifact: new FixtureArtifactAdapter(harborArtifacts),
    catalog: new FixtureArtifactCatalog(harborArtifacts),
    agent: new UnavailableAgentAdapter(),
    interaction: new FixtureInteractionAdapter(),
  })
}
