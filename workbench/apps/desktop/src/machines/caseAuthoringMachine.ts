import { assign, fromPromise, setup } from "xstate";
import { invoke } from "@tauri-apps/api/core";
import {
  validatePreflightResult,
  type PreflightResult,
  type RecordDigest,
} from "@sea-forge/contracts";

/**
 * Case authoring: draft -> preflight -> commit (Task 6).
 *
 * `preflight_ok` and `committing` are the only states with a `COMMIT`
 * transition. `ambiguous` (a lost/uncertain response to `case.commit`) only
 * exposes `RECOVER` -> `status_recovery`; it has no `COMMIT` handler, so a
 * duplicate commit is structurally unreachable from `ambiguous` — proven by
 * `caseAuthoringMachine.test.ts`, not just documented here.
 */

export interface CaseDraft {
  templateRef: string;
  params: Record<string, string>;
}

/** The ad hoc `case.commit` success shape (`{case_id, state, exit_code}`).
 * Not a generated contract: the server's `commit_plan` response is the same
 * untyped `serde_json::json!` shape `Submit` has always returned — no schema
 * exists server-side either, so inventing one here would be a fabricated
 * contract, not a real one. */
export interface CommitSuccess {
  case_id: string;
  state: string;
  exit_code: number;
}

export interface RejectedAsStale {
  outcome: "rejected_as_stale";
  code: "precondition_failed";
  changed_records: unknown[];
  next_actions: string[];
}

function isRejectedAsStale(value: unknown): value is RejectedAsStale {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as { outcome?: unknown }).outcome === "rejected_as_stale"
  );
}

function isCommitSuccess(value: unknown): value is CommitSuccess {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as { case_id?: unknown }).case_id === "string"
  );
}

interface CaseAuthoringContext {
  draft: CaseDraft;
  preflight?: PreflightResult;
  requestId?: string;
  commitResult?: CommitSuccess;
  rejection?: RejectedAsStale;
  transportError?: string;
}

type CaseAuthoringEvent =
  | { type: "PREFLIGHT"; draft: CaseDraft }
  | { type: "COMMIT" }
  | { type: "RETRY" }
  | { type: "RECOVER" }
  | { type: "RESET" };

const preflightActor = fromPromise<PreflightResult, { draft: CaseDraft }>(
  async ({ input }) => {
    const raw = await invoke<unknown>("sfwp_query", {
      query: {
        verb: "case_preflight",
        template_ref: input.draft.templateRef,
        params: input.draft.params,
      },
    });
    if (!validatePreflightResult(raw)) {
      const detail = validatePreflightResult.errors
        ?.map((e) => `${e.instancePath || "(root)"} ${e.message ?? ""}`.trim())
        .join("; ");
      throw new Error(`case.preflight response failed contract validation: ${detail}`);
    }
    return raw;
  },
);

function requestId(): string {
  return `req-case-commit-${Math.random().toString(36).slice(2)}-${Date.now()}`;
}

/**
 * Whether an error response certifies that nothing was mutated.
 *
 * Every refusal the server makes *before* dispatch carries `no_side_effect`.
 * The 10s `request_timeout` deliberately does not: `dispatch_bounded` stops
 * waiting for the work, it does not cancel it, so the commit may still land
 * after the reply. Keying on the absence of the flag rather than on any
 * particular `error_class` means a server error added later that forgets to
 * certify itself is treated as ambiguous — the safe side — by default.
 */
function provesNoSideEffect(value: unknown): boolean {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as { no_side_effect?: unknown }).no_side_effect === true
  );
}

function errorText(value: unknown): string {
  return typeof value === "object" && value !== null && "error" in value
    ? String((value as { error: unknown }).error)
    : "commit failed";
}

/** The commit actor is handed its id; it never mints one. An id that only
 * exists inside the promise dies with the promise, and `ambiguous` has nothing
 * left to ask `request.get_status` about. */
const commitActor = fromPromise<
  CommitSuccess | RejectedAsStale | { error: string },
  { draft: CaseDraft; requestId: string; precondition?: RecordDigest }
>(async ({ input }) => {
  return await invoke<CommitSuccess | RejectedAsStale | { error: string }>(
    "sfwp_command",
    {
      command: {
        verb: "case_commit",
        template_ref: input.draft.templateRef,
        params: input.draft.params,
        request_id: input.requestId,
        ...(input.precondition
          ? { preconditions: { records: [input.precondition] } }
          : {}),
      },
    },
  );
});

const statusRecoveryActor = fromPromise<
  { status: string; outcome: unknown },
  { requestId: string }
>(async ({ input }) => {
  const response = await invoke<{ status: string; outcome: unknown }>(
    "sfwp_request_status",
    { requestId: input.requestId },
  );
  return response;
});

