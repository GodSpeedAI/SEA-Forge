// Temporal surface: the same object/system inspected through developmental
// or historical states. Composed over the existing operations monitor (the
// event cursor is the kernel-minted position in time). No fabricated history.

import { OperationsPage } from "../../pages/OperationsPage";
import { Surface } from "../../spatial/primitives";
import { timeVersionController } from "../../spatial/time/TimeVersionController";

export function TemporalSurface() {
  const pinned = timeVersionController.current;
  return (
    <Surface id="temporal" title="Temporal">
      <p data-testid="temporal-position">
        {pinned ? `Pinned at ${pinned.cursor} (${pinned.label})` : "Live head"}
      </p>
      <OperationsPage />
    </Surface>
  );
}
