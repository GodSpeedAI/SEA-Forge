import { describe, expect, it } from "vitest";
import { describeConnection } from "./connectionState";
import type { ServerContract } from "../hooks/useServerContract";

/**
 * The status bar's claim about the connection used to be derived from the route
 * name: `/readiness` reported "Source projection current" and every other route
 * reported "Specification view · not live". Navigating changed the reported
 * connection state while nothing about the connection changed, and an
 * unreachable kernel still read as "current" on the readiness page.
 *
 * These pin the two facts it may actually depend on: how the cell was obtained,
 * and whether its catalog was negotiated.
 */

const negotiated: ServerContract = {
  protocolVersion: "1",
  serverProtocolVersion: "1",
  implemented: new Set(["system.hello"]),
  classes: new Map(),
  incompatible: false,
};

const settled = { contract: negotiated, isLoading: false, error: undefined };

describe("connection state", () => {
  it("reports a fresh root as awaiting explicit initialization, never as connected", () => {
    const state = describeConnection({ state: "initialization_required" }, settled);
    expect(state.ready).toBe(false);
    expect(state.label).toContain("initialization required");
  });

  it("reports no cell, and why, when supervision could not produce one", () => {
    const state = describeConnection(
      {
        state: "unavailable",
        error_class: "server_binary_not_found",
        message: "no `sea-forge-server` binary found beside this application or on PATH.",
      },
      settled,
    );
    expect(state.ready).toBe(false);
    expect(state.label).toContain("No cell");
    // The operator cannot act on "something went wrong"; they can act on this.
    expect(state.label).toContain("sea-forge-server");
  });

  /**
   * The distinction decides what closing the window does: a window that started
   * the kernel stops it on quit, a window that adopted one leaves it running.
   */
  it("distinguishes a cell it started from one it adopted", () => {
    expect(describeConnection({ state: "supervised", pid: 42 }, settled).label).toContain(
      "started by this window",
    );
    expect(describeConnection({ state: "adopted" }, settled).label).toContain(
      "already running",
    );
  });

  it("refuses to claim readiness while the catalog is still being negotiated", () => {
    const state = describeConnection(
      { state: "supervised", pid: 1 },
      { contract: undefined, isLoading: true, error: undefined },
    );
    expect(state.ready).toBe(false);
  });

  /**
   * A supervised cell that cannot be spoken to is the most misleading case: the
   * process exists, so supervision succeeded, but nothing it reports can be
   * trusted. Reporting "ready" here would be the exact failure this bar exists
   * to prevent.
   */
  it("does not read a live process as a working connection", () => {
    const state = describeConnection(
      { state: "supervised", pid: 1 },
      { contract: undefined, isLoading: false, error: new Error("connection refused") },
    );
    expect(state.ready).toBe(false);
    expect(state.label).toContain("unreachable");
  });

  it("refuses across a protocol major mismatch", () => {
    const state = describeConnection(
      { state: "adopted" },
      {
        contract: { ...negotiated, serverProtocolVersion: "2", incompatible: true },
        isLoading: false,
        error: undefined,
      },
    );
    expect(state.ready).toBe(false);
    expect(state.label).toContain("2");
  });

  it("reports a negotiated cell as ready", () => {
    const state = describeConnection({ state: "adopted" }, settled);
    expect(state.ready).toBe(true);
  });
});
