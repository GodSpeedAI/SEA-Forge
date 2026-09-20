/**
 * Fixture world adapter (T03).
 *
 * Serves the fixture revisions as world snapshots and bounded artifact disclosures. This is the local
 * provider REQ-ARCH-003 requires: the core runs against it with no SEA Forge, GitHub, Gauntlet,
 * CopilotKit, or transport present. The adapter keeps all fixture knowledge here; the core sees only
 * the contract ports.
 */
import type {
  ArtifactAdapter,
  ArtifactRef,
  DisclosureLevel,
  TemporalAdapter,
  TemporalPosition,
  TemporalWindow,
  WorldAdapter,
  WorldSnapshot,
} from '../../../contracts/index'
import type { CognitiveArtifactDescriptor } from '../../core/model'
import type { ArtifactCatalogPort } from '../../core/ports'
import type { FixtureArtifact, FixtureWorldRevision } from './worlds'

export class FixtureWorldAdapter implements WorldAdapter {
  private readonly listeners = new Set<(snapshot: WorldSnapshot) => void>()

  constructor(private readonly revisions: readonly FixtureWorldRevision[]) {}

  /** The present is the last revision; history is reachable only through the temporal window. */
  async snapshot(): Promise<WorldSnapshot> {
    const rev = this.revisions[this.revisions.length - 1]
    if (!rev) throw new Error('fixture world has no revisions')
    return this.toSnapshot(rev)
  }

  subscribe(listener: (snapshot: WorldSnapshot) => void): () => void {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  /** Test/fixture hook: advance to a revision and notify subscribers. */
  emitRevision(index: number): void {
    const rev = this.revisions[index]
    if (!rev) return
    const snapshot = this.toSnapshot(rev)
    for (const listener of this.listeners) listener(snapshot)
  }

  private toSnapshot(rev: FixtureWorldRevision): WorldSnapshot {
    return {
      cursor: rev.cursor,
      surfaces: rev.surfaces,
      objects: rev.objects,
      relationships: rev.relationships,
    }
  }
}

export class FixtureTemporalAdapter implements TemporalAdapter {
  constructor(private readonly revisions: readonly FixtureWorldRevision[]) {}

  async window(_since: string | null, limit: number): Promise<TemporalWindow> {
    const positions: TemporalPosition[] = this.revisions.slice(-limit).map((rev) => ({
      cursor: rev.cursor,
      at: rev.at,
      summary: rev.summary,
    }))
    return { positions, truncated: false }
  }

  async live(): Promise<TemporalPosition> {
    const rev = this.revisions[this.revisions.length - 1]
    if (!rev) throw new Error('fixture world has no revisions')
    return { cursor: rev.cursor, at: rev.at, summary: rev.summary }
  }
}

export class FixtureArtifactAdapter implements ArtifactAdapter {
  constructor(private readonly artifacts: readonly FixtureArtifact[]) {}

  async resolve(ref: string, level: DisclosureLevel): Promise<ArtifactRef> {
    const artifact = this.artifacts.find((a) => a.ref === ref)
    if (!artifact) throw new Error(`fixture artifact ${ref} not found`)
    const available = artifact.levels.find((l) => l.level === level)
    if (!available) {
      throw new Error(`fixture artifact ${ref} does not disclose level ${level}`)
    }
    return {
      ref: artifact.ref,
      kind: artifact.kind,
      mediaType: available.mediaType,
      byteLength: available.text.length,
    }
  }

  /** Reads the payload at the deepest disclosed level — the fixture's stand-in for source bytes. */
  async read(ref: string): Promise<Uint8Array> {
    const artifact = this.artifacts.find((a) => a.ref === ref)
    if (!artifact) throw new Error(`fixture artifact ${ref} not found`)
    const deepest = artifact.levels[artifact.levels.length - 1]
    if (!deepest) throw new Error(`fixture artifact ${ref} has no levels`)
    return new TextEncoder().encode(deepest.text)
  }
}

export class FixtureArtifactCatalog implements ArtifactCatalogPort {
  constructor(private readonly artifacts: readonly FixtureArtifact[]) {}

  async descriptors(): Promise<readonly CognitiveArtifactDescriptor[]> {
    return this.artifacts.map((a) => ({
      ref: a.ref,
      kind: a.kind,
      title: a.title,
      boundObject: a.boundObject,
    }))
  }
}

/**
 * The fixture interaction adapter is honest about what a fixture is: it refuses every consequential
 * intent as `unavailable`. Nothing consequential is ever concluded locally, even in a demo.
 */
export class FixtureInteractionAdapter {
  async dispatch(intent: { kind: string; target?: string }): Promise<{
    status: 'refused'
    reason: 'unavailable'
  }> {
    void intent
    return { status: 'refused', reason: 'unavailable' }
  }
}
