import styles from "./GovernedStatusPill.module.css";

export type GovernedStatusVariant =
  | "ready"
  | "degraded"
  | "blocked"
  | "integrity_halted"
  | "running"
  | "finished"
  | "cancelled"
  | "failed"
  | "pending"
  | "committed"
  | "rejected"
  | "unknown";

export interface GovernedStatusPillProps {
  variant?: GovernedStatusVariant | string | null;
  label?: string;
  className?: string;
}

const KNOWN_VARIANTS = new Set<GovernedStatusVariant>([
  "ready",
  "degraded",
  "blocked",
  "integrity_halted",
  "running",
  "finished",
  "cancelled",
  "failed",
  "pending",
  "committed",
  "rejected",
  "unknown",
]);

export function GovernedStatusPill({
  variant,
  label,
  className = "",
}: GovernedStatusPillProps) {
  const isKnown = typeof variant === "string" && KNOWN_VARIANTS.has(variant as GovernedStatusVariant);
  const resolvedVariant: GovernedStatusVariant = isKnown ? (variant as GovernedStatusVariant) : "unknown";

  const displayLabel = label ?? resolvedVariant.replace("_", " ");
  const variantClass = styles[resolvedVariant] || styles.unknown;

  return (
    <span
      className={`${styles.pill} ${variantClass} status-pill--${resolvedVariant} ${className}`.trim()}
      data-testid="governed-status-pill"
      data-variant={resolvedVariant}
    >
      <span className={styles.pillDot} aria-hidden="true" />
      <span>{displayLabel}</span>
    </span>
  );
}
