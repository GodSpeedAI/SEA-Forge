import styles from "./AuthorityBoundaryPanel.module.css";

export interface AuthorityBoundaryPanelProps {
  actorName?: string;
  roleName?: string;
  sponsorName?: string;
  policyBundleRef?: string;
  allowedBoundaries?: string[];
  className?: string;
}

export function AuthorityBoundaryPanel({
  actorName = "Operator",
  roleName = "Administrator",
  sponsorName = "Human Sponsor",
  policyBundleRef = "default-policy-v1",
  allowedBoundaries = ["local-execution", "read-evidence", "commit-cases"],
  className = "",
}: AuthorityBoundaryPanelProps) {
  return (
    <section
      className={`${styles.panel} ${className}`.trim()}
      data-testid="authority-boundary-panel"
      aria-label="Authority and boundary context"
    >
      <h3 className={styles.title}>Authority & Boundary Context</h3>

      <div className={styles.grid}>
        <div className={styles.field}>
          <span className={styles.fieldLabel}>Actor</span>
          <span className={styles.fieldValue}>{actorName}</span>
        </div>
        <div className={styles.field}>
          <span className={styles.fieldLabel}>Role</span>
          <span className={styles.fieldValue}>{roleName}</span>
        </div>
        <div className={styles.field}>
          <span className={styles.fieldLabel}>Sponsor</span>
          <span className={styles.fieldValue}>{sponsorName}</span>
        </div>
        <div className={styles.field}>
          <span className={styles.fieldLabel}>Policy Bundle</span>
          <span className={styles.fieldValue}>{policyBundleRef}</span>
        </div>
      </div>

      <div className={styles.field}>
        <span className={styles.fieldLabel}>Permitted Operational Boundaries</span>
        <div className={styles.boundariesList}>
          {allowedBoundaries.map((b) => (
            <span key={b} className={styles.boundaryChip}>
              {b}
            </span>
          ))}
        </div>
      </div>
    </section>
  );
}
