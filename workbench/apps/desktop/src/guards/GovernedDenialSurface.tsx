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
        backgroundColor: "var(--surface-panel)",
        border: "1px solid var(--color-danger)",
        borderRadius: "var(--radius-overlay, 6px)",
        display: "flex",
        flexDirection: "column",
        gap: "var(--space-4, 16px)",
      }}
      data-testid="governed-denial-surface"
      aria-labelledby="denial-title"
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1 id="denial-title" style={{ fontSize: 18, lineHeight: 1.3, color: "var(--color-danger)", margin: 0 }}>
          Governed access denial: {guardResult.name}
        </h1>
        <GovernedStatusPill variant="blocked" label={guardResult.guardId} />
      </div>

      <p style={{ fontSize: 13, lineHeight: 1.5, color: "var(--fg-secondary)", margin: 0 }}>
        {guardResult.reason ?? "A required governance guard evaluated to false. Route execution halted to prevent unauthorized side effects."}
      </p>

      <div
        style={{
          padding: "var(--space-3, 12px)",
          backgroundColor: "var(--surface-code)",
          border: "1px solid var(--surface-muted)",
          borderRadius: "var(--radius-control, 4px)",
          fontSize: 12,
          color: "var(--fg-secondary)",
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
              backgroundColor: "var(--color-authority-allowed)",
              color: "var(--fg-inverse)",
              borderRadius: "var(--radius-control, 4px)",
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
              backgroundColor: "var(--surface-panel-elevated)",
              color: "var(--fg-secondary)",
              border: "1px solid var(--surface-muted)",
              borderRadius: "var(--radius-control, 4px)",
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
