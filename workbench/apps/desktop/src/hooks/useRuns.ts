import { useQuery } from "@tanstack/react-query";
import {
  validateRunListResult,
  validateRunRecord,
  type RunListResult,
  type RunRecord,
} from "@sea-forge/contracts";
import { toError } from "./bridgeError";
import { queryGoverned } from "./governedQuery";
import { affectsCases } from "./eventKinds";
import { useGovernedEventInvalidation } from "./useGovernedEventInvalidation";

/**
 * Run records over `run.list` and `run.get`.
 *
 * Both are read-only projections the kernel rebuilds from committed per-run
 * files, so a live event is a signal to re-read — never data to patch into a
 * held view. That keeps recovery after a missed frame identical to the steady
 * state and leaves no client-side reduction that could drift from the kernel's
 * own answer (epic invariant 14).
 *
 * Run relevance reuses `affectsCases`: every kind that can change a case can
 * change one of its runs, and the taxonomy has no run-specific kinds to narrow
 * against yet. Sharing the predicate keeps the two from silently disagreeing
 * about the same frame.
 */

export const RUNS_QUERY_KEY = ["runs"] as const;

function fetchRunList(caseId?: string): Promise<RunListResult> {
  const query: { verb: "run_list"; case_id?: string } = { verb: "run_list" };
  if (caseId) query.case_id = caseId;
  return queryGoverned<RunListResult>(query, validateRunListResult, "run.list");
}

const fetchRunRecord = (runId: string) =>
  queryGoverned<RunRecord>({ verb: "run_get", run_id: runId }, validateRunRecord, "run.get");

const useRunEventInvalidation = () =>
  useGovernedEventInvalidation(affectsCases, RUNS_QUERY_KEY);

export function useRunList(caseId?: string) {
  useRunEventInvalidation();

  const query = useQuery<RunListResult, Error>({
    queryKey: [...RUNS_QUERY_KEY, "list", caseId ?? null],
    queryFn: () => fetchRunList(caseId),
    retry: false,
  });

  return {
    runs: query.data?.runs ?? [],
    /** Run directories with no readable trace or settlement — integrity signal. */
    unreadable: query.data?.unreadable ?? [],
    isLoading: query.isLoading,
    error: query.error ? toError(query.error) : null,
  };
}

export function useRunRecord(runId: string | undefined) {
  useRunEventInvalidation();

  const query = useQuery<RunRecord, Error>({
    queryKey: [...RUNS_QUERY_KEY, "record", runId ?? null],
    queryFn: () => fetchRunRecord(runId as string),
    enabled: Boolean(runId),
    retry: false,
  });

  return {
    record: query.data,
    isLoading: query.isLoading,
    error: query.error ? toError(query.error) : null,
  };
}
