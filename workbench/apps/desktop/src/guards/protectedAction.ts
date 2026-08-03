import type { ReadinessView } from "@sea-forge/contracts";
import type { IdentityState } from "../hooks/useIdentity";

export interface ProtectedOperation {
  method: string;
  capabilityId?: string;
  actionLabel: string;
  /** Some controls have their own canonical server preconditions, not readiness. */
  requiresReadiness?: boolean;
}

export interface ProtectedActionRefusal {
  code: "identity_unresolved" | "readiness_unavailable" | "readiness_blocked";
  message: string;
  unchangedEffect: string;
  repairRoute: string;
  repairLabel: string;
}

export interface ProtectedActionDecision {
  isAllowed: boolean;
  refusal?: ProtectedActionRefusal;
}

/**
 * A source-backed affordance guard, not an authority decision.
 *
 * It answers whether a renderer may present one protected action as currently
 * spendable. The server remains the sole authority enforcement point. Keeping
 * this guard client-side and conservative prevents a ready capability from
 * being presented as an invitation to create work when `identity.get` cannot
 * attribute that work to an actor.
 */
export function evaluateProtectedAction(
  identity: IdentityState | undefined,
  readiness: ReadinessView | undefined,
  operation: ProtectedOperation,
): ProtectedActionDecision {
  if (!identity?.actor) {
    const detail = identity?.refusal?.message ?? "This cell has not resolved an actor for this connection.";
    return {
      isAllowed: false,
      refusal: {
        code: "identity_unresolved",
        message: `identity_unresolved: ${detail}`,
        unchangedEffect: `No ${operation.actionLabel.toLowerCase()} will be created until a server-derived actor is resolved.`,
        repairRoute: "/admin",
        repairLabel: "Inspect identity bindings",
      },
    };
  }

  if (operation.requiresReadiness !== false && !readiness) {
    return {
      isAllowed: false,
      refusal: {
        code: "readiness_unavailable",
        message: "Readiness projection unavailable.",
        unchangedEffect: `No ${operation.actionLabel.toLowerCase()} will be created while current readiness is unavailable.`,
        repairRoute: "/readiness",
        repairLabel: "Run readiness checks",
      },
    };
  }

  if (operation.requiresReadiness === false) return { isAllowed: true };

  // An operation depends on every foundation but only its named capability.
  // An unproven external endpoint must not falsely block a local case, just as
  // a ready local kernel must not authorize an external delegation.
  const blocker = readiness!.foundations.find((item) => item.status !== "ready");
  const requiredCapability = readiness!.operational_capabilities.find(
    (item) => item.id === operation.capabilityId,
  );
  if (blocker || requiredCapability?.status !== "ready") {
    return {
      isAllowed: false,
      refusal: {
        code: "readiness_blocked",
        message:
          blocker?.reason ||
          `The ${operation.capabilityId ?? "required"} capability is not currently ready for ${operation.method}.`,
        unchangedEffect: `No ${operation.actionLabel.toLowerCase()} will be created until readiness is restored.`,
        repairRoute: "/readiness",
        repairLabel: "Inspect readiness",
      },
    };
  }

  return { isAllowed: true };
}
