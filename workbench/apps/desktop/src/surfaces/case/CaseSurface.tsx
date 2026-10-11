// Case surface: the existing case-horizon capability, composed as a spatial
// surface. No domain behavior fabricated: the wrapped page owns its governed
// reads; this module only sets the spatial framing + focus wiring.

import { CaseHorizonPage } from "../../pages/CaseHorizonPage";
import { Surface } from "../../spatial/primitives";
import { focusController } from "../../spatial/focus/FocusController";

export function CaseSurface() {
  return (
    <Surface id="case" title="Case">
      <CaseHorizonPage />
      <button type="button" onClick={() => focusController.focus({ kind: "object", objectId: "case" })}>
        Focus this case
      </button>
    </Surface>
  );
}
