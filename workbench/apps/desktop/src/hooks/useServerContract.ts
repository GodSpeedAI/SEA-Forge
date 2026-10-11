import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toError } from "./bridgeError";
import {
  validateDescribeResult,
  validateHelloResult,
  type DescribeResult,
  type HelloResult,
} from "@sea-forge/contracts";

/**
 * Protocol negotiation and method-catalog discovery (API spec §5, §17.4).
 *
 * The renderer must never decide on its own authority which methods this cell
 * supports. A hardcoded list drifts the moment the kernel grows a verb, and the
 * drift is invisible: the surface keeps saying "not available" for something
 * that now works, or worse, calls something that no longer exists. So the
 * kernel is asked, and its answer is the source of truth for every surface's
 * availability. When `case.get_horizon` lands server-side, the case surface
 * changes standing without a frontend edit.
 */

/** The protocol major this client is written against. */
export const CLIENT_PROTOCOL_VERSION = "1";

export const CONTRACT_QUERY_KEY = ["sfwp", "contract"] as const;

/** How a method stands relative to *this* build of the workbench. */
export type MethodStanding =
  /** The kernel implements it and a surface here reads it. */
  | "live"
  /** The kernel implements it; no surface here reads it yet. */
  | "unsurfaced"
  /** The kernel does not implement it in this cell. */
  | "unimplemented"
  /** Negotiation failed, so nothing can be claimed either way. */
  | "unknown";

/**
 * Methods this workbench actually reads today. Kept explicit rather than
 * inferred: "the kernel can do it" and "the operator can reach it from here"
 * are different claims, and collapsing them would let a surface imply reach it
 * does not have (epic invariant 6).
 */
export const SURFACED_METHODS: ReadonlySet<string> = new Set([
  "system.hello",
  "system.describe",
  "readiness.get",
  "identity.get",
  "thoth.ask",
  "case.entry_options",
  "case.preflight",
  "case.commit",
  "case.list",
  "case.get_overview",
  "case.get_horizon",
  "approval.list",
  "approval.decide",
  "asset.list",
  "delegation.preview",
  "delegation.list",
  "run.list",
  "run.get",
  "events.get_range",
  "request.get_status",
]);

export interface ServerContract {
  /** Protocol major the server accepted. */
  protocolVersion: string;
  /** The server's own supported major, which may differ from what it accepted. */
  serverProtocolVersion: string;
  /** Exactly what `system.hello` reported — never a client-side guess. */
  implemented: ReadonlySet<string>;
  /** Interaction class per method, from `system.describe`. */
  classes: ReadonlyMap<string, string>;
  /**
   * True when the server speaks a different major than this client. The app
   * must not present governed state across a major mismatch: field meanings
   * are not guaranteed to survive it, and a plausible-looking render of
   * misread fields is worse than no render.
   */
  incompatible: boolean;
}

function describeAjv(errors: unknown): string {
  const list = errors as { instancePath?: string; message?: string }[] | null | undefined;
  if (!list?.length) return "no detail";
  return list.map((e) => `${e.instancePath || "/"} ${e.message ?? ""}`.trim()).join("; ");
}

/**
 * Negotiation never throws. A kernel that is not listening, or one that answers
 * off-contract, is an expected operating condition for this app — not an
 * exception. Returning it as a typed result keeps "we could not ask" a
 * first-class displayable state rather than something the UI has to infer from
 * a caught error (API spec §2.6).
 */
export type NegotiationResult =
  | { ok: true; contract: ServerContract }
  | { ok: false; reason: string };

