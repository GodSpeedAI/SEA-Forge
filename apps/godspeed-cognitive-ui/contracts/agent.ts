/**
 * Agent contract.
 *
 * Agents act through the SAME interaction model as humans: they read current UI context and request
 * representational operations through UI-owned actions. Nothing here lets an agent mutate governed
 * case truth directly, and narration is interruptible by construction.
 */
import type { DisclosureLevel } from './artifact'
import type { IntentKind } from './interaction'
import type { ObjectId, WorldRelationship, ZoomLevel } from './world'

export interface UiContextSnapshot {
  readonly focus: string | null
  readonly visibleObjectIds: readonly string[]
  readonly disclosure: 'minimal' | 'summary' | 'source'
  /** Current surface arrangement, when one is established. */
  readonly surfaceId?: string | null
  /** Temporal standing: live now, or standing at a projection cursor. */
  readonly live?: boolean
  readonly temporalCursor?: number | null
  /** Text the user selected inside an open artifact, when a selection exists. */
  readonly selectionText?: string
  readonly openArtifactRefs?: readonly string[]
  readonly pinnedArtifactRefs?: readonly string[]
}

export interface AgentRequest {
  readonly requestId: string
  readonly question: string
  readonly context: UiContextSnapshot
}

/**
 * Agent-framework adapters may request only this renderer-independent vocabulary. Every variant
 * names an application object, relationship, surface, artifact, temporal index, or governed
 * interaction. Scene coordinates deliberately have no representation here.
 */
export type AgentSemanticCommand =
  | { readonly name: 'focus'; readonly objectId: ObjectId }
  | { readonly name: 'back' }
  | { readonly name: 'reveal'; readonly objectIds: readonly ObjectId[] }
  | { readonly name: 'connect'; readonly relationship: WorldRelationship }
  | { readonly name: 'arrange'; readonly surfaceId: string }
  | { readonly name: 'compare'; readonly left: ObjectId; readonly right: ObjectId; readonly label?: string }
  | { readonly name: 'set-temporal'; readonly index: number }
  | { readonly name: 'set-temporal'; readonly leftIndex: number; readonly rightIndex: number }
  | { readonly name: 'return' }
  | { readonly name: 'materialize'; readonly ref: string; readonly level?: DisclosureLevel }
  | { readonly name: 'collapse'; readonly objectIds?: readonly ObjectId[]; readonly artifactRef?: string }
  | {
      readonly name: 'invoke'
      readonly kind: IntentKind
      readonly target?: string
      readonly parameters?: Readonly<Record<string, string>>
    }

/**
 * One choreography directive: a representational operation the narration asks the environment to
 * perform while the thought is spoken. Directives are requests through the same action vocabulary a
 * human uses; they carry no authority and no case mutation.
 */
export interface NarrationDirective {
  readonly focus?: ObjectId | null
  readonly camera?: { readonly target: ObjectId | 'core'; readonly zoom?: ZoomLevel }
  readonly emphasize?: readonly ObjectId[]
  readonly deEmphasize?: readonly ObjectId[]
  readonly reveal?: readonly ObjectId[]
  readonly conceal?: readonly ObjectId[]
  /** Contextual relationships for the current thought only; never permanent graph furniture. */
  readonly relationships?: readonly WorldRelationship[]
  readonly annotate?: readonly { readonly target: ObjectId; readonly label: string }[]
  /** Materialize a bounded artifact at minimal disclosure, anchored to its bound object. */
  readonly artifact?: { readonly ref: string; readonly boundObject: ObjectId }
  readonly comparison?: { readonly left: ObjectId; readonly right: ObjectId; readonly label?: string }
  readonly temporal?: { readonly step?: -1 | 1; readonly now?: boolean }
}

export interface NarrationBeat {
  readonly index: number
  readonly text: string
  /** Artifact refs the beat cites; never a bare claim without evidence. */
  readonly evidenceRefs: readonly string[]
  readonly directives?: readonly NarrationDirective[]
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
