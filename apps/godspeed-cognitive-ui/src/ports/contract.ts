// The UI's contract boundary. Everything the world, history, artifacts, authority, execution and
// settlement bring into the UI crosses this port in the shapes defined by the interface-contract
// package (.agents/reports/interface-contracts). Replacing the local adapter with the Go adapter
// must not change anything above this file.
//
// Base shapes come verbatim from typescript/types.ts. Shapes that spec 04 defines but types.ts does
// not yet carry (relationships, artifact descriptors, narration) are declared here from spec 04.
// Proposed extensions are optional, namespaced under `x`, and listed in
// .agents/reports/godspeed-cognitive-ui-functional/03-CONTRACT-PORTS-AND-SEAMS.md.

import type {
  ActionDescriptor,
  ActorRole,
  ArtifactPayload,
  CognitiveObject,
  CognitiveWorldSnapshot,
  ExecutionProgressPayload,
  IntentResponse,
  InteractionIntent,
  OperationalSettlement,
  StreamEvent,
  TemporalTrajectoryResponse,
} from '../../../../.agents/reports/interface-contracts/typescript/types'

export type {
  ActionDescriptor,
  ActorRole,
  ArtifactPayload,
  CognitiveObject,
  CognitiveWorldSnapshot,
  ExecutionProgressPayload,
  IntentResponse,
  InteractionIntent,
  OperationalSettlement,
  StreamEvent,
  TemporalTrajectoryResponse,
}
export type {
  ActionIntentKind,
  CognitiveObjectKind,
  CognitiveObjectStatus,
  InteractionActionName,
  TemporalCheckpoint,
} from '../../../../.agents/reports/interface-contracts/typescript/types'

// ---------------------------------------------------------------------------
// Spec 04 shapes not yet in types.ts

/** Spec 04 §3. */
export interface CognitiveRelationship {
  readonly from: string
  readonly to: string
  readonly kind: 'depends-on' | 'produces' | 'governed-by' | 'attests' | 'contradicts'
  readonly label?: string
}

/** Spec 04 §7. */
export type DisclosureLevel = 'minimal' | 'summary' | 'source'

/** Spec 04 §7: an artifact bound to an object; its bytes come from `resolveArtifact(ref)`. */
export interface CognitiveArtifact {
  readonly ref: string
  readonly kind: 'code_diff' | 'test_report' | 'decision_record' | 'document' | 'data_table'
  readonly title: string
  readonly boundObject: string
  readonly currentLevel: DisclosureLevel
  readonly mediaType: string
  readonly sourceProvenance: string
}

/** Spec 04 §8. */
export interface NarrationDirective {
  readonly focus?: string | null
  readonly camera?: { readonly target: string | 'core'; readonly zoom?: 'system' | 'local' | 'detail' }
  readonly highlight?: readonly string[]
  readonly annotate?: readonly { readonly target: string; readonly label: string }[]
  readonly openArtifact?: { readonly ref: string; readonly level: DisclosureLevel }
  readonly temporalStep?: -1 | 1
  // Proposed extensions (seam S3): the remaining representation operators of the grammar.
  readonly x?: {
    readonly arrange?: 'orbital' | 'causal' | 'compare'
    /** Move to a specific checkpoint, or back to live. */
    readonly temporalTo?: string | 'live'
    readonly compare?: { readonly a: string; readonly b: string | 'live' }
    readonly reveal?: readonly string[]
    readonly dim?: readonly string[]
  }
}

/** Spec 04 §8. */
export interface NarrationBeat {
  readonly index: number
  readonly thoughtText: string
  readonly evidenceCitations: readonly string[]
  readonly directives?: readonly NarrationDirective[]
}

/** Spec 04 §8. */
export interface AgentNarrationStream {
  readonly beats: AsyncIterable<NarrationBeat>
  interrupt(): void
}

// ---------------------------------------------------------------------------
// Proposed optional extensions (seams S1–S2), all under `x`

export type ToneHint = 'ok' | 'progress' | 'attention' | 'critical' | 'hypothesis' | 'muted'

