import { describe, expect, it } from "vitest";
import { createActor, fromPromise, waitFor } from "xstate";
import {
  caseAuthoringMachine,
  type CaseDraft,
  type CommitSuccess,
  type RejectedAsStale,
} from "./caseAuthoringMachine";
import type { PreflightResult, RecordDigest } from "@sea-forge/contracts";

const draft: CaseDraft = { templateRef: "demo@0.1.0", params: {} };

const okPreflight: PreflightResult = {
  ok: true,
  template_ref: draft.templateRef,
  items: [{ plan_item_id: "item_01", name: "Item", item_kind: "SandboxedTask" }],
  precondition: { ref: "template:demo@0.1.0", expected_digest: "sha256:abc" },
};

type CommitResponse = CommitSuccess | RejectedAsStale | { error: string };
type CommitOutput = { requestId: string; response: CommitResponse };
type CommitInput = { draft: CaseDraft; precondition?: RecordDigest };

function withPreflight(result: PreflightResult) {
  return caseAuthoringMachine.provide({
    actors: {
      preflightActor: fromPromise(async () => result),
    },
  });
}

function mockCommit(machine: typeof caseAuthoringMachine, respond: () => Promise<CommitOutput>) {
  return machine.provide({
    actors: {
      commitActor: fromPromise<CommitOutput, CommitInput>(respond),
    },
  });
}

describe("caseAuthoringMachine", () => {
  it("draft -> validating -> preflight_ok on an ok preflight", async () => {
    const machine = withPreflight(okPreflight);
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));
    expect(actor.getSnapshot().context.preflight?.ok).toBe(true);
  });

  it("draft -> validating -> draft on a failed preflight (errors surfaced, no case created)", async () => {
    const machine = withPreflight({ ...okPreflight, ok: false, errors: ["bad plan"] });
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("draft") && Boolean(s.context.preflight));
    expect(actor.getSnapshot().context.preflight?.errors).toContain("bad plan");
  });

  it("preflight_ok -> committing -> committed on a successful commit", async () => {
    const machine = mockCommit(withPreflight(okPreflight), async () => ({
      requestId: "req-1",
      response: { case_id: "case_1", state: "active", exit_code: 0 },
    }));
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));
    actor.send({ type: "COMMIT" });
    await waitFor(actor, (s) => s.matches("committed"));
    expect(actor.getSnapshot().context.commitResult?.case_id).toBe("case_1");
  });

  it("proof scenario 7: rejected_as_stale routes back through validating, never straight to committing", async () => {
    const machine = mockCommit(withPreflight(okPreflight), async () => ({
      requestId: "req-2",
      response: {
        outcome: "rejected_as_stale",
        code: "precondition_failed",
        changed_records: [],
        next_actions: [],
      },
    }));
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));
    actor.send({ type: "COMMIT" });
    await waitFor(actor, (s) => s.matches("rejected_as_stale"));

    actor.send({ type: "RETRY" });
    // RETRY must land on `validating` (a fresh preflight), not `committing`.
    expect(actor.getSnapshot().value).toBe("validating");
  });

  it("proof scenario 6: a duplicate commit is unreachable from `ambiguous`", async () => {
    const machine = mockCommit(withPreflight(okPreflight), async () => {
      throw new Error("connection lost");
    });
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));
    actor.send({ type: "COMMIT" });
    await waitFor(actor, (s) => s.matches("ambiguous"));

    // No COMMIT transition is defined on `ambiguous` — sending it must be a
    // no-op, not a second `case.commit` call.
    actor.send({ type: "COMMIT" });
    expect(actor.getSnapshot().value).toBe("ambiguous");
  });

  it("ambiguous -> status_recovery -> committed recovers the outcome instead of resubmitting", async () => {
    const machine = mockCommit(withPreflight(okPreflight), async () => {
      throw new Error("connection lost");
    }).provide({
      actors: {
        statusRecoveryActor: fromPromise<{ status: string; outcome: unknown }, { requestId: string }>(
          async () => ({
            status: "completed",
            outcome: { case_id: "case_2", state: "active", exit_code: 0 },
          }),
        ),
      },
    });
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));
    actor.send({ type: "COMMIT" });
    await waitFor(actor, (s) => s.matches("ambiguous"));

    actor.send({ type: "RECOVER" });
    await waitFor(actor, (s) => s.matches("committed"));
    expect(actor.getSnapshot().context.commitResult?.case_id).toBe("case_2");
  });
});
