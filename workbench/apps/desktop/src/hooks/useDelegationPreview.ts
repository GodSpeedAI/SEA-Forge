import { useQuery } from "@tanstack/react-query";
import {
  validateDelegationPreviewResult,
  type DelegationPreviewResult,
} from "@sea-forge/contracts";
import { toError } from "./bridgeError";
import { queryGoverned } from "./governedQuery";

/**
 * The job contract a delegation would run under, over `delegation.preview`
 * (epic 9.4).
 *
 * Unlike the other inspect hooks this one is parameterised, and the parameters
 * are the whole point: a contract is only meaningful for one exact request. So
 * the request travels in the query key, and a request the caller has not
 * committed yet (`null`) issues no query at all rather than previewing a
 * half-typed instruction against the socket on every keystroke.
 *
 * Not event-invalidated, for the same reason `useAssets` is not: the inputs are
 * `server.yaml` and the operator's own form, and no kind in `KNOWN_EVENT_KINDS`
 * announces a change to either. What *can* move underneath a preview is the
 * endpoint's standing, when a probe settles — `refresh()` exists for that, and
 * `contract_digest` is what tells an operator whether what they read is still
 * what would run.
 */

export interface DelegationPreviewRequest {
  endpoint: string;
  instruction: string;
  max_turns: number;
  model?: string;
  token_budget?: number;
}

export const delegationPreviewQueryKey = (request: DelegationPreviewRequest | null) =>
  ["delegation-preview", request] as const;

/**
 * Build the wire query, omitting optionals rather than sending `null`. The
 * host bridge's `Option<String>`/`Option<u64>` fields reject an explicit null,
 * so "no model chosen" has to be an absent key.
 */
function toQuery(request: DelegationPreviewRequest): Record<string, unknown> {
  const query: Record<string, unknown> = {
    verb: "delegation_preview",
    endpoint: request.endpoint,
    instruction: request.instruction,
    max_turns: request.max_turns,
  };
  if (request.model) query.model = request.model;
  if (request.token_budget !== undefined) query.token_budget = request.token_budget;
  return query;
}

export function useDelegationPreview(request: DelegationPreviewRequest | null) {
  const query = useQuery<DelegationPreviewResult, Error>({
    queryKey: delegationPreviewQueryKey(request),
    queryFn: () =>
      queryGoverned<DelegationPreviewResult>(
        toQuery(request as DelegationPreviewRequest),
        validateDelegationPreviewResult,
        "delegation.preview",
      ),
    enabled: request !== null,
    retry: false,
  });

  return {
    preview: query.data ?? null,
    isLoading: query.isLoading && request !== null,
    isFetching: query.isFetching,
    error: query.error ? toError(query.error) : null,
    refresh: () => void query.refetch(),
  };
}
