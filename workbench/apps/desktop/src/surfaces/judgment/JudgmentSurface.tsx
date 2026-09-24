// Judgment surface: bounded human judgment over pending approvals, composed
// over the existing approval inbox. Approvals round-trip through
// `approval.decide`; nothing here invents judgment semantics.

import { ApprovalInboxPage } from "../../pages/ApprovalInboxPage";
import { Surface } from "../../spatial/primitives";

export function JudgmentSurface() {
  return (
    <Surface id="judgment" title="Judgment">
      <ApprovalInboxPage />
    </Surface>
  );
}
