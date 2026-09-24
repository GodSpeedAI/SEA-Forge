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

/**
 * Representational resolution bands. The renderer never decides what is visible from geometry alone:
 * the core maps (focus, zoom) to the set of objects in the current representation, so zooming changes
 * WHAT is represented, not merely how large it is drawn.
 */
export type ZoomLevel = 'system' | 'local' | 'detail'

export interface WorldObject {
  readonly id: ObjectId
  /** Stable, application-owned kind used by the grammar (surface/object/relationship). */
  readonly kind: string
  readonly label: string
  /** Position in the scene's own coordinate space; the renderer maps this, not the other way round. */
  readonly position: { x: number; y: number; depth: number }
  /** Relevance in [0,1] as computed by the projection, never by the renderer. */
  readonly salience: number
  /**
   * Optional parent: an object with a parent enters the representation only when the parent (or one
   * of its ancestors) is focused and the zoom resolves deeper. This is the semantic-zoom seam.
   */
  readonly parentId?: ObjectId
  /** Ordinary-language state line supplied by the projection, never backend vocabulary. */
  readonly note?: string
  /** Projection attention hint. Absent means quiet; the renderer must not invent importance. */
  readonly attention?: 'notable' | 'requires-judgment'
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
  /** Honest labeling of who produced this projection, e.g. "fixture:harbour" or "go:casework". */
  readonly provenance?: string
}

export interface WorldAdapter {
  snapshot(): Promise<WorldSnapshot>
  subscribe(listener: (snapshot: WorldSnapshot) => void): () => void
  /**
   * Optional: the world as it stood at a temporal cursor. Providers that cannot project history
   * omit this method; the environment then records an honest degradation instead of presenting the
   * present as the past.
   */
  snapshotAt?(cursor: number): Promise<WorldSnapshot>
}
