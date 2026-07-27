import type { GovernedStatusVariant } from "@sea-forge/ui-components";

/**
 * How the kernel's standing vocabularies render as pills.
 *
 * These maps are shared rather than copied per page because each one encodes a
 * governance rule, not a styling preference — and a rule with two copies is a
 * rule that will eventually be enforced in only one of them.
 */

/**
 * Execution standing. Never borrows settlement vocabulary.
 *
 * `completed` is deliberately `degraded`, not `ready`: execution finishing is
 * not the same as work being accepted, and a green pill here would read as
 * governed completion the kernel has not granted (epic invariant 2).
 */
export const EXECUTION_VARIANT: Record<string, GovernedStatusVariant> = {
  pending: "unknown",
  enabled: "unknown",
  active: "degraded",
  completed: "degraded",
  failed: "blocked",
  terminated: "blocked",
};

/** Only an accepting settlement record earns a success pill. */
export const SETTLEMENT_VARIANT: Record<string, GovernedStatusVariant> = {
  unsettled: "unknown",
  accepted: "ready",
  rejected: "blocked",
  escalated: "degraded",
};

export const CASE_STATE_VARIANT: Record<string, GovernedStatusVariant> = {
  active: "degraded",
  awaiting_approval: "degraded",
  completed: "ready",
  terminated: "blocked",
};

/**
 * Criterion standing keeps four distinct chips because it has four distinct
 * meanings. Collapsing `recorded` (observed evidence that never decides
 * acceptance, §10.6) or `unavailable` (the settlement said nothing) into a
 * shared "warning" would erase exactly the distinctions epic 12.3 exists to
 * preserve.
 */
export const CRITERION_VARIANT: Record<string, GovernedStatusVariant> = {
  satisfied: "ready",
  unsatisfied: "blocked",
  recorded: "committed",
  unavailable: "unknown",
};

/** Authority verdicts. An escalation is unresolved, not a failure. */
export const VERDICT_VARIANT: Record<string, GovernedStatusVariant> = {
  allow: "ready",
  deny: "blocked",
  escalate: "degraded",
  boundary: "degraded",
  degraded: "degraded",
};

/** `snake_case` record vocabulary as sentence-case display text. */
export function humanize(value: string): string {
  return value.replace(/_/g, " ").replace(/^./, (c) => c.toUpperCase());
}
