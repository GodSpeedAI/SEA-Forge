import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReadinessView } from "@sea-forge/contracts";

// `ReadinessPage` navigates to `/cases/new` on "Create case" (Task 6); mock
// `useNavigate` so the page renders without a full `RouterProvider` tree.
const navigateMock = vi.fn();
vi.mock("@tanstack/react-router", async () => {
  const actual =
    await vi.importActual<typeof import("@tanstack/react-router")>("@tanstack/react-router");
  return { ...actual, useNavigate: () => navigateMock };
});

// --- Bridge mocks -----------------------------------------------------------
// The renderer talks to the host only through the closed `sfwp_query` command
// and the `sfwp://event` stream. We mock both boundaries directly (the
// installed @tauri-apps/api ships a ./mocks module, but mocking the two thin
// entry points keeps the test deterministic and framework-agnostic).
const invokeMock = vi.fn();
const listenMock = vi.fn();
let emittedListener: (() => void) | undefined;

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (_event: string, handler: () => void) => {
    emittedListener = handler;
    return listenMock(_event, handler) ?? Promise.resolve(() => {});
  },
}));

import { ReadinessPage } from "./ReadinessPage";

function readyView(): ReadinessView {
  return {
    overall: "ready",
    intended_operation: undefined,
    foundations: [
      {
        id: "self_model_integrity",
        name: "Self-model integrity",
        category: "foundation",
        status: "ready",
        reason: "",
        source_ref: "sea-forge-self-model/src/store.rs::validate",
      },
    ],
    operational_capabilities: [
      {
        id: "local_governed_execution",
        name: "Local governed execution",
        category: "operational_capability",
        status: "ready",
        reason: "",
        source_ref: "sea-forge-server::case_dispatch",
      },
      {
        id: "external_delegation",
        name: "External delegation",
        category: "operational_capability",
        status: "ready",
        reason: "",
        source_ref: "sea-forge-agent::AgentConfig::endpoints",
      },
    ],
    recent_invalidations: [],
  };
}

function integrityHaltedView(): ReadinessView {
  return {
    overall: "integrity_halted",
    intended_operation: undefined,
    foundations: [
      {
        id: "self_model_integrity",
        name: "Self-model integrity",
        category: "foundation",
        status: "blocked",
        reason: "snapshot hash chain verification failed",
        source_ref: "sea-forge-self-model/src/store.rs::validate",
      },
    ],
    operational_capabilities: [
      {
        id: "local_governed_execution",
        name: "Local governed execution",
        category: "operational_capability",
        status: "blocked",
        reason: "Blocked: self-model foundation is not trusted",
        source_ref: "sea-forge-server::case_dispatch",
      },
    ],
    recent_invalidations: [],
  };
}

function renderPage() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <ReadinessPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  invokeMock.mockReset();
  listenMock.mockReset();
  listenMock.mockResolvedValue(() => {});
  navigateMock.mockReset();
  emittedListener = undefined;
});

afterEach(() => {
  cleanup();
});

describe("ReadinessPage", () => {
  it("mirrors the Readiness mockup's governed-focus and operational region structure", async () => {
    invokeMock.mockResolvedValue(readyView());
    renderPage();

    await waitFor(() =>
      expect(document.querySelector('[data-variant="ready"]')).toBeInTheDocument(),
    );

    expect(
      document.querySelector('[data-od-id="readiness-governed-focus"]'),
    ).toBeInTheDocument();
    expect(
      document.querySelector('[data-od-id="intended-work-selector"]'),
    ).toBeInTheDocument();
    expect(
      document.querySelector('[data-od-id="critical-foundations"]'),
    ).toBeInTheDocument();
    expect(
      document.querySelector('[data-od-id="operational-capabilities"]'),
    ).toBeInTheDocument();
    expect(
      document.querySelector('[data-od-id="current-affordance-rail"]'),
    ).toBeInTheDocument();

    expect(screen.getByRole("radiogroup", { name: "Intended operation" })).toBeInTheDocument();
    expect(screen.getByRole("table", { name: "Critical foundations" })).toBeInTheDocument();
  });

  it("(a) ready state renders ready pill and an enabled case-creation button that navigates to /cases/new", async () => {
    invokeMock.mockResolvedValue(readyView());
    renderPage();

    // Wait for the resolved view: the overall pill flips to the "ready" variant.
    await waitFor(() =>
      expect(document.querySelector('[data-variant="ready"]')).toBeInTheDocument(),
    );

    // Task 6: case authoring exists now, so a fully-ready projection enables it.
    const button = screen.getByRole("button", { name: /create case/i });
    expect(button).not.toBeDisabled();
    button.click();
    expect(navigateMock).toHaveBeenCalledWith({ to: "/cases/new" });
  });

  it("(b) integrity-halted state renders the blocking reason and a compromised integrity indicator", async () => {
    invokeMock.mockResolvedValue(integrityHaltedView());
    renderPage();

    // Wait for the resolved view to drive the indicator to "compromised"
    // (it renders "checking" while the query is still in flight).
    await waitFor(() =>
      expect(screen.getByTestId("integrity-indicator")).toHaveAttribute(
        "data-status",
        "compromised",
      ),
    );

    // The blocking reason from the view is surfaced (not a generic error).
    const surfaced = await screen.findAllByText(/hash chain verification failed/i);
    expect(surfaced.length).toBeGreaterThan(0);

    // Case creation reason is sourced from the first non-ready item.
    const reason = await screen.findByTestId("disabled-reason");
    expect(reason).toHaveTextContent(/hash chain verification failed/i);
  });

  it("(c) a simulated sfwp event triggers a readiness refetch", async () => {
    invokeMock.mockResolvedValue(readyView());
    renderPage();

    await screen.findByTestId("integrity-indicator");
    await waitFor(() => expect(listenMock).toHaveBeenCalledWith("sfwp://event", expect.any(Function)));
    const callsBefore = invokeMock.mock.calls.length;

    // Fire the captured event handler as the host would.
    expect(emittedListener).toBeTypeOf("function");
    emittedListener!();

    await waitFor(() => expect(invokeMock.mock.calls.length).toBeGreaterThan(callsBefore));
  });

  it("(d) disconnect after one success keeps prior data and marks the source stale", async () => {
    invokeMock.mockResolvedValueOnce(readyView());
    renderPage();

    // First successful load: source is live.
    await screen.findByText(/Source current/i);

    // Next fetch errors (socket disconnected string from the host).
    invokeMock.mockRejectedValue("SocketError::Disconnected");
    emittedListener?.(); // any event triggers the failing refetch

    // Prior data still rendered; badge flips to stale.
    await waitFor(() => expect(screen.getByText(/Stale projection/i)).toBeInTheDocument());
    // The last-known-good view is still on screen (integrity indicator present).
    expect(screen.getByTestId("integrity-indicator")).toBeInTheDocument();
  });
});
