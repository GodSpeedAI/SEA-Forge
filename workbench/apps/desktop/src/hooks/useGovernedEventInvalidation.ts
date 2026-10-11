import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useQueryClient, type QueryKey } from "@tanstack/react-query";

/**
 * Re-read a governed view when a durable event says it may have changed.
 *
 * Every inspect hook that wants liveness needs the same subscription, and by
 * the time a fifth surface wanted one there were four hand-copied versions of
 * it. They differed only in the predicate and the query key — but they each
 * carried two rules subtle enough that a sixth copy would eventually get one
 * wrong:
 *
 * 1. **`event?.payload`, not `event.payload`.** A listener that throws tears
 *    down the whole subscription, so a frame in an unexpected shape has to
 *    degrade to "invalidate anyway" rather than to a silently dead listener.
 *    The failure modes are not symmetric: an extra read costs a round trip, a
 *    dead listener costs every future update.
 * 2. **The `disposed` flag.** `listen()` returns a promise. If the component
 *    unmounts before it resolves, the returned unlisten function has nobody to
 *    call it, and the listener leaks for the life of the window. Resolving
 *    after disposal must therefore unlisten immediately.
 *
 * A missing host bridge (browser dev, tests) is not an error: the view simply
 * has no liveness and still re-reads on demand.
 *
 * `useOperationsStream` deliberately does not use this — it consumes the event
 * frames as data rather than treating them as a signal to refetch, which is a
 * different job and not a variation on this one.
 */
export function useGovernedEventInvalidation(
  affects: (frame: unknown) => boolean,
  queryKey: QueryKey,
): void {
  const queryClient = useQueryClient();
  // `queryKey` is an array literal at nearly every call site, so a new
  // identity arrives on every render; serializing it keeps the effect from
  // resubscribing on each one.
  const keyToken = JSON.stringify(queryKey);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    const safeUnlisten = (fn?: () => void) => {
      try {
        fn?.();
      } catch {
        /* subscription already gone */
      }
    };

    listen<unknown>("sfwp://event", (event) => {
      if (!affects(event?.payload)) return;
      void queryClient.invalidateQueries({ queryKey: JSON.parse(keyToken) as QueryKey });
    })
      .then((fn) => {
        if (disposed) safeUnlisten(fn);
        else unlisten = fn;
      })
      .catch(() => {
        /* no host bridge — the view still re-reads on demand */
      });

    return () => {
      disposed = true;
      safeUnlisten(unlisten);
    };
  }, [queryClient, affects, keyToken]);
}
