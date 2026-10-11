import { useState } from "react";
import { Button } from "@astryxdesign/core/Button";
import styles from "./ProtectedActionButton.module.css";

export interface ProtectedActionButtonProps {
  label: string;
  onClick: () => void | Promise<void>;
  isAllowed?: boolean;
  disabledReason?: string;
  requiresConfirmation?: boolean;
  confirmationMessage?: string;
  preflightState?: "idle" | "evaluating" | "passed" | "failed";
  hasStanding?: boolean;
  variant?: "primary" | "secondary" | "danger";
  isLoading?: boolean;
  className?: string;
}

export function ProtectedActionButton({
  label,
  onClick,
  isAllowed = true,
  disabledReason,
  requiresConfirmation = false,
  confirmationMessage = "Are you sure you want to execute this governed operation?",
  preflightState = "idle",
  hasStanding = true,
  variant = "primary",
  isLoading = false,
  className = "",
}: ProtectedActionButtonProps) {
  const [confirming, setConfirming] = useState(false);
  const [executing, setExecuting] = useState(false);

  const isDisabled = !isAllowed || !hasStanding || preflightState === "failed" || isLoading || executing;

  const resolvedDisabledReason =
    disabledReason ??
    (!hasStanding
      ? "Actor lacks required standing for this decision"
      : preflightState === "failed"
      ? "Preflight evaluation failed"
      : !isAllowed
      ? "Operation not permitted by policy"
      : undefined);

  const handlePrimaryClick = async () => {
    if (isDisabled) return;
    if (requiresConfirmation && !confirming) {
      setConfirming(true);
      return;
    }

    try {
      setExecuting(true);
      await onClick();
    } finally {
      setExecuting(false);
      setConfirming(false);
    }
  };

  const handleCancelConfirm = () => {
    setConfirming(false);
  };

  const astryxVariant = variant === "danger" ? "primary" : variant;

  return (
    <div className={`${styles.wrapper} ${className}`.trim()} data-testid="protected-action-button">
      {confirming ? (
        <div className={styles.confirmBox}>
          <span className={styles.warningNotice}>{confirmationMessage}</span>
          <Button
            label="Confirm"
            variant="primary"
            isLoading={executing}
            onClick={handlePrimaryClick}
          />
          <Button
            label="Cancel"
            variant="secondary"
            onClick={handleCancelConfirm}
          />
        </div>
      ) : (
        <Button
          label={preflightState === "evaluating" ? "Preflight evaluating..." : label}
          variant={astryxVariant}
          isDisabled={isDisabled}
          isLoading={isLoading || executing || preflightState === "evaluating"}
          onClick={handlePrimaryClick}
        />
      )}
      {isDisabled && resolvedDisabledReason && (
        <span className={styles.reasonText} data-testid="disabled-reason">
          {resolvedDisabledReason}
        </span>
      )}
    </div>
  );
}
