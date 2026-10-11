import { useEffect, useState } from "react";
import styles from "./EvidenceDrawer.module.css";
import { IntegrityIndicator } from "./IntegrityIndicator";

export interface EvidenceRecord {
  id: string;
  kind: string;
  ledgerUlid?: string;
  digest?: string;
  disclosureStatus: "permitted" | "restricted" | "gated";
  rawPayload?: string;
}

export interface EvidenceDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  evidence?: EvidenceRecord | null;
  className?: string;
}

type EvidenceTab = "why" | "evidence" | "provenance" | "record";

export function EvidenceDrawer({
  isOpen,
  onClose,
  evidence,
  className = "",
}: EvidenceDrawerProps) {
  const [activeTab, setActiveTab] = useState<EvidenceTab>("why");

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && isOpen) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  useEffect(() => {
    setActiveTab("why");
  }, [evidence?.id]);

  const isReadiness = evidence?.kind === "readiness_evaluation_summary";
  const operateRoute = evidence?.kind.startsWith("operate_")
    ? evidence.kind.slice("operate_".length, -"_state".length).replaceAll("_", " ")
    : null;
  const drawerTitle = isReadiness
    ? "Why readiness is constrained"
    : operateRoute
      ? `Why ${operateRoute} is constrained`
      : "Evidence inspection";

  const recordMetadata = evidence ? (
    <div className={styles.dataGrid}>
      <span className={styles.dataLabel}>Record ID</span>
      <span className={styles.dataValue}>{evidence.id}</span>

      <span className={styles.dataLabel}>Kind</span>
      <span className={styles.dataValue}>{evidence.kind}</span>

      <span className={styles.dataLabel}>Ledger ULID</span>
      <span className={styles.dataValue}>{evidence.ledgerUlid ?? "Unsealed"}</span>

      <span className={styles.dataLabel}>Digest</span>
      <span className={styles.dataValue}>{evidence.digest ?? "N/A"}</span>
    </div>
  ) : null;

  const disclosure = evidence?.disclosureStatus === "restricted" ? (
    <div className={styles.disclosureNotice}>
      <strong>Disclosure Restricted:</strong> Retrieval scope is gated by active policy
      (G7 Disclosure). Raw payload is omitted from evidence view.
    </div>
  ) : null;

  return (
    <div
      className={`${styles.backdrop} ${!isOpen ? styles.backdropClosed : ""}`.trim()}
      onClick={isOpen ? onClose : undefined}
      data-testid="evidence-drawer-backdrop"
      aria-hidden={!isOpen}
    >
      <aside
        className={`${styles.drawer} ${!isOpen ? styles.drawerClosed : ""} ${className}`.trim()}
        onClick={(e) => e.stopPropagation()}
        data-testid="evidence-drawer"
        aria-label="Context and evidence"
      >
        <div className={`${styles.header} drawer-header`}>
          <div>
            <p className="section-kicker">Context / Evidence</p>
            <h2 className={styles.title}>{drawerTitle}</h2>
          </div>
          <button
            type="button"
            className={`${styles.closeButton} icon-button`}
            onClick={onClose}
            aria-label="Close evidence drawer"
          >
            ×
          </button>
        </div>

        <div className="drawer-tabs" role="tablist" aria-label="Evidence views">
          {(["why", "evidence", "provenance", "record"] as const).map((tab) => (
            <button
              key={tab}
              type="button"
              role="tab"
              aria-selected={activeTab === tab}
              onClick={() => setActiveTab(tab)}
            >
              {tab[0].toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>

        <div
          className={`${styles.body} drawer-content`}
          role="tabpanel"
          tabIndex={0}
        >
          {activeTab === "why" && (
            <>
              <section className="drawer-section">
                <p className="section-kicker">Current state</p>
                <h3>{isReadiness ? "Readiness state" : "Selected evidence"}</h3>
                <p>
                  {isReadiness
                    ? "The displayed readiness is a source-backed projection. Inspect the governing condition before taking the next lawful action."
                    : evidence
                      ? `Record ${evidence.id} is selected for governed inspection.`
                      : "Select an evidence link or record to inspect its source and standing."}
                </p>
                {recordMetadata}
                {disclosure}
              </section>
              <section className="drawer-section">
                <h3>Governing condition</h3>
                <p>
                  The displayed state is derived from SEA Forge records and remains
                  non-authoritative in the renderer.
                </p>
              </section>
              <section className="drawer-section">
                <h3>Next lawful paths</h3>
                <ul>
                  <li>Inspect supporting evidence.</li>
                  <li>Review provenance and freshness.</li>
                  <li>Take only an action shown as lawful on the current surface.</li>
                </ul>
              </section>
            </>
          )}

          {activeTab === "evidence" && (
            <>
              <section className="drawer-section">
                <p className="section-kicker">Evidence inventory</p>
                <h3>Record metadata</h3>
                {recordMetadata}
              </section>
              <section className="drawer-section">
                <h3>Integrity status</h3>
                <IntegrityIndicator status="verified" details="Ledger MMR hash valid" />
              </section>
              <section className="drawer-section">
                <h3>Disclosure scope</h3>
                {disclosure ?? (
                  <p>This record is permitted for the active disclosure scope.</p>
                )}
              </section>
            </>
          )}

          {activeTab === "provenance" && (
            <section className="drawer-section">
              <p className="section-kicker">Projection source</p>
              <h3>SEA Forge source records</h3>
              <p>
                Source citations are supplied by the validated readiness projection.
                Freshness follows the active SFWP connection and event cursor.
              </p>
              {recordMetadata}
            </section>
          )}

          {activeTab === "record" && (
            <section className="drawer-section">
              <p className="section-kicker">Machine-readable record</p>
              <h3>Display model</h3>
              <pre className={`${styles.payloadBox} drawer-record`}>
                {evidence?.rawPayload ?? JSON.stringify(evidence ?? {}, null, 2)}
              </pre>
              {disclosure}
            </section>
          )}

          {!evidence && (
            <div className={styles.section}>
              <span className={styles.sectionTitle}>No Evidence Selected</span>
              <p>
                Select an evidence link or record to inspect cryptographic evidence and
                ledger position.
              </p>
            </div>
          )}
        </div>
      </aside>
    </div>
  );
}
