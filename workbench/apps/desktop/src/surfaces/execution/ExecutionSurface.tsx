// Execution surface: governed delegation work, composed over the existing
// delegation workbench. Authority precedes side effects; the workbench owns
// that ordering and this module does not touch it.

import { DelegationWorkbench } from "../../pages/DelegationWorkbench";
import { Surface } from "../../spatial/primitives";

export function ExecutionSurface() {
  return (
    <Surface id="execution" title="Execution">
      <DelegationWorkbench />
    </Surface>
  );
}
