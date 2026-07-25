import styles from "./DualStateIndicator.module.css";
import { GovernedStatusPill, type GovernedStatusVariant } from "./GovernedStatusPill";

export interface DualStateIndicatorProps {
  executionState: GovernedStatusVariant | string;
  settlementState: GovernedStatusVariant | string;
  className?: string;
}

export function DualStateIndicator({
  executionState,
  settlementState,
  className = "",
}: DualStateIndicatorProps) {
  return (
    <div
      className={`${styles.container} ${className}`.trim()}
      data-testid="dual-state-indicator"
      aria-label="Execution and settlement states"
    >
      <div className={styles.stateBox}>
        <span className={styles.stateLabel}>Execution State</span>
        <GovernedStatusPill variant={executionState} />
      </div>
      <span className={styles.separator} aria-hidden="true">
        │
      </span>
      <div className={styles.stateBox}>
        <span className={styles.stateLabel}>Settlement State</span>
        <GovernedStatusPill variant={settlementState} />
      </div>
    </div>
  );
}
