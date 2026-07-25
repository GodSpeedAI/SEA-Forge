import { WhyStatePanel, AvailabilityLadder, GovernedStatusPill } from "@sea-forge/ui-components";

export function ReadinessPage() {
  return (
    <div style={{ padding: "var(--space-6, 24px)", display: "flex", flexDirection: "column", gap: "var(--space-4, 16px)" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1 style={{ font: "var(--text-page-title, 24px / 1.2 system-ui, sans-serif)", margin: 0 }}>
          Readiness Console
        </h1>
        <GovernedStatusPill variant="ready" label="Cell Ready" />
      </div>

      <WhyStatePanel
        statusVariant="ready"
        summary="Kernel execution and SFWP local transport are verified. Operation-sensitive readiness active."
        conditions={[
          { id: "c1", name: "Local Sync Kernel", status: "pass", explanation: "Verified zero-async kernel execution" },
          { id: "c2", name: "SFWP Socket Transport", status: "pass", explanation: "NDJSON unix socket bound" },
        ]}
        evidenceRef="ev_readiness_init_proof"
      />

      <AvailabilityLadder currentLevel={2} />
    </div>
  );
}
