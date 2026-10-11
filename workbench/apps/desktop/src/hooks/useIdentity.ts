import { useCallback, useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { validateIdentityView, type IdentityView } from "@sea-forge/contracts";
import { describeAjv } from "./governedQuery";
import { toError } from "./bridgeError";

/**
 * Who this cell believes is acting, and which cell that is.
 *
 * Both answers used to be constants. `router.tsx` declared an actor
 * (`actor_op_01`), a sponsor, a policy digest, and a verified integrity state,
 * none of which named anything real; the cell defaulted to the string
 * `"cell_local_01"`. Every governance surface downstream inherited those, so
 * the app displayed an attribution it could not support — the exact failure the
 * product exists to prevent.
 *
 * Now the actor comes from `identity.get`, which the kernel answers from the
 * connection's own uid via `SO_PEERCRED`, and the cell comes from the socket
 * path the host actually dialed. Neither is assertable by this renderer.
 */

export const IDENTITY_QUERY_KEY = ["sfwp", "identity"] as const;
export const CELL_QUERY_KEY = ["sfwp", "cell"] as const;
const ACTING_IDENTITY_STORAGE_KEY = "sea-forge.acting-identity";
const ACTING_IDENTITY_EVENT = "sea-forge:acting-identity-changed";

/** The actor this session acts as, once one has been resolved. */
export interface ResolvedActor {
  actorId: string;
  /** Wire spelling, e.g. `operator` or `R-SO`. */
  role: string;
}

export interface IdentityState {
  /** Every actor this connection may claim, in the cell's own order. */
  available: IdentityView["available"];
  /**
   * The actor this session acts as: the sole available one, or none when the
   * cell binds several and the operator has not chosen. Ambiguity resolves to
   * *no* actor rather than to the first, which would be an identity picked by
   * array order.
   */
  actor?: ResolvedActor;
  /** Whether this cell configures any identity bindings at all. */
  configured: boolean;
  /** The cell's own reason nothing is claimable, when nothing is. */
  refusal?: IdentityView["refusal"];
  /** The kernel's `Option<u32>` crosses the wire as absent *or* null. */
  uid?: number | null;
}

/**
 * The renderer may remember only which server-advertised actor the operator
 * selected for this window. It never stores a role or a fabricated identity;
 * every render derives those again from the validated `identity.get` answer,
 * and the host repeats the same check immediately before each protected call.
 */
export function selectedActorId(): string | undefined {
  try {
    return window.sessionStorage.getItem(ACTING_IDENTITY_STORAGE_KEY) ?? undefined;
  } catch {
    return undefined;
  }
}

function persistSelectedActor(actorId: string | undefined): void {
  try {
    if (actorId) {
      window.sessionStorage.setItem(ACTING_IDENTITY_STORAGE_KEY, actorId);
    } else {
      window.sessionStorage.removeItem(ACTING_IDENTITY_STORAGE_KEY);
    }
    window.dispatchEvent(new Event(ACTING_IDENTITY_EVENT));
  } catch {
    // Session presentation may be unavailable (privacy mode/test host). The
    // local state still works in this component; a new window honestly starts
    // unresolved rather than inheriting an invented actor.
  }
}

async function fetchIdentity(): Promise<IdentityView> {
  const raw = await invoke<unknown>("sfwp_identity");
  if (!validateIdentityView(raw)) {
    throw new Error(
      `identity.get response failed contract validation: ${describeAjv(validateIdentityView.errors)}`,
    );
  }
  return raw as IdentityView;
}

/**
 * How this window came to have a kernel to talk to (decision U-06).
 *
 * The distinction is load-bearing for the operator, not decoration. "This
 * window started the cell" and "this window attached to a cell that was already
 * running" have different consequences on quit — closing the first stops the
 * kernel, closing the second leaves it alone — and "there is no cell" is the
 * only one of the three that is the operator's problem to fix. A failed call
 * looks identical in all three cases, so the host reports which it is.
 */
export type SupervisionState =
  | { state: "adopted" }
  | { state: "supervised"; pid: number }
  | { state: "initialization_required" }
  | { state: "unavailable"; error_class: string; message: string };

export interface CellInfo {
  socketPath: string;
  /** The cell root: where the records are, which need not hold the socket. */
  root: string;
  supervision?: SupervisionState;
}

async function fetchCell(): Promise<CellInfo> {
  const raw = await invoke<{
    socket_path?: string;
    root?: string;
    supervision?: SupervisionState;
  }>("sfwp_cell");
  return {
    socketPath: raw?.socket_path ?? "",
    root: raw?.root ?? "",
    supervision: raw?.supervision,
  };
}

/**
 * The cell's display name: the last segment of the cell root.
 *
 * Derived from the root rather than the socket because the two need not be in
 * the same place — `SEA_FORGE_SOCKET` moves the socket while deliberately
 * leaving the records where they are, which is the server's own printed remedy
 * for a cell root too deep for a Unix socket path. Naming the cell after the
 * socket's directory would then name `/tmp`.
 */
export function cellNameFromRoot(root: string): string {
  const parts = root.split("/").filter(Boolean);
  return parts.length > 0 ? parts[parts.length - 1] : root;
}

export function useIdentity() {
  const identity = useQuery<IdentityView, Error>({
    queryKey: IDENTITY_QUERY_KEY,
    queryFn: fetchIdentity,
    // Bindings change only when `server.yaml` is reloaded. Refetching on focus
    // would make the header's actor flicker for no gain.
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    retry: false,
  });

  const cell = useQuery<CellInfo, Error>({
    queryKey: CELL_QUERY_KEY,
    queryFn: fetchCell,
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    retry: false,
  });

  const view = identity.data;
  const available = view?.available ?? [];
  const [selectedActorIdState, setSelectedActorIdState] = useState(selectedActorId);
  const [initializationError, setInitializationError] = useState<Error>();
  const [isInitializing, setIsInitializing] = useState(false);

  useEffect(() => {
    const synchronize = () => setSelectedActorIdState(selectedActorId());
    window.addEventListener(ACTING_IDENTITY_EVENT, synchronize);
    return () => window.removeEventListener(ACTING_IDENTITY_EVENT, synchronize);
  }, []);

  const selectActor = useCallback(
    (actorId: string | undefined) => {
      const valid = actorId
        ? available.some((actor) => actor.actor_id === actorId && actor.roles.length > 0)
        : true;
      if (!valid) return;
      setSelectedActorIdState(actorId);
      persistSelectedActor(actorId);
    },
    [available],
  );

  const sole = available.length === 1 ? available[0] : undefined;
  const selected = selectedActorIdState
    ? available.find((actor) => actor.actor_id === selectedActorIdState)
    : undefined;
  const acting = selected ?? sole;

  const state: IdentityState | undefined = view && {
    available,
    // A bound actor holding no role can claim nothing, so it yields no actor
    // here either — the cell records who they are while authorizing nothing.
    actor:
      acting && acting.roles.length > 0
        ? { actorId: acting.actor_id, role: acting.roles[0] }
        : undefined,
    configured: view.configured,
    refusal: view.refusal,
    uid: view.uid,
  };

  return {
    identity: state,
    selectedActorId: state?.actor?.actorId,
    selectActor,
    socketPath: cell.data?.socketPath,
    cellRoot: cell.data?.root,
    cellId: cell.data?.root ? cellNameFromRoot(cell.data.root) : undefined,
    supervision: cell.data?.supervision,
    isLoading: identity.isLoading || cell.isLoading,
    error: identity.isError
      ? toError(identity.error)
      : cell.isError
        ? toError(cell.error)
        : undefined,
    refresh: () => {
      void identity.refetch();
      void cell.refetch();
    },
    initializationError,
    isInitializing,
    initializeCell: async () => {
      setInitializationError(undefined);
      setIsInitializing(true);
      try {
        await invoke("sfwp_initialize_cell");
        await Promise.all([identity.refetch(), cell.refetch()]);
      } catch (reason) {
        setInitializationError(toError(reason));
      } finally {
        setIsInitializing(false);
      }
    },
  };
}
