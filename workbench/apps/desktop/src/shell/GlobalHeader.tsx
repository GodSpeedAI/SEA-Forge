import styles from "./GlobalHeader.module.css";

/**
 * The always-visible governance context (epic 2.1: "I want to see the resolved
 * principal, actor type, role, identity source, and active cell context so that
 * I know who SEA Forge believes is acting").
 *
 * Every field here used to have a plausible default — `actorName = "Operator"`,
 * `integrityStatus = "verified"`, `inboxCount = 1` — and `AppShell` passed none
 * of them, so the bar asserted a verified integrity state, a named actor, and a
 * waiting approval on a cell that had reported none of those. A header is the
 * worst place for that: it is on screen for every surface, so one fabricated
 * default there is a claim repeated on every page.
 *
 * There are no defaults now. An absent value renders as absent.
 */
export interface GlobalHeaderProps {
  /** Resolved from `identity.get`; absent when this cell resolved no actor. */
  actorName?: string;
  roleName?: string;
  /** The cell this window is attached to, from the host's resolved socket. */
  cellName?: string;
  integrityStatus?: "verified" | "compromised" | "checking" | "unverified";
  /** Pending approvals; `undefined` while unknown, which is not zero. */
  inboxCount?: number;
  onOpenSearch?: () => void;
  onOpenInbox?: () => void;
  onToggleEvidence?: () => void;
}

/** Rendered wherever a value has no source. Never styled as a success state. */
const UNRESOLVED = "Unresolved";

export function GlobalHeader({
  actorName,
  roleName,
  cellName,
  integrityStatus,
  inboxCount,
  onOpenSearch,
  onOpenInbox,
  onToggleEvidence,
}: GlobalHeaderProps) {
  const actorLabel =
    actorName && roleName ? `${actorName} · ${roleName}` : (actorName ?? UNRESOLVED);
  const inboxLabel =
    inboxCount === undefined
      ? "Unknown"
      : `${inboxCount} approval${inboxCount === 1 ? "" : "s"}`;

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
          <span>{cellName ?? "No cell resolved"}</span>
        </div>
      </div>

      <div className={`${styles.globalContext} global-context`}>
        <button
          className={`${styles.contextChip} context-chip`}
          type="button"
          aria-label="Active actor and role"
          data-resolved={actorName ? "true" : "false"}
        >
          <span>Actor</span>
          <strong>{actorLabel}</strong>
        </button>
        <button
          className={`${styles.contextChip} context-chip`}
          type="button"
          aria-label="Active cell"
        >
          <span>Cell</span>
          <strong>{cellName ?? UNRESOLVED}</strong>
        </button>
        <button
          className={`${styles.contextChip} context-chip`}
          type="button"
          aria-label="Integrity state"
        >
          <span
            className={`state-dot ${integrityStatus === "verified" ? "state-dot--ready" : ""}`}
          />
          <strong>Integrity {integrityStatus ?? "unknown"}</strong>
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
          aria-label={`Open approval inbox: ${inboxLabel}`}
        >
          <span>Inbox</span>
          <strong>{inboxLabel}</strong>
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
