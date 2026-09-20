/**
 * UI-core representations (T03).
 *
 * Everything the interaction model needs is declared here in application-owned terms. The world,
 * temporal, artifact and agent shapes are the contract types T01 froze (`contracts/`); the core adds
 * only its OWN state vocabulary on top. Nothing in this package may import a renderer, a transport,
 * or a backend: `purity.test.ts` enforces that mechanically as part of GATE_UI.
 *
 * REQ-GOAL-002, REQ-GOAL-003, REQ-OUT-002, REQ-ARCH-003, REQ-ARCH-004.
 */
import type { DisclosureLevel, IntentRefusal } from '../../contracts/index'
import type {
  NarrationBeat,
  ObjectId,
  TemporalPosition,
  WorldSnapshot,
} from '../../contracts/index'

export type { DisclosureLevel, IntentRefusal, NarrationBeat, ObjectId, TemporalPosition, WorldSnapshot }

/**
 * A bounded cognitive artifact as the core sees it: a descriptor that binds an adapter-resolved
 * artifact to the object whose disclosure reaches it. The core never holds artifact bytes; it holds
 * the descriptor and the level the interaction has reached.
 */
export interface CognitiveArtifactDescriptor {
  readonly ref: string
  readonly kind: string
  readonly title: string
  /** The object whose disclosure exposes this artifact. */
  readonly boundObject: ObjectId
}

/** Where the environment stands on the time line. `live` means "now"; anything else is a position. */
export interface TimeState {
  readonly positions: readonly TemporalPosition[]
  /** Index into `positions`; null only when no window has been loaded yet. */
  readonly index: number | null
  readonly live: boolean
  /** True when the window is a truncated view and older positions exist beyond it. */
  readonly truncated: boolean
}

/** Disclosure state for one object's bound artifact. */
export interface ArtifactDisclosure {
  readonly objectId: ObjectId
  readonly level: DisclosureLevel
  readonly title?: string
  /** What the interaction can read at the current level (adapter-supplied, never core-authored). */
  readonly note?: string
  readonly pending: boolean
}

export interface NarrationState {
  /** False when no agent adapter is configured; the environment degrades, it does not block. */
  readonly available: boolean
  readonly beats: readonly NarrationBeat[]
  readonly interrupted: boolean
  readonly note?: string
}

export interface RefusalRecord {
  readonly reason: IntentRefusal
  readonly note?: string
  readonly at: number
}

/**
 * The whole of the environment's local state. It is a projection and interaction record only: no
 * case, repository, execution, judgment or settlement truth lives here (REQ-OUT-002).
 */
export interface UiState {
  readonly world: WorldSnapshot | null
  readonly surfaceId: string | null
  readonly focusId: ObjectId | null
  readonly selectedId: ObjectId | null
  readonly time: TimeState
  /** Artifact descriptors joined to the world by object id, supplied by the catalog port. */
  readonly artifactCatalog: Readonly<Record<ObjectId, readonly CognitiveArtifactDescriptor[]>>
  readonly disclosures: Readonly<Record<ObjectId, ArtifactDisclosure>>
  readonly narration: NarrationState
  readonly lastRefusal: RefusalRecord | null
}

export const initialState: UiState = {
  world: null,
  surfaceId: null,
  focusId: null,
  selectedId: null,
  time: { positions: [], index: null, live: true, truncated: false },
  artifactCatalog: {},
  disclosures: {},
  narration: { available: false, beats: [], interrupted: false },
  lastRefusal: null,
}
