import React from "react";
import styles from "./JourneyRibbon.module.css";

export const JOURNEY_STEPS = [
  "Readiness",
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
    <div className={styles.ribbon} aria-label="Governed journey">
      <strong>Governed Journey:</strong>
      {JOURNEY_STEPS.map((step, idx) => {
        const isActive = step.toLowerCase() === currentStep.toLowerCase();
        return (
          <React.Fragment key={step}>
            {idx > 0 && <span className={styles.arrow}>→</span>}
            <span className={`${styles.step} ${isActive ? styles.stepActive : ""}`}>
              {step}
            </span>
          </React.Fragment>
        );
      })}
    </div>
  );
}
