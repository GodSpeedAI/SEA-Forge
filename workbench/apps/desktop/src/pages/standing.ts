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

/**
 * Asset standing, across three deliberately disjoint vocabularies.
 *
 * `asset.list` carries each asset kind's own word verbatim (see
 * `crates/sea-forge-server/src/sfwp/assets.rs`), so this map holds all three.
 * The keys do not collide, and they must not be merged into a shared ladder:
 * an unprobed endpoint (`declared`) and a refused extension (`quarantined`) are
 * not the same condition, and one shared "unavailable" chip would say they are.
 *
 * Note what is *not* here: nothing maps to a blocked variant. Whether an asset
 * may be used is `blocking_reason`, a separate field rendered as a separate
 * chip — an extension can be `active` and still quarantined by trust level, and
 * a pill that folded the two would have to pick one truth to tell.
 */
export const ASSET_STANDING_VARIANT: Record<string, GovernedStatusVariant> = {
  // Plan templates: on disk is the only fact a template has.
  materialized: "committed",
  // Agent endpoints (`sea_forge_agent::EndpointStatus`). `declared` is the
  // floor: configured, and nothing more is claimed.
  declared: "unknown",
  probed: "degraded",
  demonstrated: "ready",
  // Extensions (`sea_forge_extension::ExtensionStatus`).
  active: "ready",
  disabled: "unsupported",
  quarantined: "blocked",
  superseded: "degraded",
};

/**
 * Delegation lifecycle standing (`sfwp::delegations::DelegationStanding`).
 *
 * `settled` maps to `finished`, not to `ready`: a settled delegation may have
 * been rejected, and the verdict is a separate column. Nothing here maps to a
 * success variant, because no lifecycle position is itself a success.
 *
 * `unresolved` is the one that matters. It means the server holds no handle and
 * no settlement exists — typically an episode that was in flight when the
 * server stopped. It is deliberately `unknown` rather than `cancelled` or
 * `failed`: those would assert an outcome the kernel never recorded.
 */
export const DELEGATION_STANDING_VARIANT: Record<string, GovernedStatusVariant> = {
  active: "running",
  cancellation_requested: "pending",
  settled: "finished",
  unresolved: "unknown",
};

/**
 * Who decided a resolved value (`sfwp::delegation_preview::ValueSource`).
 *
 * Not a standing ladder and deliberately not a pill: provenance is not better
 * or worse, it is only *different*, and a `ready`/`degraded` treatment would
 * quietly rank "the operator chose this" above "the endpoint chose it". These
 * render as plain qualifying text beside the value.
 */
export const VALUE_SOURCE_LABEL: Record<string, string> = {
  requested: "you asked for this",
  endpoint_default: "from the endpoint descriptor",
  cell_default: "from this cell's agent configuration",
  built_in: "nothing declared one; kernel fallback",
};

/** `snake_case` record vocabulary as sentence-case display text. */
export function humanize(value: string): string {
  return value.replace(/_/g, " ").replace(/^./, (c) => c.toUpperCase());
}
