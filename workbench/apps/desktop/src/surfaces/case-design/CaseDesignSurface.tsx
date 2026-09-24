// Case-design surface: authoring new governed work, composed over the
// existing case-creation workbench. Creation stays behind the kernel's
// admission/settlement path; this module only sets the spatial framing.

import { CaseCreationWorkbench } from "../../pages/CaseCreationWorkbench";
import { Surface } from "../../spatial/primitives";

export function CaseDesignSurface() {
  return (
    <Surface id="case-design" title="Case design">
      <CaseCreationWorkbench />
    </Surface>
  );
}
