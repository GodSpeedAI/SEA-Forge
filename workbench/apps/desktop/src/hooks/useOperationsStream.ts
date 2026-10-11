import { useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toError } from "./bridgeError";
import { listen } from "@tauri-apps/api/event";
import { useQuery, useQueryClient, type UseQueryResult } from "@tanstack/react-query";
import {
  validateEventFrame,
  validateRequestRecord,
  type EventFrame,
  type RequestRecord,
} from "@sea-forge/contracts";

export const EVENTS_QUERY_KEY = ["events"] as const;
export const REQUEST_STATUS_QUERY_KEY = ["request-status"] as const;

/**
 * Per-read frame budget. The server caps every `events.get_range` at
 * `EVENTS_REPLAY_CAP` (500) and returns the *earliest* matching frames, so a
 * single unpaged read of a long ledger yields ancient history rather than the
 * tail. We page from the last known cursor instead.
 */
const PAGE_SIZE = 500;

/**
 * Page budget for one drain. Bounds both a very long ledger and the pathological
 * case of a server that keeps returning full pages without advancing the cursor
 * — without it, that combination is an unbounded loop in the render process.
 */
const MAX_PAGES = 20;

/**
 * Retained frames. The console is a monitor, not an archive; the durable ledger
 * remains the source of truth and is re-readable at any cursor.
 */
// ponytail: fixed window, no virtualization. Revisit only if operators actually
// scroll past it.
const MAX_RETAINED = 2000;

/**
 * Read one bounded page of durable event frames, exclusive of `fromCursor`.
 *
 * Every frame is validated against the generated `EventFrame` contract before
 * it is treated as one. A frame that fails validation is a contract drift
 * between server and renderer, not a display problem, so it throws rather than
 * being skipped — silently dropping frames is exactly the "no silent loss"
 * failure this stream exists to prevent.
 */
async function fetchEventPage(fromCursor?: string): Promise<EventFrame[]> {
  const query: {
    verb: "events_get_range";
    from_cursor?: string;
    limit: number;
  } = { verb: "events_get_range", limit: PAGE_SIZE };
  if (fromCursor) {
    query.from_cursor = fromCursor;
  }

  const raw = await invoke<unknown>("sfwp_query", { query });

  const events = (raw as { events?: unknown })?.events;
  if (!Array.isArray(events)) {
    throw new Error("events.get_range response has no `events` array");
  }

  return events.map((frame, index) => {
    if (!validateEventFrame(frame)) {
      const detail = validateEventFrame.errors
        ?.map((e) => `${e.instancePath || "(root)"} ${e.message ?? ""}`.trim())
        .join("; ");
      throw new Error(
        `event frame ${index} failed contract validation: ${detail ?? "unknown schema error"}`,
      );
    }
    return frame as EventFrame;
  });
}

/**
 * Drain every durable frame after `fromCursor`, following pages to the tail.
 *
 * A short page means the ledger is exhausted. A full page that does not advance
 * the cursor would otherwise re-request the same window forever, so we stop on
 * a non-advancing cursor as well.
 */
async function drainFrom(fromCursor?: string): Promise<EventFrame[]> {
  const drained: EventFrame[] = [];
  let cursor = fromCursor;

  for (let page = 0; page < MAX_PAGES; page += 1) {
    const batch = await fetchEventPage(cursor);
    drained.push(...batch);
    if (batch.length < PAGE_SIZE) {
      break;
    }
    const next = batch[batch.length - 1]?.cursor;
    if (!next || next === cursor) {
      break;
    }
    cursor = next;
  }

  return drained;
}

export interface OperationsStream {
  /** Durable frames in append order, oldest first. */
  frames: EventFrame[];
  /** The newest durable cursor held, or `undefined` before the first read. */
  cursor: string | undefined;
  /**
   * The last authoritative read failed. Frames already held stay rendered and
   * must be presented as last-known rather than current.
   */
  stale: boolean;
  error: Error | null;
  isLoading: boolean;
  /** Force an authoritative re-read from the last held cursor. */
  refresh: () => void;
}

/**
 * The governed event stream for the operations console.
 *
 * The server documents its live subscription as "a convenience for connected
 * subscribers, never the source truth" (`sfwp/events.rs`), so a live frame is
 * treated purely as a signal to re-read the durable ledger — never as data.
 * Recovery after a dropped or missed frame is therefore the same code path as
 * the steady state, and no gap bookkeeping exists to drift.
 */
