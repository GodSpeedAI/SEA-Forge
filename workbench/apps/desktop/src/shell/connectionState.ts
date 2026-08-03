import type { useIdentity } from "../hooks/useIdentity";
import type { useServerContract } from "../hooks/useServerContract";

/**
 * What the status bar may claim about the connection.
 *
 * This used to be derived from the route name: the readiness page reported
 * "Source projection current" and every other page reported "Specification view
 * · not live", regardless of whether a kernel was reachable at all. Navigating
 * changed the reported connection state without anything about the connection
 * changing, and an unreachable cell still read as current on the readiness
 * page. It now reads the two facts that actually determine it — how the cell
 * was obtained, and whether its method catalog was negotiated.
 */
export function describeConnection(
  supervision: ReturnType<typeof useIdentity>["supervision"],
  contract: Pick<ReturnType<typeof useServerContract>, "contract" | "isLoading" | "error">,
): { label: string; ready: boolean } {
  if (supervision?.state === "initialization_required") {
    return { label: "Cell initialization required", ready: false };
  }
  if (supervision?.state === "unavailable") {
    return { label: `No cell — ${supervision.message}`, ready: false };
  }
  if (contract.isLoading) return { label: "Connecting to the cell…", ready: false };
  if (contract.error) {
    return { label: `Cell unreachable — ${contract.error.message}`, ready: false };
  }
  if (contract.contract?.incompatible) {
    return {
      label: `Protocol mismatch — this cell speaks ${contract.contract.serverProtocolVersion}`,
      ready: false,
    };
  }
  if (!contract.contract) return { label: "Cell state unknown", ready: false };
  // Which of the two the operator is in decides what closing the window does:
  // a window that started the kernel stops it on quit, one that adopted a
  // running kernel leaves it alone.
  const origin =
    supervision?.state === "supervised"
      ? "started by this window"
      : supervision?.state === "adopted"
        ? "already running"
        : "connected";
  return {
    label: `Cell ${origin} · protocol ${contract.contract.protocolVersion}`,
    ready: true,
  };
}
