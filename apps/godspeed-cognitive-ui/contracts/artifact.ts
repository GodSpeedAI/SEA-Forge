/**
 * Artifact contract.
 *
 * Artifacts are bounded and progressive: a minimal representation first, deeper source material only
 * when the interaction reaches it. Renderers are resolved through an application-owned registry so a
 * heavy renderer stays unloaded until it is reached.
 */

export interface ArtifactRef {
  readonly ref: string
  readonly kind: string
  readonly mediaType: string
  readonly byteLength: number
}

export type DisclosureLevel = 'minimal' | 'summary' | 'source'

export interface ArtifactAdapter {
  resolve(ref: string, level: DisclosureLevel): Promise<ArtifactRef>
  /** Reads the payload. Only called once the interaction reaches the required depth. */
  read(ref: string): Promise<Uint8Array>
}

export interface ArtifactRendererDescriptor {
  readonly kind: string
  readonly mediaTypes: readonly string[]
  /** Dynamic import by contract: the environment never statically pulls a heavy renderer. */
  readonly load: () => Promise<{ render: (target: unknown, data: Uint8Array) => void }>
}

export interface ArtifactRegistry {
  register(descriptor: ArtifactRendererDescriptor): void
  forKind(kind: string): ArtifactRendererDescriptor | undefined
}
