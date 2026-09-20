/**
 * T03 interaction scenario — the plan's teeth, executed as tests.
 *
 * Tooth 1 (adapter replacement): the environment runs with the scene renderer replaced by a
 * recording no-op adapter and the agent replaced by an unavailable adapter; every assertion below is
 * about core state, never about rendering.
 *
 * Tooth 2 (provider swap): `runInteractionScenario` is ONE shared scenario function executed against
 * two structurally different providers (the fixture world and the adjacency-map alt provider). The
 * core interaction grammar is identical for both, so nothing fixture-specific leaked into the core.
 */
import { describe, expect, test } from 'bun:test'

import type { EnvironmentAdapters } from './engine'
import { CognitiveEnvironment } from './engine'
import {
  AltArtifactProvider,
  AltCatalogProvider,
  AltTemporalProvider,
  AltWorldProvider,
} from '../adapters/fixture/alt-provider'
import {
  FixtureArtifactAdapter,
  FixtureArtifactCatalog,
  FixtureInteractionAdapter,
  FixtureTemporalAdapter,
  FixtureWorldAdapter,
} from '../adapters/fixture/fixture-provider'
import { harborArtifacts, harborDredgingHistory } from '../adapters/fixture/worlds'
import { RecordingSceneAdapter, UnavailableAgentAdapter } from '../adapters/test/test-adapters'

function fixtureAdapters(scene?: EnvironmentAdapters['scene']): EnvironmentAdapters {
  const revisions = harborDredgingHistory
  return {
    world: new FixtureWorldAdapter(revisions),
    temporal: new FixtureTemporalAdapter(revisions),
    artifact: new FixtureArtifactAdapter(harborArtifacts),
    catalog: new FixtureArtifactCatalog(harborArtifacts),
    agent: new UnavailableAgentAdapter(),
    interaction: new FixtureInteractionAdapter(),
    scene,
  }
}

/** A structurally different provider: adjacency map, lazy lenses, different kinds and id shapes. */
function altAdapters(scene?: EnvironmentAdapters['scene']): EnvironmentAdapters {
  const nodes = [
    { key: 'n/programme', nodeKind: 'programme', name: 'Wetland restoration programme', weight: 1, cell: { x: 0, y: 0, depth: 0 } },
    { key: 'n/plot-7', nodeKind: 'site', name: 'Plot 7 re-wetting', weight: 0.85, cell: { x: 2, y: 1, depth: 1 } },
    { key: 'n/plot-12', nodeKind: 'site', name: 'Plot 12 breach', weight: 0.6, cell: { x: 2, y: 2, depth: 2 } },
    { key: 'n/well-3', nodeKind: 'sensor', name: 'Piezometer W-3', weight: 0.75, cell: { x: 4, y: 1, depth: 2 } },
    { key: 'n/consent', nodeKind: 'approval', name: 'Consent to vary', weight: 0.5, cell: { x: 4, y: 3, depth: 1 } },
  ]
  const edges = [
    { a: 'n/programme', b: 'n/plot-7', label: 'comprises' },
    { a: 'n/programme', b: 'n/plot-12', label: 'comprises' },
    { a: 'n/well-3', b: 'n/plot-7', label: 'measures' },
    { a: 'n/consent', b: 'n/programme', label: 'conditions' },
  ]
  const lenses = [
    { id: 'lens/all', name: 'Everything', members: nodes.map((n) => n.key) },
    { id: 'lens/sites', name: 'Sites', members: ['n/plot-7', 'n/plot-12'] },
  ]
  const past = [
    { cursor: 12, at: '2026-08-01T00:00:00Z', summary: 'programme sketched' },
    { cursor: 30, at: '2026-08-20T00:00:00Z', summary: 'plots identified' },
  ]
  return {
    world: new AltWorldProvider(nodes, edges, lenses, 44),
    temporal: new AltTemporalProvider(44, '2026-09-12T00:00:00Z', past),
    artifact: new AltArtifactProvider([
      {
        node: 'n/plot-7',
        title: 'Plot 7 re-wetting record',
        text: {
          minimal: 'Record for plot 7.',
          summary: 'Cell breached in August; water table rising.',
          source: 'PLOT 7 — breach completed 2026-08-21; water table −0.3 m and rising 4 cm/day; two photo stations active.',
        },
      },
      {
        node: 'n/well-3',
        title: 'Piezometer W-3 calibration',
        text: {
          minimal: 'Sensor record.',
          summary: 'Calibrated 2026-09-10; drift within tolerance.',
          source: 'W-3 — two-point calibration 2026-09-10; drift 0.8 cm over 14 days (tolerance 2 cm); next service 2026-10-10.',
        },
      },
    ]),
    catalog: new AltCatalogProvider([
      { node: 'n/plot-7', title: 'Plot 7 re-wetting record' },
      { node: 'n/well-3', title: 'Piezometer W-3 calibration' },
    ]),
    agent: new UnavailableAgentAdapter(),
    interaction: { async dispatch() { return { status: 'refused' as const, reason: 'unavailable' as const } } },
    scene,
  }
}

const FIXTURE_ONLY_FIRST_OBJECT = 'obj-case' // used ONLY by fixture-specific tooth-1 tests below

/**
 * THE shared scenario. One function, two providers, ZERO hardcoded ids: every id is derived from the
 * state the environment itself loaded, so nothing fixture-specific can hide in the grammar. Focus →
 * move → select → surface policy → disclosure cycle → time movement → return to now → refused
 * consequential intent → unavailable narration.
 */
