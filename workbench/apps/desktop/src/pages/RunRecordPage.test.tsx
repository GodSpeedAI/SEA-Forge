import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

const RUN_ID = "run-01JABCDEF";

vi.mock("@tanstack/react-router", () => ({
  useParams: () => ({ runId: RUN_ID }),
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

import { RunRecordPage } from "./RunRecordPage";

/**
 * The default fixture is the state the page exists to keep legible: a command
 * that exited zero whose settlement nevertheless rejected the work.
 */
function runRecord(overrides: Record<string, unknown> = {}) {
  return {
    run_id: RUN_ID,
    case_id: "case-01JABCDEF",
    plan_item_id: "item-1",
    plan_item_name: "Build the report",
    item_kind: "sandboxed_task",
    execution: "completed",
    settlement: "rejected",
    started_at: "2026-07-26T09:00:00Z",
    finished_at: "2026-07-26T09:00:04Z",
    termination: {
      trace_kind: "command_finished",
      exit_code: 0,
      execution_status: "completed",
      at: "2026-07-26T09:00:02Z",
    },
    criteria: [
      {
        criterion: "require_exit_zero",
        expected: "process exits 0",
        standing: "satisfied",
        basis: ["exit_zero"],
      },
      {
        criterion: "required_artifact",
        expected: "out/report.md",
        standing: "unsatisfied",
        basis: ["required_artifact_missing:out/report.md"],
      },
    ],
    settlement_record: {
      settlement_id: "set-1",
      status: "rejected",
      basis: ["authority_allow", "exit_zero", "required_artifact_missing:out/report.md"],
      review_required: false,
      settled_at: "2026-07-26T09:00:04Z",
    },
    declarations: [],
    evidence: [],
    trace: [],
    records: [
      { record: "plan.json", present: true, bytes: 512 },
      { record: "authority.json", present: false },
    ],
    ...overrides,
  };
}

function respond(body: unknown) {
  invokeMock.mockImplementation(() => Promise.resolve(body));
}

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <RunRecordPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => invokeMock.mockClear());
afterEach(cleanup);

describe("run record", () => {
  /**
   * The load-bearing assertion for epic 12.4. If a future change lets exit code
   * drive the settlement chip (or the reverse), this fails.
   */
  it("shows execution termination and settlement as separate facts", async () => {
    respond(runRecord());
    renderPage();

    const outcome = await screen.findByRole("region", {
      name: "Termination and settlement",
    });

    expect(within(outcome).getByText("Exit code")).toBeVisible();
    expect(within(outcome).getByText("0")).toBeVisible();

    // Execution completed, settlement rejected — two chips, neither rewritten
    // by the other. Asserted on the pills themselves rather than on the text,
    // because "Completed" legitimately appears twice (the chip and the recorded
    // execution status) and only the chip carries the standing.
    const pills = [...outcome.querySelectorAll("[data-variant]")].map((el) =>
      el.getAttribute("data-variant"),
    );
    expect(pills).toEqual(["degraded", "blocked"]);
  });

  it("never renders a success pill for execution that merely finished", async () => {
    respond(runRecord());
    renderPage();

    const outcome = await screen.findByRole("region", {
      name: "Termination and settlement",
    });

    // `completed` maps to `degraded`, never `ready`: finishing is not accepting.
    // Scoped to the outcome panel, because a *satisfied criterion* elsewhere on
    // the page is legitimately `ready` — the rule is about execution standing.
    expect(outcome.querySelector('[data-variant="ready"]')).toBeNull();
  });

  it("pairs each criterion with the settlement basis that decided it", async () => {
    respond(runRecord());
    renderPage();

    const table = await screen.findByRole("region", { name: "Criteria and evidence" });
    expect(within(table).getByText("exit_zero")).toBeVisible();
    expect(within(table).getByText("required_artifact_missing:out/report.md")).toBeVisible();
    expect(within(table).getByText("Satisfied")).toBeVisible();
    expect(within(table).getByText("Unsatisfied")).toBeVisible();
  });

  /**
   * An unsettled run must read as unknown, not as passing. This is the
   * distinction that stops an in-flight run from looking successful.
   */
  it("reports an undecided criterion as unavailable rather than satisfied", async () => {
    respond(
      runRecord({
        settlement: "unsettled",
        settlement_record: undefined,
        criteria: [
          {
            criterion: "require_exit_zero",
            expected: "process exits 0",
            standing: "unavailable",
            basis: [],
          },
        ],
      }),
    );
    renderPage();

    const table = await screen.findByRole("region", { name: "Criteria and evidence" });
    expect(within(table).getByText("Unavailable")).toBeVisible();
    expect(
      within(table).getByText(/recorded nothing deciding this criterion/i),
    ).toBeVisible();
    expect(within(table).queryByText("Satisfied")).not.toBeInTheDocument();
  });

  it("reports an absent source record as absent rather than omitting it", async () => {
    respond(runRecord());
    renderPage();

    const inventory = await screen.findByRole("region", { name: "Record inventory" });
    expect(within(inventory).getByText("authority.json")).toBeVisible();
    expect(within(inventory).getByText("not present")).toBeVisible();
  });

  /**
   * `not_found` and `record_unreadable` need different operator responses, so
   * the page must not flatten them into one failure message.
   */
  it("distinguishes a missing run from an unreadable one", async () => {
    respond({ error: "no run record for run-x", error_class: "not_found" });
    renderPage();

    expect(await screen.findByRole("alert")).toHaveTextContent("No such run");
    expect(screen.getByRole("alert")).toHaveTextContent("no run record for run-x");
  });

  it("names an unreadable record as an integrity signal, not an absence", async () => {
    respond({
      error: "run run-x directory exists but holds no readable trace, plan, or settlement record",
      error_class: "record_unreadable",
    });
    renderPage();

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Run record unreadable");
    expect(alert).toHaveTextContent(/integrity signal, not an absence/);
  });
});
