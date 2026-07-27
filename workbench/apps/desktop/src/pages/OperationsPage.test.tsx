import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { EventFrame, RequestRecord } from "@sea-forge/contracts";

// The renderer reaches the host only through `sfwp_query` / `sfwp_request_status`
// and the `sfwp://event` stream. Mocking those two entry points keeps the test
// deterministic, matching `ReadinessPage.test.tsx`.
const invokeMock = vi.fn();
let emitEvent: (() => void) | undefined;

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (_event: string, handler: () => void) => {
    emitEvent = handler;
    return Promise.resolve(() => {});
  },
}));

import { OperationsPage } from "./OperationsPage";
import { settlementOf } from "../hooks/useOperationsStream";

function frame(cursor: string, kind: string, extra: Partial<EventFrame> = {}): EventFrame {
  return {
    cursor,
    kind,
    committed_at: "2026-07-24T10:00:00Z",
    detail: {},
    ...extra,
  };
}

/** Mirrors the server's `{"events": [...]}` envelope. */
function events(...frames: EventFrame[]) {
  return { events: frames };
}

function renderPage() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={client}>
      <OperationsPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  invokeMock.mockReset();
  emitEvent = undefined;
});

afterEach(cleanup);

describe("operations event stream", () => {
  it("renders durable frames read from the ledger", async () => {
    invokeMock.mockResolvedValue(events(frame("01H0", "case.submitted", { case_id: "C-1" })));

    renderPage();

    expect(await screen.findByText("case.submitted")).toBeVisible();
    expect(screen.getByText("01H0")).toBeVisible();
    expect(screen.getByText(/case C-1/)).toBeVisible();
  });

  it("renders an unrecognised event kind as unknown, never as a success", async () => {
    // The decisive governance property: a kind this renderer has never seen must
    // not inherit a success appearance just because it arrived without error.
    invokeMock.mockResolvedValue(events(frame("01H1", "settlement.evaluated")));

    const { container } = renderPage();

    expect(await screen.findByText("settlement.evaluated")).toBeVisible();
    const pill = container.querySelector('.event-frame [data-variant]');
    expect(pill).toHaveAttribute("data-variant", "unknown");
  });

  it("rejects a frame that fails contract validation instead of dropping it", async () => {
    // A frame missing `committed_at` is contract drift. Silently skipping it
    // would be exactly the silent loss this stream exists to prevent.
    invokeMock.mockResolvedValue({ events: [{ cursor: "01H2", kind: "case.submitted" }] });

    renderPage();

    expect(await screen.findByRole("alert")).toHaveTextContent(/failed contract validation/);
  });

  it("recovers frames missed by the live stream on the next authoritative read", async () => {
    // Gap recovery: the live notification carries no data, so a frame that was
    // never announced still arrives on the re-read the notification triggers.
    invokeMock.mockResolvedValueOnce(events(frame("01H0", "case.submitted")));
    renderPage();
    await screen.findByText("case.submitted");

    invokeMock.mockResolvedValueOnce(events(frame("01H1", "approval.approved")));
    await act(async () => {
      emitEvent?.();
    });

    // Both the original and the never-announced frame are present.
    expect(await screen.findByText("approval.approved")).toBeVisible();
    expect(screen.getByText("case.submitted")).toBeVisible();
  });

  it("reads forward from the last held cursor rather than re-reading from zero", async () => {
    invokeMock.mockResolvedValueOnce(events(frame("01H0", "case.submitted")));
    renderPage();
    await screen.findByText("case.submitted");

    invokeMock.mockResolvedValueOnce(events());
    await act(async () => {
      emitEvent?.();
    });

    await waitFor(() => expect(invokeMock).toHaveBeenCalledTimes(2));
    const secondQuery = invokeMock.mock.calls[1]?.[1] as { query: { from_cursor?: string } };
    expect(secondQuery.query.from_cursor).toBe("01H0");
  });

  it("keeps the last confirmed frames and marks them stale when a re-read fails", async () => {
    invokeMock.mockResolvedValueOnce(events(frame("01H0", "case.submitted")));
    renderPage();
    await screen.findByText("case.submitted");

    invokeMock.mockRejectedValueOnce(new Error("socket closed"));
    await act(async () => {
      emitEvent?.();
    });

    // Blanking the view would lose operator context; presenting it as current
    // would be a lie. It stays, labelled.
    expect(await screen.findByText(/Last authoritative read failed/)).toBeVisible();
    expect(screen.getByText("case.submitted")).toBeVisible();
  });
});

describe("execution and settlement standing", () => {
  function record(over: Partial<RequestRecord> = {}): RequestRecord {
    return {
      request_id: "REQ-1",
      method: "case.submit",
      status: "completed",
      submitted_at: "2026-07-24T10:00:00Z",
      ...over,
    };
  }

  it("reports a rejecting outcome as a settlement rejection, not an execution failure", () => {
    // Plan Task 8: a request can execute successfully and still settle rejected.
    const standing = settlementOf(record({ outcome: { error: "policy PB-14 denied" } }));
    expect(standing.kind).toBe("rejected");
    expect(standing.reason).toBe("policy PB-14 denied");
  });

  it("does not infer settlement from an outcome it cannot classify", () => {
    expect(settlementOf(record({ outcome: {} })).kind).toBe("not_projected");
    expect(settlementOf(record({ status: "pending" })).kind).toBe("not_projected");
    expect(settlementOf(undefined).kind).toBe("not_projected");
  });

  it("renders execution and settlement as separate standings", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "sfwp_request_status") {
        return Promise.resolve(record({ outcome: { error: "policy PB-14 denied" } }));
      }
      return Promise.resolve(events());
    });

    renderPage();
    fireEvent.change(await screen.findByLabelText("Request id"), {
      target: { value: "REQ-1" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Inspect" }));

    // Collapsing these into one pill would report the rejection as a success.
    expect(await screen.findByText("Execution succeeded")).toBeVisible();
    expect(screen.getByText("Settlement rejected")).toBeVisible();
  });
});
