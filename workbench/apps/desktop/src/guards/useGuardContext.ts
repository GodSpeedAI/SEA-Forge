import { useIdentity } from "../hooks/useIdentity";
import { useReadiness } from "../hooks/useReadiness";
import type { GuardContext, GuardId } from "./guards";
import type { ReadinessView } from "@sea-forge/contracts";

/**
 * Assemble the guard context from the cell's own answers.
 *
 * The rule for every field: include it only when a source reported it. A field
 * left `undefined` makes its guard `indeterminate`, which is the honest state
 * for the several guards this kernel has no verb behind yet (sponsorship,
 * disclosure scope, the compatibility matrix). Filling those in with plausible
 * values is what `mockGuardContext` did, and it made six guards report success
 * on no evidence at all.
 */

/**
 * The guards a route is gated on.
 *
 * These three are exactly the guards with a real source today: the cell comes
 * from the host's resolved socket, the actor from `identity.get`, and readiness
 * from `readiness.get`. The rest are still evaluated and displayed — they are
 * simply not grounds to refuse a route while nothing can answer them.
 */
export const BLOCKING_GUARDS: readonly GuardId[] = ["G1", "G2", "G9"];

/** The readiness item carrying integrity state, by its stable kernel id. */
const INTEGRITY_ITEM_ID = "self_model_integrity";

/**
 * Map the kernel's overall verdict onto the guard vocabulary.
 *
 * `stale` is deliberately unmapped: the kernel reserves it but never derives
 * it, and inventing a guard reading for a value that cannot occur would be a
 * claim about a state this cell has never been in.
 */
function readinessStateOf(view: ReadinessView): GuardContext["readinessState"] {
  switch (view.overall) {
    case "ready":
      return "ready";
    case "ready_with_limitations":
      return "ready_degraded";
    case "blocked":
      return "blocked";
    case "integrity_halted":
      return "integrity_halted";
    default:
      return undefined;
  }
}

/** Integrity, read off the foundation item that reports it. */
function integrityOf(view: ReadinessView): GuardContext["integrityStatus"] {
  const item = view.foundations.find((entry) => entry.id === INTEGRITY_ITEM_ID);
  if (!item) return undefined;
  switch (item.status) {
    case "ready":
      return "verified";
    case "blocked":
      return "compromised";
    case "degraded":
      return "unverified";
    case "unknown":
      return "checking";
    default:
      return undefined;
  }
}

export function useGuardContext(): GuardContext {
  const { identity, cellId } = useIdentity();
  const { query } = useReadiness();
  const readiness = query.data;

  return {
    cellId,
    actorId: identity?.actor?.actorId,
    actorRole: identity?.actor?.role,
    // Sponsorship, disclosure scope, resource lookup, and the compatibility
    // matrix have no SFWP verb behind them in this cell. Left absent on
    // purpose: `indeterminate` is the truthful verdict, and any value here
    // would be one this renderer made up.
    readinessState: readiness ? readinessStateOf(readiness) : undefined,
    integrityStatus: readiness ? integrityOf(readiness) : undefined,
  };
}
