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

const staleRejection: RejectedAsStale = {
  outcome: "rejected_as_stale",
  code: "precondition_failed",
  changed_records: [],
  next_actions: [],
};

type CommitResponse = CommitSuccess | RejectedAsStale | { error: string };
type CommitInput = { draft: CaseDraft; requestId: string; precondition?: RecordDigest };

/** The reply `dispatch_bounded` actually sends when the 10s bound expires,
 * copied field for field from `bounded()` in `crates/sea-forge-server/src/lib.rs`.
 * The missing field is the one that matters: the server stops *waiting* for the
 * work, it does not cancel it, so it cannot certify `no_side_effect` and does
 * not claim to. A stub that added one would be testing a server that does not
 * exist. */
function serverTimeoutBody(requestId: string) {
  return {
    error: "request exceeded the 10s server timeout; the work was not cancelled",
    error_class: "request_timeout",
    timeout_seconds: 10,
    request_id: requestId,
    recover_with: "request.get_status",
  };
}

/** The `request_id_reused` refusal, likewise copied from the dedupe guard in
 * `lib.rs`. Decided above `tokio::spawn`, hence the certificate. */
function certifiedRefusalBody(requestId: string) {
  return {
    error:
      `request_id \`${requestId}\` was already used for a different operation ` +
      "or payload; issue a new id rather than re-using this one",
    error_class: "request_id_reused",
    no_side_effect: true,
  };
}

function withPreflight(result: PreflightResult) {
  return caseAuthoringMachine.provide({
    actors: {
      preflightActor: fromPromise(async () => result),
    },
  });
}

/** The mock receives the actor's `input`, because what SF-006 changed is what
 * is in that input: the request id now arrives from context instead of being
 * minted inside the promise. */
