// AppShell — the spatial application shell.
//
// Root composition (task §5):
//
//   AppShell
//   ├── CoreViewport   (persistent; mounted once, never remounts on focus/surface change)
//   ├── SpatialSurface (route content framed as Surface/Object/Artifact)
//   ├── Composer       (persistent, context-bound)
//   └── transient/contextual overlays (identity, readout, hints, evidence, alerts)
//
// What was removed: the page-based SaaS chrome (Sidebar, GlobalHeader,
// JourneyRibbon, mockup fidelity kit). What was retained: every underlying
// capability and integration contract — guards, hooks/ports, pages (now
// composed as surfaces), the evidence drawer, connection state, and the cell
// availability alert.

import { useEffect, useMemo, useState } from "react";
import { useLocation, useNavigate } from "@tanstack/react-router";
import { CoreViewport } from "../core/CoreViewport";
import type { CoreRenderer } from "../core/CoreRenderer";
import { DEFAULT_VISUAL_PARAMS, type CoreVisualParams } from "../core/CoreVisualState";
import { ContextIdentity } from "../core/chrome/ContextIdentity";
import { ContextReadout } from "../core/chrome/ContextReadout";
import { InteractionHints } from "../core/chrome/InteractionHints";
import { SurfaceRail } from "./SurfaceRail";
import { Composer } from "../composer/Composer";
import { CoreTuningPanel } from "../dev/CoreTuningPanel";
import { EvidenceDrawer, type EvidenceRecord } from "@sea-forge/ui-components";
import { EvidenceContextProvider } from "./EvidenceContext";
import { focusController, type FocusTarget } from "../spatial/focus/FocusController";
import { zoomController } from "../spatial/zoom/ZoomController";
import { projectAffordances } from "../projections/AffordanceProjection";
import {
  activityFromPendingWork,
  projectCoreVisual,
} from "../projections/CoreVisualProjection";
import { useIdentity } from "../hooks/useIdentity";
import { useGuardContext } from "../guards/useGuardContext";
import { useApprovals } from "../hooks/useApprovals";
import { useCaseList } from "../hooks/useCases";
import { useServerContract } from "../hooks/useServerContract";
import { describeConnection } from "./connectionState";
import styles from "./AppShell.module.css";

export interface AppShellProps {
  children: React.ReactNode;
}

const SURFACE_BY_PATH: Array<{ prefix: string; id: string }> = [
  { prefix: "/cases/new", id: "case-design" },
  { prefix: "/cases", id: "case" },
  { prefix: "/thoth", id: "semantic-beat" },
  { prefix: "/artifacts", id: "artifacts" },
  { prefix: "/operations", id: "temporal" },
  { prefix: "/evidence", id: "causal" },
  { prefix: "/inbox", id: "judgment" },
  { prefix: "/delegate", id: "execution" },
  { prefix: "/runs", id: "temporal" },
];

function surfaceIdFor(pathname: string): string {
  if (pathname === "/") return "home";
  for (const { prefix, id } of SURFACE_BY_PATH) {
    if (pathname.startsWith(prefix)) return id;
  }
  return "home";
}

function focusLabel(target: FocusTarget): string {
  if (target.kind === "object") return target.objectId;
  if (target.kind === "anchor") return "anchor";
  return "CORE";
}

