import { useState, useEffect } from "react";
import { useLocation, useNavigate } from "@tanstack/react-router";
import styles from "./AppShell.module.css";
import { GlobalHeader } from "./GlobalHeader";
import { Sidebar } from "./Sidebar";
import { JourneyRibbon } from "./JourneyRibbon";
import { EvidenceDrawer, type EvidenceRecord } from "@sea-forge/ui-components";
import { EvidenceContextProvider } from "./EvidenceContext";
import { OPERATE_ROUTE_BY_PATH } from "../pages/operateRoutes";
import { useIdentity } from "../hooks/useIdentity";
import { useGuardContext } from "../guards/useGuardContext";
import { useApprovals } from "../hooks/useApprovals";
import { useServerContract } from "../hooks/useServerContract";
import { describeConnection } from "./connectionState";

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
  // The governance context bar reads the same sources every surface does, so
  // the header can never disagree with the page under it.
  const { identity, cellId, cellRoot, supervision, selectActor } = useIdentity();
  const { integrityStatus } = useGuardContext();
  const approvals = useApprovals();
  const contract = useServerContract();
  const connection = describeConnection(supervision, contract);
  const routeContext =
    ROUTE_CONTEXT[location.pathname === "/" ? "/readiness" : location.pathname] ??
    ROUTE_CONTEXT["/readiness"];
  const [isEvidenceOpen, setIsEvidenceOpen] = useState(true);
  // No evidence until a surface hands over a real record.
  //
  // This used to be seeded with a written-in `readiness_evaluation_summary`
  // whose payload was the object literal above it, re-synthesized on every
  // navigation. It rendered in the evidence drawer, beside real records, with
  // the same affordances — a claim about the cell sourced from this file. The
  // drawer already renders an honest empty state, which is the correct thing to
  // show when nothing has been inspected.
  const [selectedEvidence, setSelectedEvidence] = useState<EvidenceRecord | undefined>(
    undefined,
  );

  // Changing surface clears the previous surface's evidence rather than
  // carrying it over, which would attribute one page's record to another.
  useEffect(() => {
    setSelectedEvidence(undefined);
  }, [location.pathname]);

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

      {/*
        A window with no kernel behind it must say so once, loudly, rather than
        letting every surface report its own call failure. `role="alert"` because
        nothing in the application will work until this is resolved, and the
        message carries the cell root so the operator knows which cell failed.
      */}
      {supervision?.state === "unavailable" && (
        <div className={styles.cellAlert} role="alert" data-od-id="cell-unavailable">
          <strong>No cell is running.</strong> {supervision.message}
          {cellRoot ? <span className="machine-value"> ({cellRoot})</span> : null}
        </div>
      )}

      <GlobalHeader
        actorName={identity?.actor?.actorId}
        roleName={identity?.actor?.role}
        availableActors={identity?.available.flatMap((actor) =>
          actor.roles.slice(0, 1).map((role) => ({ actorId: actor.actor_id, role })),
        )}
        onSelectActor={selectActor}
        cellName={cellId}
        integrityStatus={integrityStatus}
        // `undefined` unless the inbox was actually read. An unread inbox, an
        // unreadable journal, and an empty queue are three different things,
        // and only the third is "0 approvals".
        inboxCount={
          approvals.isLoading || approvals.error || approvals.unreadable
            ? undefined
            : approvals.approvals.length
        }
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
          <span className={`state-dot ${connection.ready ? "state-dot--ready" : ""}`} />
          {connection.label}
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