export const caseAuthoringMachine = setup({
  types: {} as {
    context: CaseAuthoringContext;
    events: CaseAuthoringEvent;
  },
  actors: { preflightActor, commitActor, statusRecoveryActor },
}).createMachine({
  id: "caseAuthoring",
  initial: "draft",
  context: { draft: { templateRef: "", params: {} } },
  states: {
    draft: {
      on: {
        PREFLIGHT: {
          target: "validating",
          actions: assign({ draft: ({ event }) => event.draft }),
        },
      },
    },
    validating: {
      invoke: {
        src: "preflightActor",
        input: ({ context }) => ({ draft: context.draft }),
        onDone: [
          {
            guard: ({ event }) => event.output.ok,
            target: "preflight_ok",
            actions: assign({ preflight: ({ event }) => event.output }),
          },
          {
            target: "draft",
            actions: assign({ preflight: ({ event }) => event.output }),
          },
        ],
        onError: {
          target: "draft",
          actions: assign({
            transportError: ({ event }) => String(event.error),
          }),
        },
      },
    },
    preflight_ok: {
      on: {
        // Re-running preflight (e.g. after editing params) always re-validates
        // before allowing another commit attempt.
        PREFLIGHT: {
          target: "validating",
          actions: assign({ draft: ({ event }) => event.draft }),
        },
        COMMIT: "committing",
      },
    },
    committing: {
      // Minted on entry, so the id is machine state before the actor exists and
      // survives the actor's rejection. A fresh one per attempt is safe because
      // both routes back into this state have ruled out a pending mutation:
      // `validating` re-runs preflight and pins a new digest, and `preflight_ok`
      // is now only reachable from a refusal that certified no side effect. The
      // ambiguous routes deliberately have no COMMIT at all.
      entry: assign({ requestId: () => requestId() }),
      invoke: {
        src: "commitActor",
        input: ({ context }) => ({
          draft: context.draft,
          requestId: context.requestId ?? "",
          precondition: context.preflight?.precondition ?? undefined,
        }),
        onDone: [
          {
            guard: ({ event }) => isCommitSuccess(event.output),
            target: "committed",
            actions: assign({
              commitResult: ({ event }) => event.output as CommitSuccess,
            }),
          },
          {
            guard: ({ event }) => isRejectedAsStale(event.output),
            target: "rejected_as_stale",
            actions: assign({
              rejection: ({ event }) => event.output as RejectedAsStale,
            }),
          },
          {
            // A refusal that certified it changed nothing. Only here is it
            // safe to go back to preflight_ok, where COMMIT mints a new id:
            // nothing was written under the old one.
            guard: ({ event }) => provesNoSideEffect(event.output),
            target: "preflight_ok",
            actions: assign({
              transportError: ({ event }) => errorText(event.output),
            }),
          },
          {
            // An error the server did not certify as effect-free — above all
            // `request_timeout`, whose work explicitly keeps running. Getting
            // a reply is not the same as knowing the outcome, so this is the
            // same ambiguity as no reply at all and recovers the same way.
            target: "ambiguous",
            actions: assign({
              transportError: ({ event }) => errorText(event.output),
            }),
          },
        ],
        // The invoke() promise itself rejected (no response at all) — a real
        // transport ambiguity: the client cannot know whether the case was
        // created. Recovery is `request.get_status` on the id context still
        // holds, never a resubmit.
        onError: {
          target: "ambiguous",
          actions: assign({
            transportError: ({ event }) => String(event.error),
          }),
        },
      },
    },
    rejected_as_stale: {
      on: {
        // Repair path: re-run preflight (fresh digest), never re-commit
        // straight from a stale rejection.
        RETRY: "validating",
      },
    },
    ambiguous: {
      // Deliberately no `COMMIT` transition here — see the file-level comment.
      on: {
        RECOVER: "status_recovery",
      },
    },
    status_recovery: {
      invoke: {
        src: "statusRecoveryActor",
        input: ({ context }) => ({ requestId: context.requestId ?? "" }),
        onDone: [
          {
            guard: ({ event }) => isCommitSuccess(event.output.outcome),
            target: "committed",
            actions: assign({ commitResult: ({ event }) => event.output.outcome as CommitSuccess }),
          },
          {
            guard: ({ event }) => isRejectedAsStale(event.output.outcome),
            target: "rejected_as_stale",
            actions: assign({ rejection: ({ event }) => event.output.outcome as RejectedAsStale }),
          },
          {
            // Still pending/unknown — stay recoverable, never resubmit.
            target: "ambiguous",
          },
        ],
        onError: "ambiguous",
      },
    },
    committed: {
      type: "final",
    },
  },
  on: {
    RESET: { target: ".draft" },
  },
});
