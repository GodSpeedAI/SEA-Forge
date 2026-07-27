import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { AssetsPage, ModelsPage, ThothPage } from "./SurfacesPages";

// Surfaces that have graduated out of this file are covered by their own
// suites: `OperationsPage` (durable events ledger), `AdminPage` →
// `SystemContractPage`, `CasesPage` → `CaseHorizonPage`, and `InboxPage` →
// `ApprovalInboxPage`. What remains here is only what is still illustrative.

function renderPage(Page: () => React.JSX.Element) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <Page />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  invokeMock.mockClear();
  invokeMock.mockResolvedValue({
    protocol_version: "1",
    server_protocol_version: "1",
    implemented_methods: ["system.hello", "readiness.get"],
  });
});

afterEach(cleanup);

describe.each([
  { Page: ThothPage, heading: "Thoth workspace", region: "thoth-question-composer", method: "thoth.ask" },
  { Page: AssetsPage, heading: "Asset catalog", region: "asset-catalog-table", method: "asset.list" },
  {
    Page: ModelsPage,
    heading: "Domain models",
    region: "domain-model-workbench",
    method: "domain_model.list",
  },
])("$heading specimen layout", ({ Page, heading, region, method }) => {
  it("renders the route-specific layout rather than a generic placeholder", async () => {
    const { container } = renderPage(Page);

    expect(screen.getByRole("heading", { level: 1, name: heading })).toBeVisible();
    expect(container.querySelector(`[data-od-id="${region}"]`)).toBeInTheDocument();
    expect(await screen.findByText(new RegExp(`requires ${method.replace(".", "\\.")}`))).toBeVisible();
  });

  it("renders no success pill anywhere, because none of the depicted state is read", async () => {
    // The layouts are copied from the design specification and their states are
    // illustrative. `Status` forces every pill to `unknown` at the component
    // level; a `ready` here would be indistinguishable from an evidenced one.
    const { container } = renderPage(Page);
    await screen.findAllByText(/requires /);

    const variants = [...container.querySelectorAll("[data-variant]")].map((el) =>
      el.getAttribute("data-variant"),
    );
    expect(variants.length).toBeGreaterThan(0);
    expect(variants.every((v) => v === "unknown")).toBe(true);
  });

  it("marks the illustrative region so it is not mistaken for live data", () => {
    const { container } = renderPage(Page);
    expect(container.querySelector('[data-specimen="true"]')).toBeInTheDocument();
  });
});
