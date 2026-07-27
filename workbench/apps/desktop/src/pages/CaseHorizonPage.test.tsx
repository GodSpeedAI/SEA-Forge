import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

// Render `Link` as a plain anchor so the page can be tested without standing up
// a router. The assertion that matters is *which* route a run id points at, and
// an `href` carries that faithfully.
vi.mock("@tanstack/react-router", () => ({
  Link: ({
    to,
    params,
    children,
    ...rest
  }: {
    to: string;
    params?: Record<string, string>;
    children: React.ReactNode;
  }) => {
    const href = Object.entries(params ?? {}).reduce(
      (path, [key, value]) => path.replace(`$${key}`, value),
      to,
    );
    return (
      <a href={href} {...rest}>
        {children}
      </a>
    );
  },
}));

import { CaseHorizonPage } from "./CaseHorizonPage";

const CASE_ID = "case-01JABCDEF";

function caseSummary(overrides: Record<string, unknown> = {}) {
  return {
    case_id: CASE_ID,
    case_state: "active",
    summary: "Repair recurring service incidents",
    created_at: "2026-07-26T09:00:00Z",
    run_count: 1,
    ...overrides,
  };
}

function horizonItem(overrides: Record<string, unknown> = {}) {
  return {
    plan_item_id: "item-1",
    name: "Run repair command",
    item_kind: "sandboxed_task",
    execution: "pending",
    settlement: "unsettled",
    depends_on: [],
    run_ids: [],
    ...overrides,
  };
}

/** Route each SFWP verb to its own canned body. */
function respond({
  list,
  overview,
  horizon,
  approvals,
}: {
  list?: unknown;
  overview?: unknown;
  horizon?: unknown;
  approvals?: unknown;
}) {
  invokeMock.mockImplementation((...args: unknown[]) => {
    const verb = (args[1] as { query?: { verb?: string } } | undefined)?.query?.verb;
    switch (verb) {
      case "case_list":
        return Promise.resolve(list ?? { cases: [], unreadable: [] });
      case "case_get_overview":
        return Promise.resolve(
          overview ?? {
            case_id: CASE_ID,
            case_state: "active",
            summary: "Repair recurring service incidents",
            created_at: "2026-07-26T09:00:00Z",
            item_count: 1,
            stages: [],
            settlements: [],
            run_ids: [],
          },
        );
      case "case_get_horizon":
        return Promise.resolve(
          horizon ?? {
            case_id: CASE_ID,
            case_state: "active",
            items: [horizonItem()],
            events_folded: 0,
          },
        );
      case "approval_list":
        return Promise.resolve(approvals ?? { approvals: [] });
      default:
        return Promise.resolve({});
    }
  });
}

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <CaseHorizonPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => invokeMock.mockClear());
afterEach(cleanup);

describe("case horizon", () => {
  it("renders committed cases read from the kernel, not a fixture", async () => {
    respond({ list: { cases: [caseSummary()], unreadable: [] } });
    renderPage();

    expect(await screen.findByText("Repair recurring service incidents")).toBeVisible();
    expect(screen.getByText(CASE_ID)).toBeVisible();
  });

  it("reports an empty cell as an empty cell, not as a failure to read it", async () => {
    respond({ list: { cases: [], unreadable: [] } });
    renderPage();

    expect(await screen.findByText(/No case has been committed in this cell yet/)).toBeVisible();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  /**
   * The central guarantee: a finished command is not accepted work. If these
   * ever share a pill variant, a process exit becomes indistinguishable from a
   * governed settlement.
   */
  it("never renders execution completion as a settlement success", async () => {
    respond({
      list: { cases: [caseSummary()], unreadable: [] },
      horizon: {
        case_id: CASE_ID,
        case_state: "active",
        items: [horizonItem({ execution: "completed", settlement: "unsettled" })],
        events_folded: 3,
      },
    });
    const { container } = renderPage();

    await screen.findByTestId("dual-state-indicator");

    const indicator = container.querySelector('[data-testid="dual-state-indicator"]');
    const variants = [...(indicator?.querySelectorAll("[data-variant]") ?? [])].map((el) =>
      el.getAttribute("data-variant"),
    );
    expect(variants).toHaveLength(2);
    // Execution completed is not a success; settlement is unknown, not ready.
    expect(variants).not.toContain("ready");
  });

  it("shows an accepting settlement as the only success", async () => {
    respond({
      list: { cases: [caseSummary()], unreadable: [] },
      horizon: {
        case_id: CASE_ID,
        case_state: "completed",
        items: [horizonItem({ execution: "completed", settlement: "accepted" })],
        events_folded: 4,
      },
    });
    const { container } = renderPage();

    await screen.findByTestId("dual-state-indicator");
    const indicator = container.querySelector('[data-testid="dual-state-indicator"]');
    const variants = [...(indicator?.querySelectorAll("[data-variant]") ?? [])].map((el) =>
      el.getAttribute("data-variant"),
    );
    expect(variants).toContain("ready");
  });

  /**
   * A record that exists but cannot be parsed is an integrity signal. Folding it
   * into "no cases" would hide it completely.
   */
  it("reports unreadable case records rather than counting them as absent", async () => {
    respond({ list: { cases: [], unreadable: ["case-broken"] } });
    renderPage();

    expect(await screen.findByRole("alert")).toHaveTextContent(/could not be parsed/);
    expect(screen.getByRole("alert")).toHaveTextContent("case-broken");
  });

  /**
   * Every retry is its own immutable run (epic 7.5), and each must be openable
   * — a run id an operator can see but not resolve leaves epic invariant 4
   * unsatisfiable. Counting the episodes instead of linking them was the gap.
   */
  it("links every run episode to its own record rather than counting them", async () => {
    respond({
      list: { cases: [caseSummary()], unreadable: [] },
      horizon: {
        case_id: CASE_ID,
        case_state: "active",
        items: [horizonItem({ run_ids: ["run-a", "run-b"] })],
        events_folded: 4,
      },
    });
    renderPage();

    const first = await screen.findByRole("link", { name: "Attempt 1" });
    expect(first).toHaveAttribute("href", "/runs/run-a");
    expect(screen.getByRole("link", { name: "Attempt 2" })).toHaveAttribute(
      "href",
      "/runs/run-b",
    );
  });

  it("surfaces a governed read failure with the server's own message", async () => {
    invokeMock.mockImplementation((...args: unknown[]) => {
      const verb = (args[1] as { query?: { verb?: string } } | undefined)?.query?.verb;
      if (verb === "case_list") {
        return Promise.resolve({ error: "no case record for case-x", error_class: "not_found" });
      }
      return Promise.resolve({});
    });
    renderPage();

    expect(await screen.findByRole("alert")).toHaveTextContent(/no case record for case-x/);
  });
});
