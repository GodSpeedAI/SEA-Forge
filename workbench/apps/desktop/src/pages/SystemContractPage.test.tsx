import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { SystemContractPage } from "./SystemContractPage";

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

function respondWith(helloBody: unknown, describeBody?: unknown) {
  invokeMock.mockImplementation((...args: unknown[]) => {
    const verb = (args[1] as { query?: { verb?: string } } | undefined)?.query?.verb;
    return verb === "system_describe"
      ? Promise.resolve(describeBody ?? { protocol_version: "1", methods: [] })
      : Promise.resolve(helloBody);
  });
}

function renderPage() {
  // `gcTime: 0` would let the query be collected while its rejection is still
  // settling, orphaning the error. The app never collects this query anyway —
  // it is negotiated once per session.
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <SystemContractPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => invokeMock.mockClear());
afterEach(cleanup);

describe("installation contract", () => {
  it("reports the catalog the kernel returned, not a list baked into this build", async () => {
    respondWith({
      protocol_version: "1",
      server_protocol_version: "1",
      implemented_methods: IMPLEMENTED,
    });

    renderPage();

    expect(await screen.findByText("Catalog negotiated")).toBeVisible();
    expect(screen.getByText(`${IMPLEMENTED.length} methods implemented`)).toBeVisible();
    // `readiness.get` is implemented and read by a surface here.
    expect(screen.getAllByText("Live").length).toBeGreaterThan(0);
    // `readiness.check` is in the target catalog but not this kernel.
    expect(screen.getAllByText("Not implemented in this cell").length).toBeGreaterThan(0);
  });

  it("surfaces methods the kernel implements that this build cannot reach", async () => {
    // The installation running ahead of the interface is a real operational
    // condition; silence would hide it and the operator would never learn the
    // capability exists.
    respondWith({
      protocol_version: "1",
      server_protocol_version: "1",
      implemented_methods: [...IMPLEMENTED, "capability.list"],
    });

    renderPage();

    expect(await screen.findByText("Implemented here, unknown to this workbench")).toBeVisible();
    expect(screen.getByText("capability.list")).toBeVisible();
  });

  it("refuses to present governed state across a protocol major mismatch", async () => {
    // Field meanings are not guaranteed across a major difference, so a
    // plausible render of misread values is worse than no render.
    respondWith({
      protocol_version: "1",
      server_protocol_version: "2",
      implemented_methods: IMPLEMENTED,
    });

    renderPage();

    expect(await screen.findByText("Protocol mismatch")).toBeVisible();
    expect(await screen.findByRole("alert")).toHaveTextContent(/cannot read this server|SFWP v2/);
    expect(screen.queryByText("Live")).not.toBeInTheDocument();
  });

  it("treats an unsupported_version body as a governed answer, not a crash", async () => {
    respondWith({
      error_class: "unsupported_version",
      requested: "1",
      supported: "3",
      error: "unsupported SFWP protocol version 1",
    });

    renderPage();

    expect(await screen.findByText("Protocol mismatch")).toBeVisible();
  });

  it("claims nothing about any method when negotiation fails", async () => {
    // A hello body missing `implemented_methods` is contract drift. The page
    // must report the failure and assert nothing about any method — showing a
    // partial catalog would be a claim the cell never made.
    respondWith({ protocol_version: "1" });

    renderPage();

    expect(await screen.findByText("Catalog unavailable")).toBeVisible();
    expect(await screen.findByRole("alert")).toHaveTextContent(/failed contract validation/);
    expect(screen.queryByText("Live")).not.toBeInTheDocument();
    expect(screen.queryByText("Not implemented in this cell")).not.toBeInTheDocument();
  });
});