export function useOperationsStream(): OperationsStream {
  const queryClient = useQueryClient();
  // Accumulated across reads: each read is exclusive of the cursor it starts
  // from, so the query result is a window onto this, not a replacement for it.
  const framesRef = useRef<EventFrame[]>([]);

  const query = useQuery<EventFrame[], Error>({
    queryKey: EVENTS_QUERY_KEY,
    queryFn: async () => {
      const held = framesRef.current;
      const fresh = await drainFrom(held[held.length - 1]?.cursor);
      // A fresh array each time: React must see a new reference to re-render,
      // and the window is bounded so a long-lived session cannot grow without end.
      framesRef.current = [...held, ...fresh].slice(-MAX_RETAINED);
      return framesRef.current;
    },
    retry: false,
  });

  const refresh = useCallback(() => {
    void queryClient.invalidateQueries({ queryKey: EVENTS_QUERY_KEY });
  }, [queryClient]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    const safeUnlisten = (fn?: () => void) => {
      // StrictMode mounts and unmounts effects twice; a partially-resolved
      // unlisten can throw in a mocked runtime. The subscription is going away
      // either way.
      try {
        fn?.();
      } catch {
        /* subscription already gone */
      }
    };

    listen("sfwp://event", () => {
      void queryClient.invalidateQueries({ queryKey: EVENTS_QUERY_KEY });
      // A live frame can also settle an in-flight correlated request.
      void queryClient.invalidateQueries({ queryKey: REQUEST_STATUS_QUERY_KEY });
    })
      .then((fn) => {
        if (disposed) {
          safeUnlisten(fn);
        } else {
          unlisten = fn;
        }
      })
      .catch(() => {
        /* no host bridge — the stream still re-reads on demand */
      });

    return () => {
      disposed = true;
      safeUnlisten(unlisten);
    };
  }, [queryClient]);

  const frames = query.data ?? framesRef.current;

  return {
    frames,
    cursor: frames[frames.length - 1]?.cursor,
    // Holding frames from a read that has since failed is the stale case. With
    // no frames yet there is nothing to mis-present, so that is an error, not
    // staleness.
    stale: query.isError && frames.length > 0,
    // Tauri rejects with a bare string, so normalize before anything reads
    // `.message` off it (see `toError`).
    error: query.error ? toError(query.error) : null,
    isLoading: query.isLoading,
    refresh,
  };
}

/**
 * The correlation record for one submitted request, or `undefined` when no
 * request is being tracked.
 *
 * This is the *execution* lifecycle only. Settlement is carried in `outcome`
 * once the request is terminal — see `settlementOf`.
 */
export function useRequestStatus(
  requestId?: string,
): UseQueryResult<RequestRecord, Error> {
  return useQuery<RequestRecord, Error>({
    queryKey: [...REQUEST_STATUS_QUERY_KEY, requestId ?? null],
    enabled: Boolean(requestId),
    retry: false,
    queryFn: async () => {
      const raw = await invoke<unknown>("sfwp_request_status", { requestId });
      if (!validateRequestRecord(raw)) {
        const detail = validateRequestRecord.errors
          ?.map((e) => `${e.instancePath || "(root)"} ${e.message ?? ""}`.trim())
          .join("; ");
        throw new Error(
          `request.get_status response failed contract validation: ${detail ?? "unknown schema error"}`,
        );
      }
      return raw as RequestRecord;
    },
  });
}

export type SettlementStanding =
  | { kind: "not_projected"; reason: string }
  | { kind: "rejected"; reason: string }
  | { kind: "settled"; reason: string };

/**
 * Separate settlement standing from execution standing.
 *
 * A request that executed successfully can still carry a rejecting outcome, so
 * the two must never share a pill (plan Task 8). The kernel emits no
 * `settlement.*` event kind and no settlement contract exists, so the only
 * honest source is the terminal `outcome` payload — and when that payload
 * carries nothing we recognise, settlement is reported as *not projected*
 * rather than inferred as success.
 */
export function settlementOf(record: RequestRecord | undefined): SettlementStanding {
  if (!record || record.status === "pending") {
    return {
      kind: "not_projected",
      reason: "The request has not reached a terminal state.",
    };
  }

  const outcome = record.outcome;
  if (!outcome || Object.keys(outcome).length === 0) {
    return {
      kind: "not_projected",
      reason: "The terminal response carried no settlement payload.",
    };
  }

  // `error`/`error_class` is the server's rejection envelope (see the server's
  // response construction); anything else is an outcome we cannot classify, and
  // an unclassifiable outcome is never reported as settled.
  if (typeof outcome.error === "string") {
    return { kind: "rejected", reason: outcome.error };
  }
  if (typeof outcome.rejected_as_stale === "object" && outcome.rejected_as_stale !== null) {
    return {
      kind: "rejected",
      reason: "Preconditions changed before the command was applied.",
    };
  }

  return {
    kind: "settled",
    reason: "The terminal response carried a non-error outcome.",
  };
}
