import styles from "./SourceFreshnessBadge.module.css";

export type FreshnessMode = "live" | "cached" | "stale" | "offline" | "checking";

export interface SourceFreshnessBadgeProps {
  mode: FreshnessMode;
  sourceName?: string;
  lastVerifiedAt?: string;
  onClick?: () => void;
  className?: string;
}

export function SourceFreshnessBadge({
  mode,
  sourceName = "Source",
  lastVerifiedAt,
  onClick,
  className = "",
}: SourceFreshnessBadgeProps) {
  const dotClasses: Record<FreshnessMode, string> = {
    live: styles.liveDot,
    cached: styles.cachedDot,
    stale: styles.staleDot,
    offline: styles.offlineDot,
    checking: styles.checkingDot,
  };

  const modeLabels: Record<FreshnessMode, string> = {
    live: "Source current",
    cached: "Cached snapshot",
    stale: "Stale projection",
    offline: "Offline record",
    checking: "Checking source",
  };

  const dotClass = dotClasses[mode] || styles.staleDot;
  const labelText = `${modeLabels[mode] || mode} · ${sourceName}`;

  const content = (
    <>
      <span className={`${styles.dot} ${dotClass}`} aria-hidden="true" />
      <span className={styles.textGroup}>
        <span>{labelText}</span>
        {lastVerifiedAt && (
          <span className={styles.timestamp}>({lastVerifiedAt})</span>
        )}
      </span>
    </>
  );

  if (onClick) {
    return (
      <button
        type="button"
        className={`${styles.badge} ${className}`.trim()}
        onClick={onClick}
        data-testid="source-freshness-badge"
        aria-label={`Source freshness: ${labelText}`}
      >
        {content}
      </button>
    );
  }

  return (
    <div
      className={`${styles.badge} ${className}`.trim()}
      data-testid="source-freshness-badge"
      aria-label={`Source freshness: ${labelText}`}
    >
      {content}
    </div>
  );
}
