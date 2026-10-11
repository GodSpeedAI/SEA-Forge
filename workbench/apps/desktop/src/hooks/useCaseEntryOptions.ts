import { invoke } from "@tauri-apps/api/core";
import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import { validateEntryOptionsResult, type EntryOptionsResult } from "@sea-forge/contracts";

export const CASE_ENTRY_OPTIONS_QUERY_KEY = ["case", "entry_options"] as const;

/**
 * Fetch `case.entry_options` through the closed Tauri bridge, validated
 * against the generated AJV validator — mirrors `useReadiness.ts::fetchReadiness`.
 */
async function fetchEntryOptions(): Promise<EntryOptionsResult> {
  const raw = await invoke<unknown>("sfwp_query", {
    query: { verb: "case_entry_options" },
  });
  if (!validateEntryOptionsResult(raw)) {
    const detail = validateEntryOptionsResult.errors
      ?.map((e) => `${e.instancePath || "(root)"} ${e.message ?? ""}`.trim())
      .join("; ");
    throw new Error(
      `case.entry_options response failed contract validation: ${detail ?? "unknown schema error"}`,
    );
  }
  return raw;
}

/**
 * A plain read (no lifecycle) — TanStack Query owns fetch/refetch, no
 * machine warranted (implementation-workflow.md step 10).
 */
export function useCaseEntryOptions(): { query: UseQueryResult<EntryOptionsResult, Error> } {
  const query = useQuery<EntryOptionsResult, Error>({
    queryKey: CASE_ENTRY_OPTIONS_QUERY_KEY,
    queryFn: fetchEntryOptions,
    retry: false,
  });
  return { query };
}
