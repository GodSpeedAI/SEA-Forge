import { UnbackedSurface } from "./UnbackedSurface";

/**
 * The route table's remaining surfaces, each resolved to what this cell can
 * actually evidence.
 *
 * This module used to hold two *specimens*: layouts copied from the design
 * specification and populated with illustrative content — a written-in Thoth
 * answer citing "UX epic §4.6" as its evidence, a `.sea` source for a domain
 * this cell has never held, three validation rows with invented verdicts. They
 * carried a watermark and forced every status pill to `unknown`, which made
 * them honest about their *states* while the content itself stayed fiction.
 *
 * Both are gone. Thoth is live — `thoth.ask` was always implemented, it was
 * simply missing from the method catalog, so no client could reach it. The
 * domain workbench has no kernel method at all and now says so.
 *
 * What is left here is routing. Each surface either reads a real method or
 * declares which method it is waiting on, and `UnbackedSurface` resolves that
 * standing from `system.hello` rather than from a claim baked into this build —
 * so the day the kernel implements one, its surface stops reporting absence
 * without anyone editing this file.
 */

// --- Live surfaces, re-exported under the names the route table uses. --------

// `thoth.ask` is in the catalog and this surface calls it.
export { ThothPage } from "./ThothPage";

// `asset.list` projects the templates, agent endpoints, and extensions this
// cell holds, with endpoint standing derived from probe records.
export { AssetCatalogPage as AssetsPage } from "./AssetCatalogPage";

// `case.list` / `case.get_overview` / `case.get_horizon` and `approval.list` /
// `approval.decide` read committed records.
export { CaseHorizonPage as CasesPage } from "./CaseHorizonPage";
export { ApprovalInboxPage as InboxPage } from "./ApprovalInboxPage";

// The operations console reads the durable events ledger.
export { OperationsPage } from "./OperationsPage";

// `system.describe` is live, so administration is genuinely backed.
export { SystemContractPage as AdminPage } from "./SystemContractPage";

// --- Surfaces with no method behind them in this cell. ----------------------

export function ModelsPage() {
  return (
    <UnbackedSurface
      title="Domain models"
      purpose="Validated .sea models with their source hashes, concept refs, and the projections pinned to them."
      method="domain.list_models"
    />
  );
}

export function MemoryPage() {
  return (
    <UnbackedSurface
      title="Memory recall"
      purpose="Developmental memory recalled under authority, with the records that constrain what may be retrieved."
      method="memory.recall"
    />
  );
}

export function CapabilitiesPage() {
  return (
    <UnbackedSurface
      title="Capability matrix"
      purpose="Demonstrated capability with its qualifying settlements — what has held under variation, not what was declared."
      method="capability.list"
    />
  );
}

export function ArtifactsPage() {
  return (
    <UnbackedSurface
      title="Artifact registry"
      purpose="Artifact identity, lineage, and maturity, with the producing evidence behind each stage transition."
      method="artifact.list"
    />
  );
}

export function FederationPage() {
  return (
    <UnbackedSurface
      title="Federation gateway"
      purpose="Peer cells, the bundles exchanged with them, and the verification standing of each import before adoption."
      method="federation.preview_export"
    />
  );
}
