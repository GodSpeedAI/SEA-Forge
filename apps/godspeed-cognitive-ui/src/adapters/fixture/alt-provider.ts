/**
 * A second world provider, deliberately structured differently from the fixture (T03 tooth 2).
 *
 * Where the fixture stores flat revision records, this provider keeps an adjacency map and derives
 * surfaces lazily; its worlds have different surface counts, kinds, id conventions, salience
 * distribution, and disclosure content. It implements the SAME contract ports. The point is exact:
 * the core interaction tests run unchanged against this provider, which proves the interaction
 * grammar does not encode fixture structure.
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

interface AltNode {
  readonly key: string
  readonly nodeKind: string
  readonly name: string
  readonly weight: number
  readonly cell: { x: number; y: number; depth: number }
}

interface AltDisclosable {
  readonly node: string
  readonly title: string
  readonly text: Record<DisclosureLevel, string>
}

export class AltWorldProvider implements WorldAdapter {
  private readonly listeners = new Set<(snapshot: WorldSnapshot) => void>()

  constructor(
    private readonly nodes: readonly AltNode[],
    private readonly edges: readonly { a: string; b: string; label: string }[],
    private readonly lenses: readonly { id: string; name: string; members: readonly string[] }[],
    private readonly headCursor: number,
  ) {}

  async snapshot(): Promise<WorldSnapshot> {
    return this.view()
  }

  subscribe(listener: (snapshot: WorldSnapshot) => void): () => void {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  /** Test hook: notify subscribers with a fresh view (the alt provider's "advance" shape). */
  notify(): void {
    const snapshot = this.view()
    for (const listener of this.listeners) listener(snapshot)
  }

  private view(): WorldSnapshot {
    const surfaces = this.lenses.map((lens) => ({
      id: lens.id,
      label: lens.name,
      objectIds: lens.members,
    }))
    return {
      cursor: this.headCursor,
      surfaces,
      objects: this.nodes.map((n) => ({
        id: n.key,
        kind: n.nodeKind,
        label: n.name,
        position: n.cell,
        salience: n.weight,
      })),
      relationships: this.edges.map((e) => ({ from: e.a, to: e.b, kind: e.label })),
    }
  }
}

export class AltTemporalProvider implements TemporalAdapter {
  constructor(
    private readonly headCursor: number,
    private readonly headAt: string,
    private readonly past: readonly { cursor: number; at: string; summary: string }[],
  ) {}

  async window(_since: string | null, limit: number): Promise<TemporalWindow> {
    const all: TemporalPosition[] = [
      ...this.past.map((p) => ({ cursor: p.cursor, at: p.at, summary: p.summary })),
      { cursor: this.headCursor, at: this.headAt, summary: 'head' },
    ]
    return { positions: all.slice(-limit), truncated: all.length > limit }
  }

  async live(): Promise<TemporalPosition> {
    return { cursor: this.headCursor, at: this.headAt, summary: 'head' }
  }
}

export class AltArtifactProvider implements ArtifactAdapter {
  constructor(private readonly disclosables: readonly AltDisclosable[]) {}

  async resolve(ref: string, level: DisclosureLevel): Promise<ArtifactRef> {
    const d = this.disclosables.find((x) => x.node === ref)
    if (!d) throw new Error(`alt artifact ${ref} missing`)
    const text = d.text[level]
    return { ref: d.node, kind: 'alt-record', mediaType: 'text/plain', byteLength: text.length }
  }

  async read(ref: string): Promise<Uint8Array> {
    const d = this.disclosables.find((x) => x.node === ref)
    if (!d) throw new Error(`alt artifact ${ref} missing`)
    return new TextEncoder().encode(d.text.source)
  }
}

export class AltCatalogProvider implements ArtifactCatalogPort {
  constructor(private readonly entries: readonly { node: string; title: string }[]) {}

  async descriptors(): Promise<readonly CognitiveArtifactDescriptor[]> {
    return this.entries.map((d) => ({
      ref: d.node,
      kind: 'alt-record',
      title: d.title,
      boundObject: d.node,
    }))
  }
}