async function negotiate(): Promise<NegotiationResult> {
  let helloRaw: unknown;
  try {
    helloRaw = await invoke<unknown>("sfwp_query", {
      query: {
        verb: "system_hello",
        protocol_version: CLIENT_PROTOCOL_VERSION,
        client: "workbench",
      },
    });
  } catch (reason) {
    return { ok: false, reason: toError(reason).message };
  }

  // An `unsupported_version` body is a governed outcome, not a transport error:
  // the server answered correctly, and the answer is "we cannot talk" (§12.4).
  const maybeRejection = helloRaw as { error_class?: string; supported?: string; error?: string };
  if (maybeRejection?.error_class === "unsupported_version") {
    return {
      ok: true,
      contract: {
        protocolVersion: CLIENT_PROTOCOL_VERSION,
        serverProtocolVersion: maybeRejection.supported ?? "unknown",
        implemented: new Set(),
        classes: new Map(),
        incompatible: true,
      },
    };
  }

  if (!validateHelloResult(helloRaw)) {
    return {
      ok: false,
      reason: `system.hello response failed contract validation: ${describeAjv(validateHelloResult.errors)}`,
    };
  }
  const hello = helloRaw as HelloResult;

  // `describe` enriches the catalog with interaction classes. It is not
  // required for the app to function honestly, so a failure here degrades the
  // detail shown rather than blocking negotiation.
  let classes = new Map<string, string>();
  try {
    const describeRaw = await invoke<unknown>("sfwp_query", { query: { verb: "system_describe" } });
    if (validateDescribeResult(describeRaw)) {
      const described = describeRaw as DescribeResult;
      classes = new Map(described.methods.map((m) => [m.method, m.class]));
    }
  } catch {
    // ponytail: classes are decoration on the catalog; the catalog itself came
    // from `hello` and is already authoritative. Surface the gap if operators
    // start needing class-level detail during an outage.
  }

  return {
    ok: true,
    contract: {
      protocolVersion: hello.protocol_version,
      serverProtocolVersion: hello.server_protocol_version,
      implemented: new Set(hello.implemented_methods),
      classes,
      incompatible:
        hello.server_protocol_version.split(".")[0] !== CLIENT_PROTOCOL_VERSION.split(".")[0],
    },
  };
}

/**
 * Negotiate once per session. The method catalog changes only when the kernel
 * restarts, so this deliberately does not refetch on focus or interval — a
 * catalog that flickers would make surfaces flicker with it.
 */
export function useServerContract() {
  const query = useQuery<NegotiationResult, Error>({
    queryKey: CONTRACT_QUERY_KEY,
    queryFn: negotiate,
    // The catalog changes only when the kernel restarts, so a flickering
    // refetch would make every surface's standing flicker with it.
    staleTime: Infinity,
    refetchOnWindowFocus: false,
    retry: false,
  });

  const result = query.data;
  const failure = result && !result.ok ? new Error(result.reason) : undefined;

  return {
    contract: result?.ok ? result.contract : undefined,
    isLoading: query.isLoading,
    error: failure ?? (query.isError ? toError(query.error) : undefined),
    refresh: () => void query.refetch(),
  };
}

/**
 * How `method` stands, given a negotiated contract. Separate from the hook so
 * it is directly testable and so a surface can resolve several methods without
 * re-deriving the rules.
 */
export function standingOf(
  method: string,
  contract: ServerContract | undefined,
): MethodStanding {
  if (!contract || contract.incompatible) return "unknown";
  if (!contract.implemented.has(method)) return "unimplemented";
  return SURFACED_METHODS.has(method) ? "live" : "unsurfaced";
}

/** Governed presentation for each standing. `live` is the only success. */
export const STANDING_PRESENTATION: Record<
  MethodStanding,
  { variant: string; label: string; explanation: string }
> = {
  live: {
    variant: "ready",
    label: "Live",
    explanation: "This cell implements the method and this surface reads it.",
  },
  unsurfaced: {
    variant: "degraded",
    label: "Implemented, not surfaced",
    explanation:
      "This cell implements the method, but no view here reads it yet. The capability exists; the path to it does not.",
  },
  unimplemented: {
    // `unsupported`, not `blocked`: nothing is preventing lawful work here, and
    // nothing is wrong. The installation simply does not provide the method
    // (epic invariant 6 keeps these distinct).
    variant: "unsupported",
    label: "Not implemented in this cell",
    explanation:
      "The kernel's own method catalog does not list this method, so there is nothing to read. This is a limitation of the installation, not a failure.",
  },
  unknown: {
    variant: "unknown",
    label: "Unknown",
    explanation:
      "The method catalog could not be negotiated, so nothing can be claimed about this method either way.",
  },
};
