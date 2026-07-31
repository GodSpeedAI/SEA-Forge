/**
 * The ten governance guards, evaluated against what this cell actually
 * reported.
 *
 * Every guard used to answer `true` or `false`, and `false` was the answer both
 * for "the cell says no" and for "nothing told us". Combined with the
 * fabricated `mockGuardContext` in `router.tsx` — which asserted a verified
 * integrity state, a loaded policy digest, and an actor id that named no one —
 * the result was a surface that displayed governance conclusions it had no
 * evidence for. Several guards also read `x !== false`, so an absent input
 * *passed*: a permissive fallback wearing a governance label.
 *
 * So a guard now returns one of four verdicts, and the two that used to be
 * conflated stay apart (epic invariant 6, `unknown ≠ unavailable`):
 *
 * - `passed` — a source answered, and the answer satisfies the guard.
 * - `failed` — a source answered, and the answer does not.
 * - `indeterminate` — no source answered. Not a failure, and *never* a pass.
 * - `not_applicable` — the guard does not govern this context at all.
 *
 * These guards are an operator-facing pre-flight, never the enforcement point.
 * The server refuses protected work on its own authority regardless of what any
 * of this renders (see `identity::is_protected`), which is why an
 * `indeterminate` guard does not block a route: blocking on absent evidence
 * would make surfaces unreachable without making anything safer, while
 * *claiming* absent evidence would be the fabrication this replaces.
 */

export type GuardId =
  | "G1"
  | "G2"
  | "G3"
  | "G4"
  | "G5"
  | "G6"
  | "G7"
  | "G8"
  | "G9"
  | "G10";

export type GuardVerdict = "passed" | "failed" | "indeterminate" | "not_applicable";

export interface GuardContext {
  /** The cell this window is attached to, from the host's resolved socket. */
  cellId?: string;
  actorId?: string;
  actorRole?: string;
  /**
   * Sponsor for an automated actor. Human operators need none, which is why
   * G3 reports `not_applicable` rather than passing them.
   */
  sponsorId?: string;
  policyLoaded?: boolean;
  policyHash?: string;
  integrityStatus?: "verified" | "compromised" | "checking" | "unverified";
  resourceId?: string;
  resourceFound?: boolean;
  disclosurePermitted?: boolean;
  compatibilityOk?: boolean;
  readinessState?: "ready" | "ready_degraded" | "blocked" | "integrity_halted";
  standingOk?: boolean;
}

export interface GuardResult {
  guardId: GuardId;
  name: string;
  verdict: GuardVerdict;
  /**
   * `verdict === "passed"`. Deliberately false for `indeterminate`: a caller
   * that only asks "did this pass" must not read absent evidence as consent.
   */
  passed: boolean;
  reason?: string;
  repairRoute?: string;
  repairLabel?: string;
}

/** Roles that act on their own behalf and therefore need no sponsor. */
const SELF_SPONSORING_ROLES = new Set(["operator", "system"]);

function result(
  guardId: GuardId,
  name: string,
  verdict: GuardVerdict,
  reason: string | undefined,
  repairRoute: string,
  repairLabel: string,
): GuardResult {
  return {
    guardId,
    name,
    verdict,
    passed: verdict === "passed",
    reason,
    repairRoute,
    repairLabel,
  };
}

/**
 * Resolve a guard whose evidence is a single optional value.
 *
 * `undefined` is `indeterminate` before `satisfied` is ever consulted, which is
 * the whole point: it is not possible to write a guard here that reads a
 * missing input as a pass.
 */
function fromEvidence<T>(
  guardId: GuardId,
  name: string,
  value: T | undefined,
  satisfied: (value: T) => boolean,
  describe: { missing: string; unsatisfied: (value: T) => string },
  repairRoute: string,
  repairLabel: string,
): GuardResult {
  if (value === undefined) {
    return result(guardId, name, "indeterminate", describe.missing, repairRoute, repairLabel);
  }
  return satisfied(value)
    ? result(guardId, name, "passed", undefined, repairRoute, repairLabel)
    : result(guardId, name, "failed", describe.unsatisfied(value), repairRoute, repairLabel);
}

