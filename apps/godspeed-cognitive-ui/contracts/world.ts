/**
 * World adapter contract.
 *
 * The cognitive environment renders and manipulates a world it does not own. This file declares the
 * shape of a world snapshot in the application's own terms; a transport adapter later maps the Go
 * application's payloads onto these types, so no transport, rendering or provider type appears here.
 *
 * REQ-ARCH-003/REQ-ARCH-004 (T03 owns the implementation, T01 owns the contract shape).
 */

export type ObjectId = string

export interface WorldObject {
  readonly id: ObjectId
  /** Stable, application-owned kind used by the grammar (surface/object/relationship). */
  readonly kind: string
  readonly label: string
  /** Position in the scene's own coordinate space; the renderer maps this, not the other way round. */
  readonly position: { x: number; y: number; depth: number }
  /** Relevance in [0,1] as computed by the projection, never by the renderer. */
  readonly salience: number
}

export interface WorldRelationship {
  readonly from: ObjectId
  readonly to: ObjectId
  readonly kind: string
}

export interface WorldSurface {
  readonly id: string
  readonly label: string
  readonly objectIds: readonly ObjectId[]
}

/**
 * A world snapshot is a projection: it carries no authority and no history. Anything consequential
 * travels as an interaction intent instead.
 */
export interface WorldSnapshot {
  readonly cursor: number
  readonly surfaces: readonly WorldSurface[]
  readonly objects: readonly WorldObject[]
  readonly relationships: readonly WorldRelationship[]
}

export interface WorldAdapter {
  snapshot(): Promise<WorldSnapshot>
  subscribe(listener: (snapshot: WorldSnapshot) => void): () => void
}