function mockCommit(
  machine: typeof caseAuthoringMachine,
  respond: (input: CommitInput) => Promise<CommitResponse>,
) {
  return machine.provide({
    actors: {
      commitActor: fromPromise<CommitResponse, CommitInput>(({ input }) => respond(input)),
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
      case_id: "case_1",
      state: "active",
      exit_code: 0,
    }));
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));
    actor.send({ type: "COMMIT" });
    await waitFor(actor, (s) => s.matches("committed"));
    expect(actor.getSnapshot().context.commitResult?.case_id).toBe("case_1");
  });

  it("proof scenario 7: rejected_as_stale routes back through validating, never straight to committing", async () => {
    const machine = mockCommit(withPreflight(okPreflight), async () => staleRejection);
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
    let asked: string | undefined;
    const machine = mockCommit(withPreflight(okPreflight), async () => {
      throw new Error("connection lost");
    }).provide({
      actors: {
        statusRecoveryActor: fromPromise<{ status: string; outcome: unknown }, { requestId: string }>(
          async ({ input }) => {
            asked = input.requestId;
            return {
              status: "completed",
              outcome: { case_id: "case_2", state: "active", exit_code: 0 },
            };
          },
        ),
      },
    });
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));
    actor.send({ type: "COMMIT" });
    const minted = actor.getSnapshot().context.requestId;
    await waitFor(actor, (s) => s.matches("ambiguous"));

    actor.send({ type: "RECOVER" });
    await waitFor(actor, (s) => s.matches("committed"));
    expect(actor.getSnapshot().context.commitResult?.case_id).toBe("case_2");
    // Recovery asked about the id the lost commit was sent under. Anything
    // else and the recovered outcome belongs to some other request.
    expect(asked).toBe(minted);
  });

  it("SF-006: the request id is in context before the commit actor settles, and is the one it sends", async () => {
    let sent: string | undefined;
    let release = () => {};
    const inFlight = new Promise<void>((resolve) => {
      release = resolve;
    });
    const machine = mockCommit(withPreflight(okPreflight), async (input) => {
      sent = input.requestId;
      await inFlight;
      return { case_id: "case_3", state: "active", exit_code: 0 };
    });
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));

    actor.send({ type: "COMMIT" });
    // The commit is still in flight — no onDone has run — yet the id already
    // exists. This is what makes it survivable.
    expect(actor.getSnapshot().value).toBe("committing");
    const minted = actor.getSnapshot().context.requestId;
    expect(minted).toMatch(/^req-case-commit-/);

    release();
    await waitFor(actor, (s) => s.matches("committed"));
    expect(sent).toBe(minted);
    expect(actor.getSnapshot().context.requestId).toBe(minted);
  });

  it("SF-006: the request id survives a rejected commit actor, so `ambiguous` stays recoverable", async () => {
    const machine = mockCommit(withPreflight(okPreflight), async () => {
      throw new Error("connection lost");
    });
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));

    actor.send({ type: "COMMIT" });
    const minted = actor.getSnapshot().context.requestId;
    expect(minted).toMatch(/^req-case-commit-/);

    await waitFor(actor, (s) => s.matches("ambiguous"));
    // The whole point of SF-006: the invoke promise rejected and took its
    // response with it, but not the id. `status_recovery` reads exactly this
    // field, so an undefined here is an unrecoverable ambiguity.
    expect(actor.getSnapshot().context.requestId).toBe(minted);
  });

  it("SF-006: each commit attempt mints its own id, because a stale retry sends a different payload", async () => {
    const sent: string[] = [];
    const machine = mockCommit(withPreflight(okPreflight), async (input) => {
      sent.push(input.requestId);
      return sent.length === 1
        ? staleRejection
        : { case_id: "case_4", state: "active", exit_code: 0 };
    });
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));
    actor.send({ type: "COMMIT" });
    await waitFor(actor, (s) => s.matches("rejected_as_stale"));

    actor.send({ type: "RETRY" });
    await waitFor(actor, (s) => s.matches("preflight_ok"));
    actor.send({ type: "COMMIT" });
    await waitFor(actor, (s) => s.matches("committed"));

    expect(sent).toHaveLength(2);
    // Reusing the first id would earn `request_id_reused` from the server: the
    // retry re-ran preflight, so its payload is not the one that id is bound to.
    expect(sent[0]).not.toBe(sent[1]);
    expect(actor.getSnapshot().context.requestId).toBe(sent[1]);
  });

  it("SF-006: the server timeout reply lands in `ambiguous`, because the commit may still land", async () => {
    let sent: string | undefined;
    const machine = mockCommit(withPreflight(okPreflight), async (input) => {
      sent = input.requestId;
      return serverTimeoutBody(input.requestId);
    });
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));

    actor.send({ type: "COMMIT" });
    const minted = actor.getSnapshot().context.requestId;
    await waitFor(actor, (s) => !s.matches("committing"));

    // `preflight_ok` is the regression: COMMIT there mints a *fresh* id, so the
    // resubmit reaches the server under an id it has never seen, dedupe has
    // nothing to match it against, and the case the timed-out work is still
    // writing becomes the second of two.
    expect(actor.getSnapshot().value).toBe("ambiguous");
    // Recovering by this id is the entire remedy, so it has to be the id the
    // lost commit was sent under and it has to still be here.
    expect(actor.getSnapshot().context.requestId).toBe(minted);
    expect(sent).toBe(minted);

    actor.send({ type: "COMMIT" });
    expect(actor.getSnapshot().value).toBe("ambiguous");
  });

  it("SF-006: an error that certifies `no_side_effect` still returns to `preflight_ok`", async () => {
    const machine = mockCommit(withPreflight(okPreflight), async (input) =>
      certifiedRefusalBody(input.requestId),
    );
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));

    actor.send({ type: "COMMIT" });
    await waitFor(actor, (s) => !s.matches("committing"));

    // The complement of the test above, and the reason it is a guard rather
    // than a blanket route to `ambiguous`: a refusal decided before the work
    // started leaves nothing to recover, so a new id is the right next move
    // and stranding the operator in `ambiguous` would be the wrong one.
    expect(actor.getSnapshot().value).toBe("preflight_ok");
    expect(actor.getSnapshot().context.transportError).toContain("already used");
  });

  it("SF-006: an uncertified error class this file has never seen is ambiguous too", async () => {
    const uncertified = {
      error: "the store went away mid-write",
      error_class: "some_error_added_next_quarter",
    };
    const machine = mockCommit(withPreflight(okPreflight), async () => uncertified);
    const actor = createActor(machine).start();
    actor.send({ type: "PREFLIGHT", draft });
    await waitFor(actor, (s) => s.matches("preflight_ok"));

    actor.send({ type: "COMMIT" });
    await waitFor(actor, (s) => !s.matches("committing"));

    // The machine keys on the absence of the certificate, not on a list of
    // known-dangerous classes. A list is only ever as current as its last
    // edit; this way a server error that forgets to certify itself is treated
    // as ambiguous by default rather than by remembering to add it here.
    expect(actor.getSnapshot().value).toBe("ambiguous");
  });
});
