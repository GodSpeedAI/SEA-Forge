import { GovernedStatusPill, AuthorityBoundaryPanel } from "@sea-forge/ui-components";

function GenericSurfacePage({ title, kind }: { title: string; kind: string }) {
  return (
    <div style={{ padding: "var(--space-6, 24px)", display: "flex", flexDirection: "column", gap: "var(--space-4, 16px)" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1 style={{ font: "var(--text-page-title, 24px / 1.2 system-ui, sans-serif)", margin: 0 }}>
          {title} Surface
        </h1>
        <GovernedStatusPill variant="ready" label={`${kind} Active`} />
      </div>

      <p style={{ color: "#c9d1d9", fontSize: 14 }}>
        Governed workspace surface for <strong>{title}</strong>. Grounded in SFWP view projections.
      </p>

      <AuthorityBoundaryPanel />
    </div>
  );
}

export function ThothPage() { return <GenericSurfacePage title="Ask Thoth" kind="Dialogue" />; }
export function AssetsPage() { return <GenericSurfacePage title="Asset Catalog" kind="Assets" />; }
export function ModelsPage() { return <GenericSurfacePage title="Domain Models" kind="Models" />; }
export function CasesPage() { return <GenericSurfacePage title="Cases Workbench" kind="Cases" />; }
export function InboxPage() { return <GenericSurfacePage title="Approval Inbox" kind="Inbox" />; }
export function OperationsPage() { return <GenericSurfacePage title="Operations Console" kind="Operations" />; }
export function EvidencePage() { return <GenericSurfacePage title="Evidence Browser" kind="Evidence" />; }
export function MemoryPage() { return <GenericSurfacePage title="Memory Recall" kind="Memory" />; }
export function CapabilitiesPage() { return <GenericSurfacePage title="Capability Matrix" kind="Capabilities" />; }
export function ArtifactsPage() { return <GenericSurfacePage title="Artifact Registry" kind="Artifacts" />; }
export function FederationPage() { return <GenericSurfacePage title="Federation Gateway" kind="Federation" />; }
export function AdminPage() { return <GenericSurfacePage title="Administration" kind="Admin" />; }
