import styles from "./IntegrityIndicator.module.css";

export type IntegrityStatus = "verified" | "compromised" | "checking" | "unverified";

export interface IntegrityIndicatorProps {
  status: IntegrityStatus;
  details?: string;
  className?: string;
}

export function IntegrityIndicator({
  status,
  details,
  className = "",
}: IntegrityIndicatorProps) {
  const statusClasses: Record<IntegrityStatus, { text: string; dot: string; label: string }> = {
    verified: { text: styles.verified, dot: styles.verifiedDot, label: "Integrity verified" },
    compromised: { text: styles.compromised, dot: styles.compromisedDot, label: "Integrity compromised" },
    checking: { text: styles.checking, dot: styles.checkingDot, label: "Integrity check in progress" },
    unverified: { text: styles.unverified, dot: styles.unverifiedDot, label: "Integrity unverified" },
  };

  const config = statusClasses[status] || statusClasses.unverified;
  const label = details ? `${config.label}: ${details}` : config.label;

  return (
    <div
      className={`${styles.indicator} ${config.text} ${className}`.trim()}
      data-testid="integrity-indicator"
      data-status={status}
      title={label}
    >
      <span className={`${styles.dot} ${config.dot}`} aria-hidden="true" />
      <span>{config.label}</span>
    </div>
  );
}
