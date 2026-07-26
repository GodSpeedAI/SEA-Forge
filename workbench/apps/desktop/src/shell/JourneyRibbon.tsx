import styles from "./JourneyRibbon.module.css";

const GOVERNED_STEPS = [
  "Authority",
  "Domain",
  "Draft",
  "Preflight",
  "Execution",
  "Evidence",
  "Settlement",
];

export interface JourneyRibbonProps {
  currentStep?: string;
}

export function JourneyRibbon({ currentStep = "Readiness" }: JourneyRibbonProps) {
  return (
    <div className={`${styles.ribbon} journey-ribbon`} aria-label="Governed journey">
      <strong className={styles.stepActive}>{currentStep}</strong>
      {GOVERNED_STEPS.map((step) => (
          <span key={step} className={styles.step}>
            {step}
          </span>
      ))}
    </div>
  );
}
