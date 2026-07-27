import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();
const commandMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, args: unknown) =>
    command === "sfwp_command" ? commandMock(args) : invokeMock(command, args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
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

import { DelegationRoster } from "./DelegationRoster";

function delegation(overrides: Record<string, unknown> = {}) {
  return {
    run_id: "run-1",
    case_id: "case-1",
    endpoint_ref: "local",
    standing: "active",
    cancellable: true,
    max_turns: 6,
    ...overrides,
  };
}

function respondWith(delegations: unknown[], unreadable: string[] = []) {
  invokeMock.mockResolvedValue({ delegations, unreadable });
}

function renderRoster() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <DelegationRoster />
    </QueryClientProvider>,
  );
}

/** `ProtectedActionButton` confirms before acting; click through both steps. */
async function cancelRun(runId: string) {
  const trigger = await screen.findByRole("button", { name: `Cancel ${runId}` });
  fireEvent.click(trigger);
  const confirm = await screen.findByRole("button", { name: /confirm/i });
  fireEvent.click(confirm);
}

beforeEach(() => {
  invokeMock.mockReset();
  commandMock.mockReset();
  commandMock.mockResolvedValue({ run_id: "run-1", state: "cancellation_requested" });
});

afterEach(cleanup);

describe("DelegationRoster", () => {
  it("asks for delegation.list with the bare verb the host bridge accepts", async () => {
    respondWith([]);
    renderRoster();
    await screen.findByRole("region", { name: "Delegation roster" });
    expect(invokeMock).toHaveBeenCalledWith("sfwp_query", {
      query: { verb: "delegation_list" },
    });
  });

  /**
   * The control claim of epic 9.7. Cancelling names one run, and no affordance
   * exists that would reach a sibling.
   */
  it("cancels exactly one run and offers no bulk affordance", async () => {
    respondWith([delegation(), delegation({ run_id: "run-2" })]);
    renderRoster();

    await cancelRun("run-1");
    await waitFor(() =>
      expect(commandMock).toHaveBeenCalledWith({
        command: { verb: "cancel_delegation", run_id: "run-1" },
      }),
    );
    expect(commandMock).toHaveBeenCalledTimes(1);

    // The sibling is still independently cancellable.
    expect(screen.getByRole("button", { name: "Cancel run-2" })).toBeEnabled();
    for (const label of [/cancel all/i, /cancel selected/i]) {
      expect(screen.queryByRole("button", { name: label })).toBeNull();
    }
  });

  /**
   * A successful cancel records a *request*. The episode still terminates on its
   * own terms, so reporting it as "cancelled" would substitute an outcome for a
   * request.
   */
  it("reports a successful cancel as requested rather than as cancelled", async () => {
    respondWith([delegation()]);
    renderRoster();
    await cancelRun("run-1");

    const status = await screen.findByRole("status", { name: "Last cancellation outcome" });
    expect(status).toHaveTextContent(/Cancellation requested/);
    expect(status).not.toHaveTextContent(/^run-1: Cancelled\b/);
  });

  it("surfaces a refused cancel as a governed outcome", async () => {
    respondWith([delegation()]);
    commandMock.mockResolvedValue({ error: "delegation run not active" });
    renderRoster();
    await cancelRun("run-1");

    expect(
      await screen.findByRole("status", { name: "Last cancellation outcome" }),
    ).toHaveTextContent("delegation run not active");
  });

  /**
   * The roster's `cancellable` and the kernel's refusal have to agree, or the
   * UI offers an action that cannot succeed.
   */
  it("disables cancel for a delegation the server does not hold", async () => {
    respondWith([
      delegation({ run_id: "run-settled", standing: "settled", cancellable: false }),
    ]);
    renderRoster();

    expect(await screen.findByRole("button", { name: "Cancel run-settled" })).toBeDisabled();
  });

  it("disables cancel once a cancellation has already been requested", async () => {
    respondWith([
      delegation({ standing: "cancellation_requested", cancellable: false }),
    ]);
    renderRoster();

    expect(await screen.findByRole("button", { name: "Cancel run-1" })).toBeDisabled();
  });

  /**
   * Standing and settlement are separate columns because they are separate
   * facts: a settled delegation may have been rejected.
   */
  it("shows a settled-and-rejected delegation as both settled and rejected", async () => {
    respondWith([
      delegation({
        standing: "settled",
        cancellable: false,
        settlement: "rejected",
        termination: "turn_cap_exceeded",
        turns_used: 6,
      }),
    ]);
    renderRoster();

    const row = (await screen.findAllByRole("row"))[1];
    // Only the pills: the action button carries a presentational `data-variant`
    // of its own and is not a governed standing.
    const variants = [...row.querySelectorAll(".status-pill[data-variant]")].map((el) =>
      el.getAttribute("data-variant"),
    );
    expect(variants).toContain("finished");
    // `rejected` maps through the shared `SETTLEMENT_VARIANT` to the blocked
    // variant, as it does everywhere else a settlement is rendered.
    expect(variants).toContain("blocked");
    expect(within(row).getByText(/Turn cap exceeded/i)).toBeVisible();
  });

  /** An unsettled delegation has no verdict — not a failed one. */
  it("says a running delegation has no verdict yet rather than showing a failure", async () => {
    respondWith([delegation()]);
    renderRoster();
    const row = (await screen.findAllByRole("row"))[1];
    expect(within(row).getByText("no verdict yet")).toBeVisible();
  });

  /**
   * The kernel publishes no per-turn state, so a running delegation's turn count
   * is genuinely unknown. Rendering `0` would claim no turn had been taken.
   */
  it("reports a running delegation's turns as unobservable rather than zero", async () => {
    respondWith([delegation()]);
    renderRoster();
    const row = (await screen.findAllByRole("row"))[1];
    expect(within(row).getByText(/not observable while running \(cap 6\)/)).toBeVisible();
    expect(within(row).queryByText("0")).toBeNull();
  });

  it("shows a finished delegation's turns against the cap it was bound to", async () => {
    respondWith([
      delegation({ standing: "settled", cancellable: false, turns_used: 4, max_turns: 6 }),
    ]);
    renderRoster();
    const row = (await screen.findAllByRole("row"))[1];
    expect(within(row).getByText("4")).toBeVisible();
    expect(within(row).getByText("6")).toBeVisible();
  });

  /**
   * The state a server restart produces. It must not read as a terminal
   * outcome, because the kernel recorded none.
   */
  it("renders an unresolved delegation without asserting an outcome", async () => {
    respondWith([
      delegation({ run_id: "run-orphan", standing: "unresolved", cancellable: false }),
    ]);
    renderRoster();

    const row = (await screen.findAllByRole("row"))[1];
    const variants = [...row.querySelectorAll(".status-pill[data-variant]")].map((el) =>
      el.getAttribute("data-variant"),
    );
    expect(variants).toContain("unknown");
    expect(variants).not.toContain("cancelled");
    expect(variants).not.toContain("failed");
    expect(within(row).getByText("no verdict yet")).toBeVisible();
  });

  it("resolves each run to its run record", async () => {
    respondWith([delegation()]);
    renderRoster();
    expect(await screen.findByRole("link", { name: "run-1" })).toHaveAttribute(
      "href",
      "/runs/run-1",
    );
  });

  it("reports unreadable delegation runs as an integrity signal", async () => {
    respondWith([], ["run-broken"]);
    renderRoster();
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("run-broken");
    expect(alert).toHaveTextContent(/unreadable/);
  });

  it("reports an empty roster as a fact about the cell, not a failure", async () => {
    respondWith([]);
    renderRoster();
    expect(
      await screen.findByText(/No delegation has been recorded in this cell/),
    ).toBeVisible();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  /** A governed error body is a denial, not a malformed response. */
  it("surfaces a governed error envelope as an alert rather than a contract failure", async () => {
    invokeMock.mockResolvedValue({
      error: "disclosure not permitted for delegation roster",
      error_class: "disclosure_denied",
    });
    renderRoster();
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("disclosure not permitted for delegation roster");
    expect(alert).not.toHaveTextContent(/contract validation/);
  });
});
