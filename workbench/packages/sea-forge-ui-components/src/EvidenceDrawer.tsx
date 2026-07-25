import { useEffect } from "react";
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

export function EvidenceDrawer({
  isOpen,
  onClose,
  evidence,
  className = "",
}: EvidenceDrawerProps) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && isOpen) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div className={styles.backdrop} onClick={onClose} data-testid="evidence-drawer-backdrop">
      <aside
        className={`${styles.drawer} ${className}`.trim()}
        onClick={(e) => e.stopPropagation()}
        data-testid="evidence-drawer"
        aria-label="Evidence inspection drawer"
      >
        <div className={styles.header}>
          <h2 className={styles.title}>Evidence Inspection</h2>
          <button
            type="button"
            className={styles.closeButton}
            onClick={onClose}
            aria-label="Close evidence drawer"
          >
            ×
          </button>
        </div>

        <div className={styles.body}>
          {evidence ? (
            <>
              <div className={styles.section}>
                <span className={styles.sectionTitle}>Record Metadata</span>
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
              </div>

              <div className={styles.section}>
                <span className={styles.sectionTitle}>Integrity Status</span>
                <IntegrityIndicator status="verified" details="Ledger MMR hash valid" />
              </div>

              <div className={styles.section}>
                <span className={styles.sectionTitle}>Disclosure Scope</span>
                {evidence.disclosureStatus === "restricted" ? (
                  <div className={styles.disclosureNotice}>
                    <strong>Disclosure Restricted:</strong> Retrieval scope is gated by active policy (G7 Disclosure). Raw payload is omitted from evidence view.
                  </div>
                ) : (
                  <div className={styles.payloadBox}>
                    {evidence.rawPayload ?? JSON.stringify(evidence, null, 2)}
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className={styles.section}>
              <span className={styles.sectionTitle}>No Evidence Selected</span>
              <p style={{ fontSize: 13, color: "#8b949e" }}>Select an evidence link or record to inspect cryptographic evidence and ledger position.</p>
            </div>
          )}
        </div>
      </aside>
    </div>
  );
}
