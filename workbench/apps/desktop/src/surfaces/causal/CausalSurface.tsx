// Causal surface: meaningful connection between objects, composed over the
// existing evidence index. Evidence records stay source-backed; this module
// only sets the spatial framing.

import { EvidencePage } from "../../pages/EvidencePage";
import { Surface } from "../../spatial/primitives";

export function CausalSurface() {
  return (
    <Surface id="causal" title="Causal">
      <EvidencePage />
    </Surface>
  );
}
