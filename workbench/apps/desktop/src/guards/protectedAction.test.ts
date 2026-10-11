import { describe, expect, it } from "vitest";
import type { IdentityState } from "../hooks/useIdentity";
import type { ReadinessView } from "@sea-forge/contracts";
import { evaluateProtectedAction } from "./protectedAction";

const actor: IdentityState = {
  available: [{ actor_id: "operator_test", roles: ["operator"] }],
  actor: { actorId: "operator_test", role: "operator" },
  configured: true,
};

const readiness: ReadinessView = {
  overall: "ready_with_limitations",
  foundations: [
    {
      id: "self_model_integrity",
      name: "Self-model integrity",
      category: "foundation",
      status: "ready",
      reason: "",
      source: {
        ledger_id: "self-model",
        entry_id: "led_01",
        record_kind: "self_model_snapshot",
        record_id: "smsnap_01",
        digest: "sha256:snapshot",
        freshness: "current",
        rebuild_standing: "verified",
      },
      next_lawful_action: "Continue with a governed operation",
    },
  ],
  operational_capabilities: [
    {
      id: "local_governed_execution",
      name: "Local governed execution",
      category: "operational_capability",
      status: "ready",
      reason: "",
      source: undefined,
      next_lawful_action: "Create or inspect a governed case",
    },
    {
      id: "external_delegation",
      name: "External delegation",
      category: "operational_capability",
      status: "unknown",
      reason: "No committed endpoint verification record is available",
      source: undefined,
      next_lawful_action: "Run a governed endpoint probe",
    },
  ],
  recent_invalidations: [],
};

describe("evaluateProtectedAction", () => {
  it("allows a local action when only an unrelated external capability is unknown", () => {
    const result = evaluateProtectedAction(actor, readiness, {
      method: "case.create",
      capabilityId: "local_governed_execution",
      actionLabel: "case",
    });

    expect(result).toEqual({ isAllowed: true });
  });

  it("keeps an external action blocked when its own source standing is unknown", () => {
    const result = evaluateProtectedAction(actor, readiness, {
      method: "agent_run.start",
      capabilityId: "external_delegation",
      actionLabel: "delegation",
    });

    expect(result.isAllowed).toBe(false);
    expect(result.refusal?.code).toBe("readiness_blocked");
  });
});
