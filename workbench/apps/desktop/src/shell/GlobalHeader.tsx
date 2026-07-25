import styles from "./GlobalHeader.module.css";
import { IntegrityIndicator } from "@sea-forge/ui-components";

export interface GlobalHeaderProps {
  actorName?: string;
  roleName?: string;
  policyStatus?: string;
  integrityStatus?: "verified" | "compromised" | "checking" | "unverified";
  inboxCount?: number;
  onOpenSearch?: () => void;
  onOpenInbox?: () => void;
  onToggleEvidence?: () => void;
}

export function GlobalHeader({
  actorName = "Operator",
  roleName = "Sponsor",
  policyStatus = "Loaded",
  integrityStatus = "verified",
  inboxCount = 1,
  onOpenSearch,
  onOpenInbox,
  onToggleEvidence,
}: GlobalHeaderProps) {
  return (
    <header className={styles.header} data-testid="global-context-bar">
      <div className={styles.cellBrand}>
        <span className={styles.mark} aria-hidden="true">
          SF
        </span>
        <div>
          <strong>SEA Forge</strong> <span>Cell Context</span>
        </div>
      </div>

      <div className={styles.contextChips}>
        <span className={styles.chip} aria-label="Active actor and role">
          <span className={styles.chipLabel}>Actor:</span>
          <strong>{actorName} · {roleName}</strong>
        </span>
        <span className={styles.chip} aria-label="Active policy">
          <span className={styles.chipLabel}>Policy:</span>
          <strong>{policyStatus}</strong>
        </span>
        <IntegrityIndicator status={integrityStatus} />
      </div>

      <div className={styles.actions}>
        <button
          type="button"
          id="searchButton"
          className={styles.iconButton}
          onClick={onOpenSearch}
          aria-label="Open command search"
          title="Search (/)"
        >
          <span aria-hidden="true">⌕</span> Search (/)
        </button>

        <button
          type="button"
          className={styles.inboxButton}
          onClick={onOpenInbox}
          aria-label={`Open approval inbox: ${inboxCount} approval`}
        >
          <span>Inbox</span>
          <strong>{inboxCount}</strong>
        </button>

        {onToggleEvidence && (
          <button
            type="button"
            className={styles.iconButton}
            onClick={onToggleEvidence}
            aria-label="Toggle evidence drawer"
          >
            Evidence Drawer
          </button>
        )}
      </div>
    </header>
  );
}
