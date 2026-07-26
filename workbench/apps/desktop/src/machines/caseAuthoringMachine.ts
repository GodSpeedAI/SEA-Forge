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

const commitActor = fromPromise<
  { requestId: string; response: CommitSuccess | RejectedAsStale | { error: string } },
  { draft: CaseDraft; precondition?: RecordDigest }
>(async ({ input }) => {
  const id = requestId();
  const response = await invoke<CommitSuccess | RejectedAsStale | { error: string }>(
    "sfwp_command",
    {
      command: {
        verb: "case_commit",
        template_ref: input.draft.templateRef,
        params: input.draft.params,
        request_id: id,
        ...(input.precondition
          ? { preconditions: { records: [input.precondition] } }
          : {}),
      },
    },
  );
  return { requestId: id, response };
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
      invoke: {
        src: "commitActor",
        input: ({ context }) => ({
          draft: context.draft,
          precondition: context.preflight?.precondition ?? undefined,
        }),
        onDone: [
          {
            guard: ({ event }) => isCommitSuccess(event.output.response),
            target: "committed",
            actions: assign({
              requestId: ({ event }) => event.output.requestId,
              commitResult: ({ event }) => event.output.response as CommitSuccess,
            }),
          },
          {
            guard: ({ event }) => isRejectedAsStale(event.output.response),
            target: "rejected_as_stale",
            actions: assign({
              requestId: ({ event }) => event.output.requestId,
              rejection: ({ event }) => event.output.response as RejectedAsStale,
            }),
          },
          {
            // A definitive (non-transport) error response — the roundtrip
            // completed, so this is not "ambiguous"; go back to preflight_ok
            // so the operator can inspect and retry deliberately.
            target: "preflight_ok",
            actions: assign({
              requestId: ({ event }) => event.output.requestId,
              transportError: ({ event }) =>
                "error" in event.output.response
                  ? String(event.output.response.error)
                  : "commit failed",
            }),
          },
        ],
        // The invoke() promise itself rejected (no response at all) — a real
        // transport ambiguity: the client cannot know whether the case was
        // created. Recovery is `request.get_status`, never a resubmit.
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
