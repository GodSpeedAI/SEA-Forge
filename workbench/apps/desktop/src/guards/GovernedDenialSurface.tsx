import { Link } from "@tanstack/react-router";
import type { GuardResult } from "./guards";
import { GovernedStatusPill } from "@sea-forge/ui-components";

export interface GovernedDenialSurfaceProps {
  guardResult: GuardResult;
  onRetry?: () => void;
}

export function GovernedDenialSurface({ guardResult, onRetry }: GovernedDenialSurfaceProps) {
  return (
    <main
      style={{
        padding: "var(--space-6, 24px)",
        maxWidth: 640,
        margin: "40px auto",
        backgroundColor: "var(--surface-panel, rgba(22, 27, 34, 0.95))",
        border: "1px solid var(--color-authority-blocked-border, rgba(248, 81, 73, 0.4))",
        borderRadius: "var(--radius-card, 8px)",
        display: "flex",
        flexDirection: "column",
        gap: "var(--space-4, 16px)",
      }}
      data-testid="governed-denial-surface"
      aria-labelledby="denial-title"
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1 id="denial-title" style={{ font: "var(--text-heading-md, 18px / 1.3 system-ui, sans-serif)", color: "#f85149", margin: 0 }}>
          Governed Access Denial — {guardResult.name}
        </h1>
        <GovernedStatusPill variant="blocked" label={guardResult.guardId} />
      </div>

      <p style={{ font: "var(--text-body-sm, 13px / 1.4 system-ui, sans-serif)", color: "#c9d1d9", margin: 0 }}>
        {guardResult.reason ?? "A required governance guard evaluated to false. Route execution halted to prevent unauthorized side effects."}
      </p>

      <div
        style={{
          padding: "var(--space-3, 12px)",
          backgroundColor: "#0d1117",
          border: "1px solid #30363d",
          borderRadius: "var(--radius-control, 6px)",
          fontSize: 12,
          color: "#8b949e",
        }}
      >
        <strong>Guard ID:</strong> <code className="machine-value">{guardResult.guardId}</code>
        <br />
        <strong>Status:</strong> Guard evaluation failed (Fail-closed)
      </div>

      <div style={{ display: "flex", gap: "var(--space-3, 12px)", marginTop: 8 }}>
        {guardResult.repairRoute && (
          <Link
            to={guardResult.repairRoute}
            style={{
              padding: "8px 16px",
              backgroundColor: "var(--color-authority-ready-bg, #2ea44f)",
              color: "#ffffff",
              borderRadius: "var(--radius-control, 6px)",
              textDecoration: "none",
              fontWeight: 600,
              fontSize: 13,
            }}
          >
            {guardResult.repairLabel ?? "Open repair path"}
          </Link>
        )}
        {onRetry && (
          <button
            type="button"
            onClick={onRetry}
            style={{
              padding: "8px 16px",
              backgroundColor: "#21262d",
              color: "#c9d1d9",
              border: "1px solid #30363d",
              borderRadius: "var(--radius-control, 6px)",
              cursor: "pointer",
              fontSize: 13,
            }}
          >
            Re-evaluate Guard
          </button>
        )}
      </div>
    </main>
  );
}
