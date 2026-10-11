import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { validateApprovalListResult, type ApprovalListResult } from "@sea-forge/contracts";
import { toError } from "./bridgeError";
import { queryGoverned } from "./governedQuery";
import { affectsApprovals } from "./eventKinds";
import { useGovernedEventInvalidation } from "./useGovernedEventInvalidation";
import { selectedActorId } from "./useIdentity";

/**
 * The approval inbox over `approval.list` and `approval.decide`.
 *
 * `approve`/`reject` were always reachable but required identifiers no method
 * could enumerate, so an approver had no lawful path to them. `approval.list`
 * closes that: the ids a decision needs now come from the protocol itself.
 */

export const APPROVALS_QUERY_KEY = ["approvals"] as const;

function fetchApprovals(caseId?: string): Promise<ApprovalListResult> {
  const query: { verb: "approval_list"; case_id?: string } = { verb: "approval_list" };
  if (caseId) query.case_id = caseId;
  return queryGoverned<ApprovalListResult>(query, validateApprovalListResult, "approval.list");
}

export type Verdict = "approve" | "reject";

/** Past tense per verdict — "reject" + "d" is not a word. */
export const VERDICT_PAST: Record<Verdict, string> = {
  approve: "approved",
  reject: "rejected",
};

export interface DecisionOutcome {
  approvalId: string;
  verdict: Verdict;
  /** True only when the server's response carried no error. */
  accepted: boolean;
  detail: string;
}

export function useApprovals(caseId?: string) {
  const queryClient = useQueryClient();
  const [deciding, setDeciding] = useState<string | undefined>();
  const [lastOutcome, setLastOutcome] = useState<DecisionOutcome | undefined>();

  const query = useQuery<ApprovalListResult, Error>({
    queryKey: [...APPROVALS_QUERY_KEY, caseId ?? null],
    queryFn: () => fetchApprovals(caseId),
    retry: false,
  });

  useGovernedEventInvalidation(affectsApprovals, APPROVALS_QUERY_KEY);

  const refresh = useCallback(() => {
    void queryClient.invalidateQueries({ queryKey: APPROVALS_QUERY_KEY });
  }, [queryClient]);

  /**
   * Resolve one approval.
   *
   * The row is never removed optimistically. An approval leaves the inbox only
   * when a re-read of `approval.list` no longer returns it — that is, when the
   * journal says it is decided. Optimistically hiding it would show the
   * operator a decision the kernel might have refused (a stale precondition, an
   * expired window), which is precisely the "no optimistic approval" rule.
   */
  const decide = useCallback(
    async (approvalId: string, targetCaseId: string, verdict: Verdict, note?: string) => {
      setDeciding(approvalId);
      try {
        const raw = await invoke<unknown>("sfwp_command", {
          command: {
            verb: "approval_decide",
            case_id: targetCaseId,
            approval_id: approvalId,
            decision: verdict,
            ...(note ? { note } : {}),
          },
          ...(selectedActorId() ? { actAs: selectedActorId() } : {}),
        });
        const body = raw as { error?: unknown } | undefined;
        const outcome: DecisionOutcome =
          typeof body?.error === "string"
            ? { approvalId, verdict, accepted: false, detail: body.error }
            : {
                approvalId,
                verdict,
                accepted: true,
                detail: `Recorded as ${VERDICT_PAST[verdict]}.`,
              };
        setLastOutcome(outcome);
        return outcome;
      } catch (reason) {
        const outcome: DecisionOutcome = {
          approvalId,
          verdict,
          accepted: false,
          detail: toError(reason).message,
        };
        setLastOutcome(outcome);
        return outcome;
      } finally {
        setDeciding(undefined);
        // Re-read either way: the journal is the authority on what happened,
        // including when the command failed.
        void queryClient.invalidateQueries({ queryKey: APPROVALS_QUERY_KEY });
      }
    },
    [queryClient],
  );

  return {
    approvals: query.data?.approvals ?? [],
    /**
     * Set when the journal itself could not be read. Distinct from an empty
     * inbox: nothing could be determined, rather than nothing is waiting.
     */
    unreadable: query.data?.unreadable,
    isLoading: query.isLoading,
    error: query.error ? toError(query.error) : null,
    deciding,
    lastOutcome,
    decide,
    refresh,
  };
}