export function evaluateGuard(guardId: GuardId, context: GuardContext): GuardResult {
  switch (guardId) {
    case "G1":
      return fromEvidence(
        "G1",
        "G1 Cell Context",
        context.cellId,
        (cellId) => cellId.length > 0,
        {
          missing: "The host has not reported which cell this window is attached to.",
          unsatisfied: () => "Active cell context could not be resolved.",
        },
        "/readiness",
        "Select active cell",
      );

    case "G2": {
      // Both halves come from `identity.get`, so they are absent together.
      // Reported as one condition because "who" and "acting as what" are a
      // single attribution — half of it is not a weaker attribution, it is none.
      if (context.actorId === undefined || context.actorRole === undefined) {
        return result(
          "G2",
          "G2 Identity",
          "indeterminate",
          "This cell has not resolved an actor for this connection.",
          "/admin",
          "Inspect identity bindings",
        );
      }
      return context.actorId && context.actorRole
        ? result("G2", "G2 Identity", "passed", undefined, "/admin", "Inspect identity bindings")
        : result(
            "G2",
            "G2 Identity",
            "failed",
            "Attributable actor identity and role not resolved.",
            "/admin",
            "Inspect identity bindings",
          );
    }

    case "G3": {
      // Sponsorship governs automated actors. A human operator acting for
      // themselves is outside this guard's scope, and reporting them as
      // `passed` would imply a sponsor check ran and succeeded.
      if (context.actorRole && SELF_SPONSORING_ROLES.has(context.actorRole.toLowerCase())) {
        return result(
          "G3",
          "G3 Sponsor",
          "not_applicable",
          `A ${context.actorRole} acts on its own behalf and requires no sponsor.`,
          "/admin",
          "Assign sponsor",
        );
      }
      return fromEvidence(
        "G3",
        "G3 Sponsor",
        context.sponsorId,
        (sponsorId) => sponsorId.length > 0,
        {
          missing: "This cell does not record sponsorship, so eligibility cannot be determined.",
          unsatisfied: () => "Automated actor lacks an eligible sponsor.",
        },
        "/admin",
        "Assign sponsor",
      );
    }

    case "G4": {
      if (context.policyLoaded === undefined && context.policyHash === undefined) {
        return result(
          "G4",
          "G4 Policy",
          "indeterminate",
          "No source reports which policy bundle governs this cell.",
          "/readiness",
          "Inspect policy bundle",
        );
      }
      return context.policyLoaded && context.policyHash
        ? result("G4", "G4 Policy", "passed", undefined, "/readiness", "Inspect policy bundle")
        : result(
            "G4",
            "G4 Policy",
            "failed",
            "Applicable policy bundle is not loaded or hash-addressed.",
            "/readiness",
            "Inspect policy bundle",
          );
    }

    case "G5":
      return fromEvidence(
        "G5",
        "G5 Integrity",
        context.integrityStatus,
        (status) => status === "verified",
        {
          missing: "Integrity state has not been reported by this cell.",
          unsatisfied: (status) => `Integrity state is ${status}.`,
        },
        "/evidence",
        "Inspect integrity ledger",
      );

    case "G6":
      return fromEvidence(
        "G6",
        "G6 Resource",
        context.resourceFound,
        (found) => found,
        {
          missing: "No resource lookup has been performed for this context.",
          unsatisfied: () => "Requested record or asset does not exist or is not visible.",
        },
        "/assets",
        "Browse assets",
      );

    case "G7":
      return fromEvidence(
        "G7",
        "G7 Disclosure",
        context.disclosurePermitted,
        (permitted) => permitted,
        {
          missing: "This cell does not evaluate disclosure scope, so none can be asserted.",
          unsatisfied: () => "Retrieval scope is restricted before data access.",
        },
        "/evidence",
        "Request disclosure scope",
      );

    case "G8":
      return fromEvidence(
        "G8",
        "G8 Compatibility",
        context.compatibilityOk,
        (ok) => ok,
        {
          missing: "No compatibility matrix has been consulted for this context.",
          unsatisfied: () => "Model/template/endpoint versions are incompatible.",
        },
        "/models",
        "Check compatibility matrix",
      );

    case "G9":
      return fromEvidence(
        "G9",
        "G9 Readiness",
        context.readinessState,
        (state) => state === "ready" || state === "ready_degraded",
        {
          missing: "Readiness could not be read from this cell.",
          unsatisfied: (state) => `Subsystem readiness state is ${state}.`,
        },
        "/readiness",
        "Open readiness console",
      );

    case "G10":
      return fromEvidence(
        "G10",
        "G10 Standing",
        context.standingOk,
        (ok) => ok,
        {
          // Standing is decided server-side at the moment of decision
          // (separation of duty is compared against the ledger). There is no
          // pre-check verb, so the honest answer here is that we do not know.
          missing: "Standing is evaluated when a decision is submitted, not before it.",
          unsatisfied: () => "Actor lacks standing for this approval or settlement decision.",
        },
        "/inbox",
        "Check standing in inbox",
      );
  }
}
