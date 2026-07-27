import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tanstack/react-router", () => ({
  Link: ({
    to,
    params,
    children,
  }: {
    to: string;
    params?: Record<string, string>;
    children: React.ReactNode;
  }) => <a href={to.replace("$runId", params?.runId ?? "")}>{children}</a>,
}));

import { AssetCatalogPage } from "./AssetCatalogPage";

function catalog(assets: unknown[], unreadable: string[] = []) {
  return { assets, unreadable };
}

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <AssetCatalogPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  invokeMock.mockReset();
});

afterEach(cleanup);

describe("AssetCatalogPage", () => {
  it("asks for asset.list with the bare verb the host bridge accepts", async () => {
    invokeMock.mockResolvedValue(catalog([]));
    renderPage();
    await screen.findByRole("heading", { level: 2, name: "Plan templates" });
    expect(invokeMock).toHaveBeenCalledWith("sfwp_query", { query: { verb: "asset_list" } });
  });

  it("renders each kind in its own table so the standings are not read as one ladder", async () => {
    invokeMock.mockResolvedValue(
      catalog([
        {
          asset_id: "template:repair@1.0.0",
          kind: "plan_template",
          name: "repair",
          version: "1.0.0",
          standing: "materialized",
        },
        {
          asset_id: "agent_endpoint:local",
          kind: "agent_endpoint",
          name: "local",
          standing: "declared",
        },
        {
          asset_id: "extension:acme.linter@2.1.0",
          kind: "extension",
          name: "acme.linter",
          version: "2.1.0",
          standing: "active",
        },
      ]),
    );
    renderPage();

    const templates = await screen.findByRole("region", { name: "Plan templates" });
    expect(within(templates).getByText("repair")).toBeVisible();
    expect(within(templates).queryByText("local")).toBeNull();

    const endpoints = screen.getByRole("region", { name: "Agent endpoints" });
    expect(within(endpoints).getByText("local")).toBeVisible();

    const extensions = screen.getByRole("region", { name: "Extensions" });
    expect(within(extensions).getByText("acme.linter")).toBeVisible();
  });

  /**
   * The invariant the epic hinges on here. A `declared` endpoint has proven
   * nothing, so it must not carry the success pill a `demonstrated` one does —
   * this is the exact claim the copied specimen layout used to fabricate.
   */
  it("gives a demonstrated endpoint a success pill and a declared one none", async () => {
    invokeMock.mockResolvedValue(
      catalog([
        {
          asset_id: "agent_endpoint:proven",
          kind: "agent_endpoint",
          name: "proven",
          standing: "demonstrated",
          evidence_refs: ["run:run-1"],
        },
        {
          asset_id: "agent_endpoint:fresh",
          kind: "agent_endpoint",
          name: "fresh",
          standing: "declared",
        },
      ]),
    );
    renderPage();

    const endpoints = await screen.findByRole("region", { name: "Agent endpoints" });
    const rows = within(endpoints).getAllByRole("row").slice(1);
    const variantOf = (row: HTMLElement) =>
      row.querySelector("[data-variant]")?.getAttribute("data-variant");

    const [proven, fresh] = rows;
    expect(within(proven).getByText("proven")).toBeVisible();
    expect(variantOf(proven)).toBe("ready");
    expect(within(fresh).getByText("fresh")).toBeVisible();
    expect(variantOf(fresh)).toBe("unknown");
  });

  /**
   * Standing and blocking are different questions. An extension can be `active`
   * and still refused, and folding the two into one pill would have to drop one
   * of the two facts.
   */
  it("shows an active-but-quarantined extension as active and blocked at once", async () => {
    invokeMock.mockResolvedValue(
      catalog([
        {
          asset_id: "extension:acme.risky@0.1.0",
          kind: "extension",
          name: "acme.risky",
          version: "0.1.0",
          standing: "active",
          blocking_reason: "extension trust level is quarantined",
        },
      ]),
    );
    renderPage();

    const extensions = await screen.findByRole("region", { name: "Extensions" });
    const row = within(extensions).getAllByRole("row")[1];
    const variants = [...row.querySelectorAll("[data-variant]")].map((el) =>
      el.getAttribute("data-variant"),
    );
    expect(variants).toContain("ready");
    expect(variants).toContain("blocked");
    expect(within(row).getByText(/trust level is quarantined/)).toBeVisible();
  });

  it("says nothing blocks an asset rather than implying it is proven", async () => {
    invokeMock.mockResolvedValue(
      catalog([
        {
          asset_id: "agent_endpoint:fresh",
          kind: "agent_endpoint",
          name: "fresh",
          standing: "declared",
        },
      ]),
    );
    renderPage();
    const endpoints = await screen.findByRole("region", { name: "Agent endpoints" });
    expect(within(endpoints).getByText("nothing blocks it")).toBeVisible();
  });

  it("resolves a run evidence ref to the run record", async () => {
    invokeMock.mockResolvedValue(
      catalog([
        {
          asset_id: "agent_endpoint:local",
          kind: "agent_endpoint",
          name: "local",
          standing: "demonstrated",
          evidence_refs: ["run:run-1", "01JLEDGERULID"],
        },
      ]),
    );
    renderPage();

    const endpoints = await screen.findByRole("region", { name: "Agent endpoints" });
    expect(within(endpoints).getByRole("link", { name: "run:run-1" })).toHaveAttribute(
      "href",
      "/runs/run-1",
    );
    // A ledger id has no view of its own, so it must not become a link that
    // goes nowhere.
    expect(within(endpoints).queryByRole("link", { name: "01JLEDGERULID" })).toBeNull();
  });

  it("reports unreadable asset sources as an integrity signal, not as absence", async () => {
    invokeMock.mockResolvedValue(catalog([], ["template:broken@9.9.9"]));
    renderPage();
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("template:broken@9.9.9");
    expect(alert).toHaveTextContent(/unreadable/);
  });

  it("reports an empty catalog as a fact about the cell, not a failure", async () => {
    invokeMock.mockResolvedValue(catalog([]));
    renderPage();
    expect(
      await screen.findByText(/No template has been materialized in this cell/),
    ).toBeVisible();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  /** A governed error body is a denial, not a malformed response. */
  it("surfaces a governed error envelope as an alert rather than a contract failure", async () => {
    invokeMock.mockResolvedValue({
      error: "disclosure not permitted for asset catalog",
      error_class: "disclosure_denied",
    });
    renderPage();
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("disclosure not permitted for asset catalog");
    expect(alert).not.toHaveTextContent(/contract validation/);
  });
});
