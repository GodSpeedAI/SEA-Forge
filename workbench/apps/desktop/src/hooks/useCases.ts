import { useCallback } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  validateCaseHorizon,
  validateCaseListResult,
  validateCaseOverview,
  type CaseHorizon,
  type CaseListResult,
  type CaseOverview,
} from "@sea-forge/contracts";
import { toError } from "./bridgeError";
import { queryGoverned } from "./governedQuery";
import { affectsCases } from "./eventKinds";
import { useGovernedEventInvalidation } from "./useGovernedEventInvalidation";

export { GovernedViewError as CaseViewError } from "./governedQuery";

/**
 * Case navigation over `case.list`, `case.get_overview`, and `case.get_horizon`.
 *
 * All three are read-only projections the kernel rebuilds from committed
 * records, so a live event is treated purely as a signal to re-read — never as
 * data to patch into a held view. That keeps recovery after a missed frame
 * identical to the steady state, and leaves no client-side reduction that could
 * drift from the kernel's own answer (epic invariant 14).
 */

export const CASES_QUERY_KEY = ["cases"] as const;

const fetchCaseList = () =>
  queryGoverned<CaseListResult>({ verb: "case_list" }, validateCaseListResult, "case.list");

const fetchCaseOverview = (caseId: string) =>
  queryGoverned<CaseOverview>(
    { verb: "case_get_overview", case_id: caseId },
    validateCaseOverview,
    "case.get_overview",
  );

const fetchCaseHorizon = (caseId: string) =>
  queryGoverned<CaseHorizon>(
    { verb: "case_get_horizon", case_id: caseId },
    validateCaseHorizon,
    "case.get_horizon",
  );

/**
 * Re-read case views when a case-relevant event lands. Unrecognized kinds also
 * trigger a read — see `eventKinds` for why that asymmetry is deliberate.
 */
const useCaseEventInvalidation = () =>
  useGovernedEventInvalidation(affectsCases, CASES_QUERY_KEY);

export function useCaseList() {
  const queryClient = useQueryClient();
  useCaseEventInvalidation();

  const query = useQuery<CaseListResult, Error>({
    queryKey: [...CASES_QUERY_KEY, "list"],
    queryFn: fetchCaseList,
    retry: false,
  });

  const refresh = useCallback(() => {
    void queryClient.invalidateQueries({ queryKey: CASES_QUERY_KEY });
  }, [queryClient]);

  return {
    cases: query.data?.cases ?? [],
    /** Case directories whose record could not be parsed — an integrity signal. */
    unreadable: query.data?.unreadable ?? [],
    isLoading: query.isLoading,
    error: query.error ? toError(query.error) : null,
    refresh,
  };
}

export function useCaseOverview(caseId: string | undefined) {
  useCaseEventInvalidation();

  const query = useQuery<CaseOverview, Error>({
    queryKey: [...CASES_QUERY_KEY, "overview", caseId ?? null],
    queryFn: () => fetchCaseOverview(caseId as string),
    enabled: Boolean(caseId),
    retry: false,
  });

  return {
    overview: query.data,
    isLoading: query.isLoading,
    error: query.error ? toError(query.error) : null,
  };
}

export function useCaseHorizon(caseId: string | undefined) {
  useCaseEventInvalidation();

  const query = useQuery<CaseHorizon, Error>({
    queryKey: [...CASES_QUERY_KEY, "horizon", caseId ?? null],
    queryFn: () => fetchCaseHorizon(caseId as string),
    enabled: Boolean(caseId),
    retry: false,
  });

  return {
    horizon: query.data,
    isLoading: query.isLoading,
    error: query.error ? toError(query.error) : null,
  };
}
