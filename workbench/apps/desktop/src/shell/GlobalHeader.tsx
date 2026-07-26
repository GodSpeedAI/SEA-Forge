import styles from "./GlobalHeader.module.css";

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
    <header
      className={`${styles.header} global-bar`}
      data-od-id="global-context-bar"
      data-testid="global-context-bar"
    >
      <div className={`${styles.productCell} product-cell`}>
        <span className={`${styles.productMark} product-mark`} aria-hidden="true">
          SF
        </span>
        <div>
          <strong>SEA Forge</strong>
          <span>Cell context</span>
        </div>
      </div>

      <div className={`${styles.globalContext} global-context`}>
        <button
          className={`${styles.contextChip} context-chip`}
          type="button"
          aria-label="Active actor and role"
        >
          <span>Actor</span>
          <strong>{actorName} · {roleName}</strong>
        </button>
        <button
          className={`${styles.contextChip} context-chip`}
          type="button"
          aria-label="Active policy"
        >
          <span>Policy</span>
          <strong>{policyStatus}</strong>
        </button>
        <button
          className={`${styles.contextChip} context-chip`}
          type="button"
          aria-label="Integrity state"
        >
          <span className={`state-dot ${integrityStatus === "verified" ? "state-dot--ready" : ""}`} />
          <strong>Integrity {integrityStatus}</strong>
        </button>
      </div>

      <div className={`${styles.globalActions} global-actions`}>
        <button
          type="button"
          id="searchButton"
          className={`${styles.iconButton} icon-button`}
          onClick={onOpenSearch}
          aria-label="Open command search"
          title="Search (/)"
        >
          <span aria-hidden="true">⌕</span>
        </button>

        <button
          type="button"
          className={`${styles.attentionButton} attention-button`}
          onClick={onOpenInbox}
          aria-label={`Open approval inbox: ${inboxCount} approval`}
        >
          <span>Inbox</span>
          <strong>{inboxCount} approval</strong>
        </button>

        <button
          type="button"
          className={`${styles.activeWork} active-work`}
          onClick={onToggleEvidence}
          aria-label="Toggle active work evidence"
        >
          <span className="state-dot state-dot--running" />
          <span>Active work</span>
          <strong>None running</strong>
        </button>
      </div>
    </header>
  );
}
