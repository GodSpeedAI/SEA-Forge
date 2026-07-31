import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { ModelsPage, ThothPage } from "./SurfacesPages";

/**
 * These two surfaces used to be *specimens*: layouts copied from the design
 * specification, populated with illustrative content, watermarked, and with
 * every status pill forced to `unknown`. The tests here asserted that fiction
 * was correctly labelled as fiction.
 *
 * Both are gone, so those assertions are too. Thoth calls `thoth.ask` — always
 * implemented in the kernel, but missing from the method catalog, so no client
 * could discover it. Domain models have no kernel method and now say so.
 *
 * The suites for the surfaces that graduated earlier live beside them:
 * `OperationsPage`, `SystemContractPage`, `CaseHorizonPage`,
 * `ApprovalInboxPage`, `AssetCatalogPage`.
 */

function renderPage(Page: () => React.JSX.Element) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <Page />
    </QueryClientProvider>,
  );
}

/** A catalog answer listing exactly `methods` as implemented. */
function catalog(methods: string[]) {
  return {
    protocol_version: "1",
    server_protocol_version: "1",
    implemented_methods: ["system.hello", ...methods],
  };
}

beforeEach(() => {
  invokeMock.mockClear();
  invokeMock.mockResolvedValue(catalog(["readiness.get"]));
});

afterEach(cleanup);

describe("Domain models", () => {
  it("reports the method it needs rather than illustrating one", async () => {
    renderPage(ModelsPage);

    expect(
      screen.getByRole("heading", { level: 1, name: "Domain models" }),
    ).toBeVisible();
    expect(await screen.findByText(/requires domain\.list_models/)).toBeVisible();
  });

  /**
   * The surface must resolve its standing from the negotiated catalog, not
   * from a constant. The whole point is that it stops claiming absence the day
   * the kernel implements the method.
   */
  it("stops reporting absence once the kernel implements the method", async () => {
    invokeMock.mockResolvedValue(catalog(["domain.list_models"]));
    renderPage(ModelsPage);

    expect(
      await screen.findByText(/This cell implements the method/),
    ).toBeVisible();
  });
});

describe("Thoth", () => {
  it("does not render an answer before one has been asked for", async () => {
    invokeMock.mockResolvedValue(catalog(["thoth.ask"]));
    renderPage(ThothPage);

    expect(await screen.findByText(/No answer yet/)).toBeVisible();
    // The removed specimen rendered a plausible answer on first paint, citing
    // a design document as its evidence.
    expect(screen.queryByText(/UX epic/)).not.toBeInTheDocument();
  });

  it("refuses to ask when the cell does not implement the method", async () => {
    invokeMock.mockResolvedValue(catalog([]));
    renderPage(ThothPage);

    expect(await screen.findByRole("button", { name: "Ask" })).toBeDisabled();
  });

  it("asks the kernel and renders the answer it receives", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "sfwp_command") {
        return Promise.resolve({
          answer_id: "ans_1",
          question_id: "q_1",
          disposition: "answered",
          claims: [
            {
              claim_id: "clm_1",
              claim_class: "demonstrated_capability",
              subject: "delegation",
              status: "demonstrated",
              statement: "Delegation has settled twice under variation.",
              snapshot_ref: "snap_1",
              evidence_refs: ["ev_1"],
              settlement_refs: ["set_1"],
            },
          ],
          omitted_claim_classes: [],
          snapshot_ref: "snap_1",
          freshness: "current",
          assurance: "local_tamper_evident",
          limitations: [],
          authority_notice: "This answer confers no execution authority.",
          answered_at: "2026-07-31T00:00:00Z",
        });
      }
      return Promise.resolve(catalog(["thoth.ask"]));
    });

    renderPage(ThothPage);
    // Wait for negotiation to settle before clicking: the button is disabled
    // until the catalog confirms this cell implements the method, which is the
    // behaviour that keeps the surface from offering an action it cannot take.
    expect(await screen.findByText("thoth.ask live")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Ask" }));

    expect(
      await screen.findByText("Delegation has settled twice under variation."),
    ).toBeVisible();
    expect(screen.getByText("local_tamper_evident")).toBeVisible();
    expect(
      screen.getByText(/confers no execution authority/),
    ).toBeVisible();
  });

  /**
   * Epic story 3.9. A partial answer that does not say what it omitted is
   * indistinguishable from a complete one — the most consequential thing this
   * surface can get wrong, because the operator would act on it believing they
   * had the whole picture.
   */
  it("surfaces withheld classes on a partial answer", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "sfwp_command") {
        return Promise.resolve({
          answer_id: "ans_2",
          question_id: "q_2",
          disposition: "partial",
          claims: [],
          omitted_claim_classes: ["security_implementation"],
          snapshot_ref: "snap_1",
          freshness: "current",
          assurance: "local_tamper_evident",
          limitations: ["Disclosure scope excluded one class."],
          authority_notice: "This answer confers no execution authority.",
          answered_at: "2026-07-31T00:00:00Z",
        });
      }
      return Promise.resolve(catalog(["thoth.ask"]));
    });

    renderPage(ThothPage);
    // Wait for negotiation to settle before clicking: the button is disabled
    // until the catalog confirms this cell implements the method, which is the
    // behaviour that keeps the surface from offering an action it cannot take.
    expect(await screen.findByText("thoth.ask live")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Ask" }));

    expect(await screen.findByText("security_implementation")).toBeVisible();
    expect(
      screen.getByText("Disclosure scope excluded one class."),
    ).toBeVisible();
  });
});
