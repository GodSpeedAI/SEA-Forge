import styles from "./AvailabilityLadder.module.css";
import { GovernedStatusPill } from "./GovernedStatusPill";

export interface CapabilityLevel {
  level: number;
  name: string;
  status: "ready" | "degraded" | "blocked";
  spendableProofPath: string;
}

export interface AvailabilityLadderProps {
  currentLevel: number;
  levels?: CapabilityLevel[];
  className?: string;
}

export const DEFAULT_LEVELS: CapabilityLevel[] = [
  { level: 1, name: "Local Kernel Execution", status: "ready", spendableProofPath: "proofs/P1-local-sync" },
  { level: 2, name: "Local SFWP Transport", status: "ready", spendableProofPath: "proofs/P2-sfwp-socket" },
  { level: 3, name: "External Agent Delegation", status: "degraded", spendableProofPath: "proofs/P3-acp-mediation" },
  { level: 4, name: "Federated Peer Settlement", status: "blocked", spendableProofPath: "proofs/P4-federation-proof" },
];

export function AvailabilityLadder({
  currentLevel,
  levels = DEFAULT_LEVELS,
  className = "",
}: AvailabilityLadderProps) {
  return (
    <section
      className={`${styles.container} ${className}`.trim()}
      data-testid="availability-ladder"
      aria-label="Capability availability ladder"
    >
      <h3 className={styles.title}>Capability Availability Ladder</h3>

      <div className={styles.ladder}>
        {levels.map((item) => {
          const isActive = item.level === currentLevel;
          return (
            <div
              key={item.level}
              className={`${styles.levelItem} ${isActive ? styles.activeLevel : ""}`}
            >
              <div>
                <span className={styles.levelName}>
                  L{item.level}: {item.name} {isActive ? "(Current State)" : ""}
                </span>
                <div className={styles.proofPath}>Proof path: {item.spendableProofPath}</div>
              </div>
              <GovernedStatusPill variant={item.status} />
            </div>
          );
        })}
      </div>
    </section>
  );
}
