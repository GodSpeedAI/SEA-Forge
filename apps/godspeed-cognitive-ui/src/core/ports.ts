/**
 * Core-owned ports beyond the T01 contract set (T03).
 *
 * The world/temporal/artifact/agent/interaction shapes are frozen in `contracts/`. Two more ports
 * are application-owned because the core itself needs them: a catalog that binds artifact descriptors
 * to objects, and the scene-renderer seam that lets the interaction model run with rendering replaced
 * by a no-op or test adapter (the plan's first T03 tooth).
 */
import type { CognitiveArtifactDescriptor, UiState } from './model'

export interface ArtifactCatalogPort {
  descriptors(): Promise<readonly CognitiveArtifactDescriptor[]>
}

export interface SceneRendererPort {
  /** Called once when the environment is composed; the renderer observes state from there. */
  mount(
    state: UiState,
    observe: (listener: (state: UiState) => void) => () => void,
  ): void
  dispose(): void
}
