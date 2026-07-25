import { useState, useEffect } from "react";
import { useNavigate } from "@tanstack/react-router";
import styles from "./AppShell.module.css";
import { GlobalHeader } from "./GlobalHeader";
import { Sidebar } from "./Sidebar";
import { JourneyRibbon } from "./JourneyRibbon";
import { EvidenceDrawer, type EvidenceRecord } from "@sea-forge/ui-components";

export interface AppShellProps {
  children: React.ReactNode;
  currentJourneyStep?: string;
  guardFailed?: boolean;
}

export function AppShell({ children, currentJourneyStep = "Readiness" }: AppShellProps) {
  const navigate = useNavigate();
  const [isEvidenceOpen, setIsEvidenceOpen] = useState(false);
  const [selectedEvidence] = useState<EvidenceRecord | null>(null);

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
    <div className={styles.appShell} data-od-id="application-shell">
      <a className={styles.skipLink} href="#main-content">
        Skip to governed focus
      </a>

      <GlobalHeader
        onOpenSearch={() => alert("Search command palette (Press /)")}
        onOpenInbox={() => navigate({ to: "/inbox" })}
        onToggleEvidence={() => setIsEvidenceOpen((prev) => !prev)}
      />

      <div className={styles.bodyLayout}>
        <Sidebar />

        <div className={styles.mainWorkspace}>
          <JourneyRibbon currentStep={currentJourneyStep} />
          <main id="main-content" tabIndex={-1} style={{ flex: 1, outline: "none" }}>
            {children}
          </main>
        </div>
      </div>

      <EvidenceDrawer
        isOpen={isEvidenceOpen}
        onClose={() => setIsEvidenceOpen(false)}
        evidence={
          selectedEvidence ?? {
            id: "ev_shell_active_session",
            kind: "session_integrity_proof",
            ledgerUlid: "01HQX_SHELL_PROOF",
            digest: "sha256:7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069",
            disclosureStatus: "permitted",
            rawPayload: JSON.stringify(
              {
                session: "active_desktop_session",
                cell: "local_sync_cell",
                integrity: "verified",
              },
              null,
              2
            ),
          }
        }
      />
    </div>
  );
}