async function runInteractionScenario(make: (scene?: EnvironmentAdapters['scene']) => EnvironmentAdapters): Promise<void> {
  const scene = new RecordingSceneAdapter()
  const env = new CognitiveEnvironment(make(scene), () => 1_000)
  await env.start()

  // Start: a snapshot is loaded, the first surface is current, the scene seam saw the first state.
  const started = env.store.get()
  const world = started.world!
  const firstSurface = world.surfaces[0]!
  const otherSurface = world.surfaces.find((s) => s.id !== firstSurface.id)!
  const firstObject = firstSurface.objectIds[0]!
  const catalogued = Object.keys(started.artifactCatalog)
  const artifactObject =
    world.objects.find((o) => catalogued.includes(o.id))!.id
  const artifactOnFirstSurface = firstSurface.objectIds.find((id) => catalogued.includes(id))
  const uncataloguedObject =
    firstSurface.objectIds.find((id) => !catalogued.includes(id)) ?? null
  expect(started.surfaceId).toBe(firstSurface.id)
  expect(started.time.live).toBe(true)
  expect(started.time.positions.length).toBeGreaterThanOrEqual(2)
  expect(scene.rendered.length).toBe(1)

  // Focus + move within the surface's ordered objects.
  env.focus(firstObject)
  expect(env.store.get().focusId).toBe(firstObject)
  env.moveFocus('previous') // at the first object already; must not move
  expect(env.store.get().focusId).toBe(firstObject)
  env.moveFocus('next')
  const afterMove = env.store.get().focusId
  expect(afterMove).not.toBe(firstObject)
  env.moveFocus('previous')
  expect(env.store.get().focusId).toBe(firstObject)

  // Focus by direct id works across surfaces (moveFocus is the surface-scoped operation).
  const foreignObject = otherSurface.objectIds.find((id) => id !== firstObject)!
  env.focus(foreignObject)
  expect(env.store.get().focusId).toBe(foreignObject)
  env.focus(firstObject)

  // Select, then surface switch clears focus and selection (documented per-surface policy).
  env.select(firstObject)
  expect(env.store.get().selectedId).toBe(firstObject)
  env.setSurface(otherSurface.id)
  expect(env.store.get().surfaceId).toBe(otherSurface.id)
  expect(env.store.get().focusId).toBeNull()
  expect(env.store.get().selectedId).toBeNull()
  env.setSurface(firstSurface.id)

  // Disclosure cycles minimal → summary → source, then stays at source.
  const disclosureTarget = artifactOnFirstSurface ?? artifactObject
  env.focus(disclosureTarget)
  const first = await env.resolve(disclosureTarget)
  expect(first?.level).toBe('summary')
  const second = await env.resolve(disclosureTarget)
  expect(second?.level).toBe('source')
  const third = await env.resolve(disclosureTarget)
  expect(third?.level).toBe('source')
  expect(second?.title).toBe(third?.title)

  // An object with no bound artifact reports no disclosure rather than inventing one.
  if (uncataloguedObject) {
    env.focus(uncataloguedObject)
    const none = await env.resolve(uncataloguedObject)
    expect(none).toBeNull()
  }

  // Time: step into history, then return to now.
  env.stepTime(-1)
  expect(env.store.get().time.live).toBe(false)
  expect(env.store.get().time.index).toBe(env.store.get().time.positions.length - 2)
  env.returnToNow()
  expect(env.store.get().time.live).toBe(true)
  expect(env.store.get().time.index).toBeNull()

  // Consequential intent: refused by the adapter, recorded in state, never concluded locally.
  await env.propose('propose-consequence', firstObject)
  expect(env.store.get().lastRefusal?.reason).toBe('unavailable')

  // Narration without an agent degrades honestly.
  await env.requestExplanation('what changed?')
  const narrated = env.store.get().narration
  expect(narrated.available).toBe(false)
  expect(narrated.beats.length).toBe(0)
  expect(narrated.note).toContain('unavailable')

  env.dispose()
}

describe('tooth 2 — the same interaction grammar under two structurally different providers', () => {
  test('fixture world provider', async () => {
    await runInteractionScenario(fixtureAdapters)
  })

  test('alt (adjacency-map) provider', async () => {
    await runInteractionScenario(altAdapters)
  })
})

describe('tooth 1 — the core runs with scene and agent replaced by no-op/test adapters', () => {
  test('recording scene adapter observes state transitions driven purely by actions', async () => {
    const scene = new RecordingSceneAdapter()
    const env = new CognitiveEnvironment(fixtureAdapters(scene), () => 1_000)
    await env.start()
    env.focus(FIXTURE_ONLY_FIRST_OBJECT)
    env.setSurface('surface-evidence')
    const states = scene.rendered
    expect(states.length).toBe(3)
    expect(states[0]!.focusId).toBeNull()
    expect(states[1]!.focusId).toBe('obj-case')
    expect(states[2]!.surfaceId).toBe('surface-evidence')
    env.dispose()
  })

  test('world-adapter push updates the store without any action', async () => {
    const adapters = fixtureAdapters()
    const env = new CognitiveEnvironment(adapters, () => 1_000)
    await env.start()
    const before = env.store.get().world!.cursor
    const adapter = adapters.world as FixtureWorldAdapter
    adapter.emitRevision(0)
    const after = env.store.get().world!.cursor
    expect(after).not.toBe(before)
    env.dispose()
  })
})
