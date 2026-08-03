import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { validateDelegationListResult, type DelegationListResult } from "@sea-forge/contracts";
import { toError } from "./bridgeError";
import { queryGoverned } from "./governedQuery";
import { affectsDelegations } from "./eventKinds";
import { useGovernedEventInvalidation } from "./useGovernedEventInvalidation";
import { selectedActorId } from "./useIdentity";

/**
 * The delegation roster over `delegation.list`, and the cancel command that
 * finally has targets (epic 9.7).
 *
 * `cancel_delegation` has been reachable since M12 and the host bridge has
 * carried `SfwpCommand::CancelDelegation` since Task 3 — but nothing could say
 * *which* delegations existed, which is the same shape the `approval.decide`
 * gap had before `approval.list`. A capability whose targets are undiscoverable
 * is not a capability an operator has.
 *
 * Unlike the asset catalog, this view *is* event-invalidated: both
 * `agent_run.*` kinds move a row, and a stale roster here would offer a cancel
 * for a delegation that already finished.
 */

export const DELEGATIONS_QUERY_KEY = ["delegations"] as const;

const fetchDelegations = () =>
  queryGoverned<DelegationListResult>(
    { verb: "delegation_list" },
    validateDelegationListResult,
    "delegation.list",
  );

export interface CancelOutcome {
  runId: string;
  /** True only when the server's response carried no error. */
  accepted: boolean;
  detail: string;
}

export function useDelegations() {
  const queryClient = useQueryClient();
  const [cancelling, setCancelling] = useState<string | undefined>();
  const [lastOutcome, setLastOutcome] = useState<CancelOutcome | undefined>();

  const query = useQuery<DelegationListResult, Error>({
    queryKey: DELEGATIONS_QUERY_KEY,
    queryFn: fetchDelegations,
    retry: false,
  });

  useGovernedEventInvalidation(affectsDelegations, DELEGATIONS_QUERY_KEY);

  const refresh = useCallback(() => {
    void queryClient.invalidateQueries({ queryKey: DELEGATIONS_QUERY_KEY });
  }, [queryClient]);

  /**
   * Request cancellation of exactly one delegation.
   *
   * The row is never updated optimistically, and the wording of a success is
   * deliberately "requested", not "cancelled". The kernel records a control
   * request; the episode then terminates on its own terms and its *settlement*
   * says what actually happened. A UI that flipped the row to "cancelled" would
   * be reporting an outcome in place of a request — the same conflation of
   * execution with settlement the epic forbids everywhere else.
   */
  const cancel = useCallback(
    async (runId: string) => {
      setCancelling(runId);
      try {
        const raw = await invoke<unknown>("sfwp_command", {
          command: { verb: "cancel_delegation", run_id: runId },
          ...(selectedActorId() ? { actAs: selectedActorId() } : {}),
        });
        const body = raw as { error?: unknown; state?: unknown } | undefined;
        const outcome: CancelOutcome =
          typeof body?.error === "string"
            ? { runId, accepted: false, detail: body.error }
            : {
                runId,
                accepted: true,
                detail:
                  body?.state === "cancellation_already_requested"
                    ? "Cancellation was already requested for this delegation."
                    : "Cancellation requested. The episode ends on its own terms; its settlement records what happened.",
              };
        setLastOutcome(outcome);
        return outcome;
      } catch (reason) {
        const outcome: CancelOutcome = {
          runId,
          accepted: false,
          detail: toError(reason).message,
        };
        setLastOutcome(outcome);
        return outcome;
      } finally {
        setCancelling(undefined);
        // Re-read either way: the roster is the authority on what happened,
        // including when the command was refused.
        void queryClient.invalidateQueries({ queryKey: DELEGATIONS_QUERY_KEY });
      }
    },
    [queryClient],
  );

  return {
    delegations: query.data?.delegations ?? [],
    /** Delegation runs that exist but could not be read — an integrity signal. */
    unreadable: query.data?.unreadable ?? [],
    isLoading: query.isLoading,
    isFetching: query.isFetching,
    error: query.error ? toError(query.error) : null,
    cancelling,
    lastOutcome,
    cancel,
    refresh,
  };
}
