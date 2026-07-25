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

export interface GuardContext {
  cellId?: string;
  actorId?: string;
  actorRole?: string;
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
  passed: boolean;
  reason?: string;
  repairRoute?: string;
  repairLabel?: string;
}

export function evaluateGuard(guardId: GuardId, context: GuardContext): GuardResult {
  switch (guardId) {
    case "G1":
      return {
        guardId: "G1",
        name: "G1 Cell Context",
        passed: Boolean(context.cellId),
        reason: context.cellId ? undefined : "Active cell context could not be resolved.",
        repairRoute: "/readiness",
        repairLabel: "Select active cell",
      };
    case "G2":
      return {
        guardId: "G2",
        name: "G2 Identity",
        passed: Boolean(context.actorId && context.actorRole),
        reason: context.actorId && context.actorRole ? undefined : "Attributable actor identity and role not resolved.",
        repairRoute: "/admin",
        repairLabel: "Resolve identity",
      };
    case "G3":
      return {
        guardId: "G3",
        name: "G3 Sponsor",
        passed: Boolean(context.sponsorId),
        reason: context.sponsorId ? undefined : "Automated actor lacks an eligible sponsor.",
        repairRoute: "/admin",
        repairLabel: "Assign sponsor",
      };
    case "G4":
      return {
        guardId: "G4",
        name: "G4 Policy",
        passed: Boolean(context.policyLoaded && context.policyHash),
        reason: context.policyLoaded && context.policyHash ? undefined : "Applicable policy bundle is not loaded or hash-addressed.",
        repairRoute: "/readiness",
        repairLabel: "Inspect policy bundle",
      };
    case "G5":
      return {
        guardId: "G5",
        name: "G5 Integrity",
        passed: context.integrityStatus === "verified",
        reason: context.integrityStatus === "verified" ? undefined : `Integrity state is ${context.integrityStatus ?? "unverified"}.`,
        repairRoute: "/evidence",
        repairLabel: "Inspect integrity ledger",
      };
    case "G6":
      return {
        guardId: "G6",
        name: "G6 Resource",
        passed: context.resourceFound !== false,
        reason: context.resourceFound !== false ? undefined : "Requested record or asset does not exist or is not visible.",
        repairRoute: "/assets",
        repairLabel: "Browse assets",
      };
    case "G7":
      return {
        guardId: "G7",
        name: "G7 Disclosure",
        passed: context.disclosurePermitted !== false,
        reason: context.disclosurePermitted !== false ? undefined : "Retrieval scope is restricted before data access.",
        repairRoute: "/evidence",
        repairLabel: "Request disclosure scope",
      };
    case "G8":
      return {
        guardId: "G8",
        name: "G8 Compatibility",
        passed: context.compatibilityOk !== false,
        reason: context.compatibilityOk !== false ? undefined : "Model/template/endpoint versions are incompatible.",
        repairRoute: "/models",
        repairLabel: "Check compatibility matrix",
      };
    case "G9":
      return {
        guardId: "G9",
        name: "G9 Readiness",
        passed: context.readinessState === "ready" || context.readinessState === "ready_degraded",
        reason:
          context.readinessState === "ready" || context.readinessState === "ready_degraded"
            ? undefined
            : `Subsystem readiness state is ${context.readinessState ?? "blocked"}.`,
        repairRoute: "/readiness",
        repairLabel: "Open readiness console",
      };
    case "G10":
      return {
        guardId: "G10",
        name: "G10 Standing",
        passed: context.standingOk !== false,
        reason: context.standingOk !== false ? undefined : "Actor lacks standing for this approval or settlement decision.",
        repairRoute: "/inbox",
        repairLabel: "Check standing in inbox",
      };
  }
}
