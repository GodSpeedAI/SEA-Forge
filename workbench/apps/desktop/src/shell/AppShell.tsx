import { useState, useEffect } from "react";
import { useLocation, useNavigate } from "@tanstack/react-router";
import styles from "./AppShell.module.css";
import { GlobalHeader } from "./GlobalHeader";
import { Sidebar } from "./Sidebar";
import { JourneyRibbon } from "./JourneyRibbon";
import { EvidenceDrawer, type EvidenceRecord } from "@sea-forge/ui-components";
import { EvidenceContextProvider } from "./EvidenceContext";
import { OPERATE_ROUTE_BY_PATH } from "../pages/operateRoutes";

export interface AppShellProps {
  children: React.ReactNode;
  currentJourneyStep?: string;
  guardFailed?: boolean;
}

interface ShellRouteContext {
  label: string;
  journeyStep: string;
  reason: string;
}

const ROUTE_CONTEXT: Record<string, ShellRouteContext> = {
  "/readiness": {
    label: "readiness",
    journeyStep: "Readiness",
    reason:
      "External delegation needs a verified endpoint and an authority boundary before the path becomes spendable.",
  },
  ...Object.fromEntries(
    Object.entries(OPERATE_ROUTE_BY_PATH).map(([path, route]) => [
      path,
      {
        label: route.shellLabel,
        journeyStep: route.journeyStep,
        reason: route.reason,
      },
    ]),
  ),
};

export function AppShell({ children, currentJourneyStep }: AppShellProps) {
  const navigate = useNavigate();
  const location = useLocation();
  const routeContext =
    ROUTE_CONTEXT[location.pathname === "/" ? "/readiness" : location.pathname] ??
    ROUTE_CONTEXT["/readiness"];
  const [isEvidenceOpen, setIsEvidenceOpen] = useState(true);
  const [selectedEvidence, setSelectedEvidence] = useState<EvidenceRecord>({
    id: "readiness.get",
    kind: "readiness_evaluation_summary",
    disclosureStatus: "permitted",
    rawPayload: JSON.stringify(
      {
        state: "readiness",
        source: "readiness.get",
        display: "source-backed projection",
      },
      null,
      2,
    ),
  });

  useEffect(() => {
    if (routeContext.label === "readiness") {
      setSelectedEvidence({
        id: "readiness.get",
        kind: "readiness_evaluation_summary",
        disclosureStatus: "permitted",
        rawPayload: JSON.stringify(
          {
            state: "readiness",
            source: "readiness.get",
            display: "source-backed projection",
          },
          null,
          2,
        ),
      });
    } else {
      const routeId = routeContext.label.replaceAll(" ", "_");
      setSelectedEvidence({
        id: `${routeId}.state`,
        kind: `operate_${routeId}_state`,
        disclosureStatus: "permitted",
        rawPayload: JSON.stringify(
          {
            route: routeContext.label,
            reason: routeContext.reason,
            display: "copied specification projection",
          },
          null,
          2,
        ),
      });
    }
    setIsEvidenceOpen(true);
  }, [location.pathname, routeContext.label, routeContext.reason]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Ignore if input/textarea is active
      const target = event.target as HTMLElement | null;
      if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)) {
        return;
      }

      if (event.key === "r" || event.key === "R") {
        event.preventDefault();
        navigate({ to: "/readiness" });
      } else if (event.key === "/") {
        event.preventDefault();
        const searchBtn = document.getElementById("searchButton");
        searchBtn?.focus();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [navigate]);

  return (
    <div
      className={`${styles.appShell} app-shell ${!isEvidenceOpen ? `${styles.drawerClosed} drawer-closed` : ""}`}
      data-od-id="application-shell"
    >
      <a className={styles.skipLink} href="#main-content">
        Skip to governed focus
      </a>

      <GlobalHeader
        onOpenSearch={() => alert("Search command palette (Press /)")}
        onOpenInbox={() => navigate({ to: "/inbox" })}
        onToggleEvidence={() => setIsEvidenceOpen((prev) => !prev)}
      />

      <Sidebar />

      <div className={`${styles.mainWorkspace} main-workspace`} data-od-id="readiness-workspace">
        <JourneyRibbon
          currentStep={currentJourneyStep ?? routeContext.journeyStep}
        />
        <main id="main-content" tabIndex={-1} style={{ outline: "none" }}>
          <EvidenceContextProvider
            value={{
              inspectEvidence: (evidence) => {
                setSelectedEvidence(evidence);
                setIsEvidenceOpen(true);
              },
            }}
          >
            {children}
          </EvidenceContextProvider>
        </main>
      </div>

      <footer
        className={`${styles.connectionBar} connection-bar`}
        data-od-id="connection-state-bar"
      >
        <span>
          <span
            className={`state-dot ${
              routeContext.label === "readiness" ? "state-dot--ready" : ""
            }`}
          />
          {routeContext.label === "readiness"
            ? "Source projection current"
            : "Specification view · not live"}
        </span>
        <span className="machine-value">
          Keyboard: R run checks · I intended work · B blocker · E evidence
        </span>
      </footer>

      <EvidenceDrawer
        isOpen={isEvidenceOpen}
        onClose={() => setIsEvidenceOpen(false)}
        evidence={selectedEvidence}
        className="evidence-drawer"
      />
    </div>
  );
}
