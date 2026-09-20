/**
 * The cognitive environment engine (T03).
 *
 * Composes the adapter set (world, temporal, artifact, catalog, agent, interaction, optional scene
 * renderer) with the store and the action vocabulary. The engine is the whole of the "application":
 * every adapter here is replaceable — the plan's teeth replace the scene renderer and the agent with
 * no-op/test adapters and swap the fixture provider for a structurally different one, and the
 * interaction tests must be unchanged.
 *
 * The engine never talks to a governed backend, an executor, an agent framework, a transport, or a
 * renderer library; adapters that do are composed in from the outside (REQ-ARCH-003).
 */
import type {
  AgentAdapter,
  ArtifactAdapter,
  InteractionAdapter,
  IntentKind,
  ObjectId,
  TemporalAdapter,
  WorldAdapter,
} from '../../contracts/index'
import * as actions from './actions'
import type { ArtifactDisclosure, CognitiveArtifactDescriptor, UiState } from './model'
import { initialState } from './model'
import type { ArtifactCatalogPort, SceneRendererPort } from './ports'
import { CognitiveStore } from './store'

export interface EnvironmentAdapters {
  world: WorldAdapter
  temporal: TemporalAdapter
  artifact: ArtifactAdapter
  catalog: ArtifactCatalogPort
  agent: AgentAdapter
  interaction: InteractionAdapter
  /** Optional: the scene seam. Absent is normal in tests; present is normal in a host. */
  scene?: SceneRendererPort
}

/** Number of temporal positions requested for the initial window. */
const TIME_WINDOW_LIMIT = 12

export class CognitiveEnvironment {
  readonly store: CognitiveStore
  private readonly adapters: EnvironmentAdapters
  private readonly now: () => number
  private unsubscribeWorld: (() => void) | null = null
  private unsubscribeStore: (() => void) | null = null
  private interruptCurrent: (() => void) | null = null

  constructor(adapters: EnvironmentAdapters, now: () => number = Date.now) {
    this.adapters = adapters
    this.now = now
    this.store = new CognitiveStore({ ...initialState, narration: { available: adapters.agent.available, beats: [], interrupted: false } })
  }

  /** Load the first snapshot, the temporal window, and the artifact catalog. */
  async start(): Promise<void> {
    const [snapshot, window, descriptors] = await Promise.all([
      this.adapters.world.snapshot(),
      this.adapters.temporal.window(null, TIME_WINDOW_LIMIT),
      this.adapters.catalog.descriptors(),
    ])

    const catalog: Record<ObjectId, readonly CognitiveArtifactDescriptor[]> = {}
    for (const d of descriptors) {
      catalog[d.boundObject] = [...(catalog[d.boundObject] ?? []), d]
    }

    this.store.update((state) => ({
      ...state,
      world: snapshot,
      surfaceId: snapshot.surfaces[0]?.id ?? null,
      time: {
        positions: window.positions,
        index: null,
        live: true,
        truncated: window.truncated,
      },
      artifactCatalog: catalog,
      narration: { ...state.narration, available: this.adapters.agent.available },
    }))

    this.unsubscribeWorld = this.adapters.world.subscribe((next) => {
      this.store.update((state) => ({ ...state, world: next }))
    })

    if (this.adapters.scene) {
      const scene = this.adapters.scene
      scene.mount(this.store.get(), (listener) => this.store.subscribe(listener))
    }
  }

  dispose(): void {
    this.unsubscribeWorld?.()
    this.unsubscribeStore?.()
    this.unsubscribeWorld = null
    this.unsubscribeStore = null
    this.adapters.scene?.dispose()
  }

  // --- interaction vocabulary: the ONLY way in, for pointer, keyboard, agent, and tests alike. ---

  focus(id: ObjectId | null): void {
    actions.focus(this.store, id)
  }

  moveFocus(direction: 'next' | 'previous'): void {
    actions.moveFocus(this.store, direction)
  }

  setSurface(surfaceId: string): void {
    actions.setSurface(this.store, surfaceId)
  }

  select(id: ObjectId | null): void {
    actions.select(this.store, id)
  }

  /** Cycle the object's disclosure one level deeper; resolves content through the artifact adapter. */
  async resolve(id: ObjectId): Promise<ArtifactDisclosure | null> {
    return actions.resolve(this.store, id, this.adapters.artifact, this.store.get().artifactCatalog)
  }

  /**
   * Consequential intent: crosses the interaction boundary. A fixture or unconfigured adapter
   * refuses; the refusal is recorded in state, never concluded locally (REQ-GOAL-006 later routes
   * this to the governed Go boundary).
   */
  async propose(kind: IntentKind, target?: string, parameters?: Readonly<Record<string, string>>): Promise<void> {
    const interaction = this.adapters.interaction
    await actions.propose(
      this.store,
      { kind, target, parameters },
      (intent) => interaction.dispatch(intent),
      this.now,
    )
  }

  stepTime(delta: 1 | -1): void {
    actions.stepTime(this.store, delta)
  }

  setTimePosition(index: number): void {
    actions.setTimePosition(this.store, index)
  }

  returnToNow(): void {
    actions.returnToNow(this.store)
  }

  clearSelection(): void {
    actions.clearSelection(this.store)
  }

  /**
   * Ask the agent adapter for narration over the current context. Unavailable degrades honestly;
   * interruption is part of the flow, not an afterthought.
   */
  async requestExplanation(question: string): Promise<void> {
    const agent = this.adapters.agent
    if (!agent.available) {
      this.store.update((state) => ({
        ...state,
        narration: {
          available: false,
          beats: [],
          interrupted: false,
          note: 'no agent adapter is configured; narration is unavailable, not failed',
        },
      }))
      return
    }
    const state = this.store.get()
    const visible = actions.surfaceObjects(state.world, state.surfaceId)
    const stream = await agent.ask({
      requestId: `ask-${this.now()}`,
      question,
      context: {
        focus: state.focusId,
        visibleObjectIds: [...visible],
        disclosure: this.deepestDisclosure(state),
      },
    })
    this.interruptCurrent = () => stream.interrupt()
    this.store.update((s) => ({ ...s, narration: { available: true, beats: [], interrupted: false } }))
    try {
      for await (const beat of stream.beats) {
        if (this.interruptCurrent === null) break
        actions.appendBeat(this.store, beat)
      }
    } catch {
      // A beat source that throws ends the stream; the state records what arrived. Interruption is
      // handled by the interrupt() call, not by an exception.
    }
  }

  interruptNarration(): void {
    const interrupt = this.interruptCurrent
    this.interruptCurrent = null
    if (interrupt) {
      interrupt()
      actions.markInterrupted(this.store)
    } else {
      actions.markInterrupted(this.store)
    }
  }

  private deepestDisclosure(state: UiState): 'minimal' | 'summary' | 'source' {
    const levels = Object.values(state.disclosures).map((d) => d.level)
    if (levels.includes('source')) return 'source'
    if (levels.includes('summary')) return 'summary'
    return 'minimal'
  }
}
