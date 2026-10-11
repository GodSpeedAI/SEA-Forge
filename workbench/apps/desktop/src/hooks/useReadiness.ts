import { invoke } from "@tauri-apps/api/core";
import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import { validateReadinessView, type ReadinessView } from "@sea-forge/contracts";
import { affectsReadiness } from "./eventKinds";
import { useGovernedEventInvalidation } from "./useGovernedEventInvalidation";

/**
 * The intended-operation shape, derived from the generated `ReadinessView`
 * contract rather than handwritten. The contracts barrel re-exports only the
 * top-level `ReadinessView`; `IntendedOperation` is an inlined `$def`, so we
 * project it off the generated type (non-null) to keep a single source of truth.
 */
export type IntendedOperation = NonNullable<ReadinessView["intended_operation"]>;

/**
 * Query key root for every readiness view. Event-driven invalidation targets
 * this prefix so any intended-operation variant refetches on a relevant change.
 */
export const READINESS_QUERY_KEY = ["readiness"] as const;

/**
 * Fetch a `readiness.get` projection through the closed Tauri bridge
 * (`sfwp_query` command, `SfwpQuery::ReadinessGet` variant), then validate the
 * raw response against the generated AJV validator before treating it as a
 * `ReadinessView`. On validation failure we throw so TanStack Query surfaces it
 * as an error rather than trusting an unvalidated shape — the generated
 * contract is the single source of truth.
 */
async function fetchReadiness(
  intendedOperation?: IntendedOperation,
): Promise<ReadinessView> {
  const query: { verb: "readiness_get"; intended_operation?: IntendedOperation } = {
    verb: "readiness_get",
  };
  if (intendedOperation) {
    query.intended_operation = intendedOperation;
  }

  const raw = await invoke<unknown>("sfwp_query", { query });

  if (!validateReadinessView(raw)) {
    const detail = validateReadinessView.errors
      ?.map((e) => `${e.instancePath || "(root)"} ${e.message ?? ""}`.trim())
      .join("; ");
    throw new Error(
      `readiness.get response failed contract validation: ${detail ?? "unknown schema error"}`,
    );
  }

  return raw as ReadinessView;
}

export interface UseReadinessResult {
  query: UseQueryResult<ReadinessView, Error>;
}

/**
 * Readiness view query. A plain read (no lifecycle), so no XState machine is
 * warranted here — TanStack Query owns fetch/refetch/last-known-data semantics
 * (see implementation-workflow.md step 10, "plain queries need no machine").
 *
 * Live invalidation: the host emits every SFWP event frame on `sfwp://event`.
 * Frames whose kind is known not to affect readiness (case commits, approval
 * decisions — none of which touch the self-model or endpoint config) are
 * skipped; every other kind, including any the server grows later, still
 * invalidates. See `eventKinds` for why that asymmetry is the safe default.
 */
export function useReadiness(intendedOperation?: IntendedOperation): UseReadinessResult {
  const query = useQuery<ReadinessView, Error>({
    queryKey: [...READINESS_QUERY_KEY, intendedOperation ?? null],
    queryFn: () => fetchReadiness(intendedOperation),
    // Keep the last successful view rendered across a failed refetch so a
    // mid-view server disconnect marks the view stale rather than blanking it.
    retry: false,
  });

  useGovernedEventInvalidation(affectsReadiness, READINESS_QUERY_KEY);

  return { query };
}