export function AppShell({ children }: AppShellProps) {
  const navigate = useNavigate();
  const location = useLocation();
  const surfaceId = surfaceIdFor(location.pathname);

  // Same governed sources every surface reads; the chrome can never disagree
  // with the content under it.
  const { supervision } = useIdentity();
  const approvals = useApprovals();
  const cases = useCaseList();
  const contract = useServerContract();
  const connection = describeConnection(supervision, contract);

  const inboxCount =
    approvals.isLoading || approvals.error || approvals.unreadable
      ? undefined
      : approvals.approvals.length;

  const [renderer, setRenderer] = useState<CoreRenderer | null>(null);
  const [focus, setFocus] = useState<FocusTarget>(focusController.current);
  const [zoom, setZoom] = useState(zoomController.current);
  const [tuning, setTuning] = useState<CoreVisualParams>({ ...DEFAULT_VISUAL_PARAMS });
  const [tuningActive, setTuningActive] = useState(false);
  const [fps, setFps] = useState<number | null>(null);
  const [isEvidenceOpen, setIsEvidenceOpen] = useState(true);
  const [selectedEvidence, setSelectedEvidence] = useState<EvidenceRecord | undefined>(
    undefined,
  );

  useEffect(() => focusController.subscribe(setFocus), []);
  useEffect(() => zoomController.subscribe(setZoom), []);

  // FocusController owns semantic intent; the renderer executes visual
  // intent. Returning to CORE restores the canonical Home framing.
  useEffect(() => {
    if (!renderer) return;
    renderer.setFocus(focusController.toCoreFocus());
    if (focusController.isHome()) renderer.setCameraIntent({ returnHome: true });
  }, [renderer, focus]);

  useEffect(() => {
    renderer?.setZoom(zoom);
  }, [renderer, zoom]);

  const visualState = useMemo(() => {
    const activity = activityFromPendingWork({
      pendingApprovals: inboxCount ?? null,
      unreadableCases: cases.error ? null : cases.unreadable.length,
    });
    const intent = projectCoreVisual({
      activity,
      focusDisplaced: focus.kind !== "core",
    });
    return {
      params: tuningActive ? tuning : intent.params,
      focus: focusController.toCoreFocus(),
      zoom,
      activity: intent.activity,
    };
  }, [inboxCount, cases.error, cases.unreadable.length, tuning, tuningActive, focus, zoom]);

  const affordances = useMemo(
    () =>
      projectAffordances({
        surfaceId,
        focusedObjectId: focus.kind === "object" ? focus.objectId : null,
        inboxCount,
        connectionReady: connection.ready,
      }),
    [surfaceId, focus, inboxCount, connection.ready],
  );

  // Changing surface clears the previous surface's evidence rather than
  // carrying it over, which would attribute one surface's record to another.
  useEffect(() => {
    setSelectedEvidence(undefined);
  }, [location.pathname]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)
      ) {
        return;
      }
      if (event.key === "r" || event.key === "R") {
        event.preventDefault();
        void navigate({ to: "/readiness" });
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [navigate]);

  const { integrityStatus } = useGuardContext();

  return (
    <div className={styles.appShell} data-testid="application-shell">
      <a className={styles.skipLink} href="#main-content">
        Skip to governed focus
      </a>

      <CoreViewport
        visualState={visualState}
        onRendererReady={setRenderer}
        onFps={setFps}
      />

      {supervision?.state === "unavailable" && (
        <div className={styles.cellAlert} role="alert" data-testid="cell-unavailable">
          <strong>No cell is running.</strong> {supervision.message}
        </div>
      )}

      <ContextIdentity />
      <ContextReadout
        focus={focusLabel(focus)}
        surface={surfaceId}
        settlement={integrityStatus ?? (connection.ready ? "connected" : "unknown")}
        activity={visualState.activity >= 0.75 ? "elevated" : "nominal"}
      />
      <InteractionHints />

      <SurfaceRail surfaceId={surfaceId} inboxCount={inboxCount} />
      <Composer surfaceId={surfaceId} affordances={affordances} />

      <main id="main-content" tabIndex={-1} className={styles.surfaceLayer}>
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

      <footer className={styles.connectionBar} data-testid="connection-state-bar">
        <span>
          <span
            className={styles.stateDot}
            data-ready={connection.ready ? "true" : "false"}
          />
          {connection.label}
        </span>
      </footer>

      <EvidenceDrawer
        isOpen={isEvidenceOpen}
        onClose={() => setIsEvidenceOpen(false)}
        evidence={selectedEvidence}
      />

      {import.meta.env.DEV ? (
        <CoreTuningPanel
          params={tuning}
          fps={fps}
          onChange={(params) => {
            setTuning(params);
            setTuningActive(true);
          }}
        />
      ) : null}
    </div>
  );
}