export interface CausalRepresentation {
  /** Position in the causal reading, 1-based. */
  readonly order: number
  readonly role: 'expected' | 'assumption' | 'condition' | 'missing' | 'observed' | 'dependency' | 'consequence'
  /** Representation-specific label, e.g. "Expected Workflow" for the workflow object. */
  readonly label?: string
  readonly explanation?: string
  readonly badge?: string
  readonly tone?: ToneHint
}

export interface ObjectExtensions {
  /** Presentation class on the system surface (a case world can contain regions, cases, facets, items). */
  readonly presentation?: 'core' | 'region' | 'case' | 'facet' | 'item' | 'person' | 'run'
  readonly metric?: string
  readonly tone?: ToneHint
  /** Present but not yet real at this cursor. */
  readonly ghost?: boolean
  readonly dormant?: boolean
  readonly label_side?: 'right' | 'left' | 'above'
  /** Center residue when this object is the local center, e.g. "1 issue needs review". */
  readonly residue?: { readonly label: string; readonly tone: ToneHint }
  readonly artifacts?: readonly CognitiveArtifact[]
  readonly representations?: { readonly causal?: CausalRepresentation }
  /** Authoritative settlement for the work this object represents (only the backend sets it). */
  readonly settlement?: OperationalSettlement
  /** Case world whose template can be designed from this object. */
  readonly template_case_id?: string
  readonly design?: {
    readonly icon?: 'target' | 'layers' | 'people' | 'doc' | 'flag' | 'shield' | 'bolt' | 'history' | 'eye'
    readonly accent?: 'orange' | 'blue' | 'green' | 'dark'
    /** Ordered items of a design node, e.g. stages with their role. */
    readonly items?: readonly { readonly id: string; readonly label: string; readonly detail?: string; readonly required?: boolean }[]
  }
}

export type XObject = CognitiveObject & { readonly x?: ObjectExtensions }

export type XSnapshot = Omit<CognitiveWorldSnapshot, 'visible_objects'> & {
  readonly visible_objects: readonly XObject[]
  readonly x?: {
    readonly relationships?: readonly (CognitiveRelationship & { readonly id: string })[]
    /** Short label of this position in history, e.g. "Sep 17". */
    readonly label?: string
  }
}

// ---------------------------------------------------------------------------
// The port

/** Contract `CaseworkAdapter` (typescript/mock-adapter.ts), with snapshots widened to carry `x`. */
export interface CaseworkPort {
  getSnapshot(caseId: string, actorId: string, role: ActorRole): Promise<XSnapshot>
  getSnapshotAt(caseId: string, cursor: string, actorId: string, role: ActorRole): Promise<XSnapshot>
  dispatchIntent(intent: InteractionIntent): Promise<IntentResponse>
  resolveArtifact(evidenceId: string): Promise<ArtifactPayload>
  queryTemporalTrajectory(caseId: string): Promise<TemporalTrajectoryResponse>
  subscribeEvents(
    caseId: string,
    sinceCursor: string | undefined,
    onEvent: (event: StreamEvent) => void,
    onError: (err: Error) => void,
  ): () => void
}

/**
 * Template authoring surface (T08, additive): served by the live gateway
 * (GET /api/templates, POST /api/templates/preflight). Feature-detect with
 * `'getTemplates' in port` — the local adapter gains fixture templates in T09.
 */
export interface TemplateSourcePort {
  getTemplates(): Promise<
    readonly {
      template_ref: string
      title: string
      description?: string
      parameters: readonly {
        name: string
        param_type: string
        required: boolean
        default?: string
      }[]
    }[]
  >
  preflightTemplate(
    templateRef: string,
    params: Record<string, unknown>,
  ): Promise<{ passed: boolean; digest?: string; reasons: readonly string[] }>
}

/** The agent side of the environment (spec 04 §8). Optional: the UI must work without it. */
export interface NarrationPort {
  /** Returns null when the agent has nothing to say about this question in this context. */
  explain(question: string, context: { caseId: string; focus: string | null; cursor: string }): AgentNarrationStream | null
}

export interface Actor {
  readonly actor_id: string
  readonly role: ActorRole
  readonly display_name: string
  readonly kind: 'human' | 'agent'
}
