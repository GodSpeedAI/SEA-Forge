// Artifacts surface: the existing artifacts index, composed spatially.

import { ArtifactsPage } from "../../pages/SurfacesPages";
import { Surface } from "../../spatial/primitives";

export function ArtifactsSurface() {
  return (
    <Surface id="artifacts" title="Artifacts">
      <ArtifactsPage />
    </Surface>
  );
}
