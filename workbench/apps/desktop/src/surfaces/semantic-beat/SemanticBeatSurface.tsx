// Semantic-beat surface: the existing Thoth workspace, composed spatially.
// Thoth interaction stays behind `ThothInteractionPort`; this module only
// sets the spatial framing.

import { ThothPage } from "../../pages/ThothPage";
import { Surface } from "../../spatial/primitives";

export function SemanticBeatSurface() {
  return (
    <Surface id="semantic-beat" title="Semantic beat">
      <ThothPage />
    </Surface>
  );
}
