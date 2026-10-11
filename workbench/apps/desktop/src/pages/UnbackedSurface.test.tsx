import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import {
  ArtifactsPage,
  CapabilitiesPage,
  FederationPage,
  MemoryPage,
} from "./SurfacesPages";
import { standingOf, type ServerContract } from "../hooks/useServerContract";

/** The kernel's real catalog as of this build: none of the surfaces below are in it. */
const IMPLEMENTED = [
  "system.hello",
  "system.describe",
  "system.get_schema",
  "request.get_status",
  "events.subscribe",
  "events.unsubscribe",
  "events.get_range",
  "readiness.get",
  "case.entry_options",
  "case.preflight",
  "case.commit",
];

function hello(methods: string[] = IMPLEMENTED) {
  return {
    protocol_version: "1",
    server_protocol_version: "1",
    implemented_methods: methods,
  };
}

function renderPage(Page: () => React.JSX.Element) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={client}>
      <Page />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockImplementation((_cmd: string, args: { query: { verb: string } }) =>
    args.query.verb === "system_hello"
      ? Promise.resolve(hello())
      : Promise.resolve({ protocol_version: "1", methods: [] }),
  );
});

afterEach(cleanup);

// Evidence is no longer here: it is backed by `run.list`/`run.get` and lives in
// `EvidencePage.tsx`. A surface leaving this list is the intended direction of
// travel — the list should shrink as the kernel grows methods.
const UNBACKED = [
  { Page: MemoryPage, heading: "Memory recall", method: "memory.recall" },
  { Page: CapabilitiesPage, heading: "Capability matrix", method: "capability.list" },
  { Page: ArtifactsPage, heading: "Artifact registry", method: "artifact.list" },
  { Page: FederationPage, heading: "Federation gateway", method: "federation.preview_export" },
];

describe.each(UNBACKED)("$heading (no kernel projection)", ({ Page, heading, method }) => {
  it("never asserts a success state it cannot evidence", async () => {
    const { container } = renderPage(Page);

    expect(screen.getByRole("heading", { level: 1, name: heading })).toBeVisible();
    await screen.findByText(`requires ${method}`);

    // The defect this replaces: these surfaces rendered a `ready` pill labelled
    // "<kind> Active" over no data at all. Whatever the negotiation returns, a
    // success variant must never appear here.
    const variants = [...container.querySelectorAll("[data-variant]")].map((el) =>
      el.getAttribute("data-variant"),
    );
    expect(variants).not.toHaveLength(0);
    expect(variants).not.toContain("ready");
  });

  it("names the method it requires and reports the kernel's own answer", async () => {
    renderPage(Page);
    expect(await screen.findByText(`requires ${method}`)).toBeVisible();
    expect(await screen.findByText("Not implemented in this cell")).toBeVisible();
  });
});

describe("contract-derived standing", () => {
  function contract(over: Partial<ServerContract> = {}): ServerContract {
    return {
      protocolVersion: "1",
      serverProtocolVersion: "1",
      implemented: new Set(IMPLEMENTED),
      classes: new Map(),
      incompatible: false,
      ...over,
    };
  }

  it("stops reporting a method as missing once the kernel implements it", () => {
    // The property that makes these surfaces self-updating: nothing in the
    // renderer decides availability, so a kernel that grows the verb flips the
    // standing with no frontend edit.
    expect(standingOf("capability.list", contract())).toBe("unimplemented");
    expect(
      standingOf("capability.list", contract({ implemented: new Set([...IMPLEMENTED, "capability.list"]) })),
    ).toBe("unsurfaced");
  });

  it("separates what the kernel implements from what this build can reach", () => {
    // `readiness.get` is both implemented and surfaced; `events.subscribe` is
    // implemented but no view here reads it. Collapsing the two would let a
    // surface imply reach it does not have.
    expect(standingOf("readiness.get", contract())).toBe("live");
    expect(standingOf("events.subscribe", contract())).toBe("unsurfaced");
  });

  it("claims nothing when negotiation failed or the protocol majors differ", () => {
    expect(standingOf("readiness.get", undefined)).toBe("unknown");
    expect(standingOf("readiness.get", contract({ incompatible: true }))).toBe("unknown");
  });
});
