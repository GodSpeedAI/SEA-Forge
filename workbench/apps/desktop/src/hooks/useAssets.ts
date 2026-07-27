import { useQuery } from "@tanstack/react-query";
import { validateAssetListResult, type AssetListResult } from "@sea-forge/contracts";
import { toError } from "./bridgeError";
import { queryGoverned } from "./governedQuery";

/**
 * The governed asset catalog over `asset.list` (epic journey 4).
 *
 * Deliberately *not* event-invalidated. The three sources behind this view —
 * materialized templates, `server.yaml`'s endpoint configuration, and the
 * extension registry — change through operator action outside any case, so no
 * kind in `KNOWN_EVENT_KINDS` announces them. Subscribing to the case/run
 * taxonomy would invalidate on frames that cannot affect the catalog while
 * still missing the ones that can, which is worse than an explicit re-read: it
 * would look like liveness without being it.
 *
 * The exception is endpoint standing, which *does* move when a probe settles.
 * That is a run event, so `refresh()` exists and the page offers it; a probe an
 * operator just ran is one click from being visible, and the page never claims
 * the catalog is live.
 */

export const ASSETS_QUERY_KEY = ["assets"] as const;

const fetchAssets = () =>
  queryGoverned<AssetListResult>(
    { verb: "asset_list" },
    validateAssetListResult,
    "asset.list",
  );

export function useAssets() {
  const query = useQuery<AssetListResult, Error>({
    queryKey: ASSETS_QUERY_KEY,
    queryFn: fetchAssets,
    retry: false,
  });

  return {
    assets: query.data?.assets ?? [],
    /** Asset sources that exist but could not be read — an integrity signal. */
    unreadable: query.data?.unreadable ?? [],
    isLoading: query.isLoading,
    isFetching: query.isFetching,
    error: query.error ? toError(query.error) : null,
    refresh: () => void query.refetch(),
  };
}
