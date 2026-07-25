import styles from "./WhyStatePanel.module.css";
import { GovernedStatusPill, type GovernedStatusVariant } from "./GovernedStatusPill";

export interface StateCondition {
  id: string;
  name: string;
  status: "pass" | "fail" | "warn";
  explanation: string;
}

export interface WhyStatePanelProps {
  title?: string;
  statusVariant: GovernedStatusVariant | string;
  summary: string;
  conditions?: StateCondition[];
  evidenceRef?: string;
  onInspectEvidence?: (ref: string) => void;
  onRepairAction?: () => void;
  repairActionLabel?: string;
  className?: string;
}

export function WhyStatePanel({
  title = "Why this state",
  statusVariant,
  summary,
  conditions = [],
  evidenceRef,
  onInspectEvidence,
  onRepairAction,
  repairActionLabel = "Repair path",
  className = "",
}: WhyStatePanelProps) {
  return (
    <aside
      className={`${styles.panel} ${className}`.trim()}
      data-testid="why-state-panel"
      aria-label={title}
    >
      <div className={styles.header}>
        <div className={styles.titleGroup}>
          <h3 className={styles.title}>{title}</h3>
          <GovernedStatusPill variant={statusVariant} />
        </div>
      </div>

      <p className={styles.summaryText}>{summary}</p>

      {conditions.length > 0 && (
        <ul className={styles.conditionsList}>
          {conditions.map((c) => {
            const conditionClass =
              c.status === "pass"
                ? styles.conditionPass
                : c.status === "fail"
                ? styles.conditionFail
                : styles.conditionWarn;

            return (
              <li key={c.id} className={`${styles.conditionItem} ${conditionClass}`}>
                <strong>[{c.status.toUpperCase()}]</strong>
                <span>
                  <strong>{c.name}:</strong> {c.explanation}
                </span>
              </li>
            );
          })}
        </ul>
      )}

      {evidenceRef && (
        <div className={styles.evidenceSection}>
          <span className={styles.sectionLabel}>Evidence Reference</span>
          {onInspectEvidence ? (
            <button
              type="button"
              className={styles.evidenceLink}
              onClick={() => onInspectEvidence(evidenceRef)}
            >
              Inspect evidence: {evidenceRef}
            </button>
          ) : (
            <span className={styles.evidenceLink}>{evidenceRef}</span>
          )}
        </div>
      )}

      {onRepairAction && (
        <div className={styles.actions}>
          <button
            type="button"
            className="button button--secondary"
            onClick={onRepairAction}
          >
            {repairActionLabel}
          </button>
        </div>
      )}
    </aside>
  );
}
