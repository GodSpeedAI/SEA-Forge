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

async function fetchIdentity(): Promise<IdentityView> {
  const raw = await invoke<unknown>("sfwp_identity");
  if (!validateIdentityView(raw)) {
    throw new Error(
      `identity.get response failed contract validation: ${describeAjv(validateIdentityView.errors)}`,
    );
  }
  return raw as IdentityView;
}

async function fetchCell(): Promise<string> {
  const raw = await invoke<{ socket_path?: string }>("sfwp_cell");
  return raw?.socket_path ?? "";
}

/**
 * The cell's display name: the directory holding the socket, which is the cell
 * root by the cell contract (`<root>/server.sock`). The full path stays
 * available for the machine-readable line — an operator with two cells open
 * needs to tell them apart, and `.sea-forge` alone would not.
 */
export function cellNameFromSocket(socketPath: string): string {
  const parts = socketPath.split("/").filter(Boolean);
  return parts.length >= 2 ? parts[parts.length - 2] : socketPath;
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

  const cell = useQuery<string, Error>({
    queryKey: CELL_QUERY_KEY,
    queryFn: fetchCell,
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    retry: false,
  });

  const view = identity.data;
  const available = view?.available ?? [];
  const sole = available.length === 1 ? available[0] : undefined;

  const state: IdentityState | undefined = view && {
    available,
    // A bound actor holding no role can claim nothing, so it yields no actor
    // here either — the cell records who they are while authorizing nothing.
    actor:
      sole && sole.roles.length > 0
        ? { actorId: sole.actor_id, role: sole.roles[0] }
        : undefined,
    configured: view.configured,
    refusal: view.refusal,
    uid: view.uid,
  };

  return {
    identity: state,
    socketPath: cell.data,
    cellId: cell.data ? cellNameFromSocket(cell.data) : undefined,
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
  };
}
