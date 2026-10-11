import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReadinessView } from "@sea-forge/contracts";

// `ReadinessPage` navigates to `/cases/new` on "Create case" (Task 6); mock
// `useNavigate` so the page renders without a full `RouterProvider` tree.
const navigateMock = vi.fn();
const inspectEvidenceMock = vi.fn();
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
import { EvidenceContextProvider } from "../shell/EvidenceContext";
import type { SupervisionState } from "../hooks/useIdentity";

const committedSnapshotSource = {
  ledger_id: "self-model",
  entry_id: "led_01J00000000000000000000000",
  record_kind: "self_model_snapshot",
  record_id: "smsnap_01J00000000000000000000000",
  digest: "sha256:readiness-test-snapshot",
  freshness: "current" as const,
  rebuild_standing: "verified" as const,
};

function resolvedIdentity() {
  return {
    available: [{ actor_id: "operator_a", roles: ["operator"] }],
    configured: true,
  };
}

function unresolvedIdentity() {
  return {
    available: [],
    configured: false,
    refusal: {
      error_class: "identity_unconfigured",
      message: "this cell configures no identity bindings",
    },
  };
}

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
        source: committedSnapshotSource,
        next_lawful_action: "Continue with a governed operation",
      },
    ],
    operational_capabilities: [
      {
        id: "local_governed_execution",
        name: "Local governed execution",
        category: "operational_capability",
        status: "ready",
        reason: "",
        source: committedSnapshotSource,
        next_lawful_action: "Create or inspect a governed case",
      },
      {
        id: "external_delegation",
        name: "External delegation",
        category: "operational_capability",
        status: "unknown",
        reason: "No committed endpoint verification record is available",
        source: undefined,
        next_lawful_action: "Run a governed endpoint probe and inspect its committed settlement",
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
        source: undefined,
        next_lawful_action: "Repair the reported integrity failure, then re-check readiness",
      },
    ],
    operational_capabilities: [
      {
        id: "local_governed_execution",
        name: "Local governed execution",
        category: "operational_capability",
        status: "blocked",
        reason: "Blocked: self-model foundation is not trusted",
        source: undefined,
        next_lawful_action: "Repair the reported integrity failure, then re-check readiness",
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
      <EvidenceContextProvider value={{ inspectEvidence: inspectEvidenceMock }}>
        <ReadinessPage />
      </EvidenceContextProvider>
    </QueryClientProvider>,
  );
}

function mockBridge(
  readiness: ReadinessView,
  identity = resolvedIdentity(),
  cell: {
    root: string;
    socket_path: string;
    supervision?: SupervisionState;
  } = { root: "/tmp/sea-forge-test-cell", socket_path: "/tmp/test.sock" },
) {
  invokeMock.mockImplementation((command: string) => {
    if (command === "sfwp_query") return Promise.resolve(readiness);
    if (command === "sfwp_identity") return Promise.resolve(identity);
    if (command === "sfwp_cell") {
      return Promise.resolve(cell);
    }
    return Promise.reject(new Error(`unexpected bridge command: ${command}`));
  });
}

beforeEach(() => {
  invokeMock.mockReset();
  listenMock.mockReset();
  listenMock.mockResolvedValue(() => {});
  navigateMock.mockReset();
  inspectEvidenceMock.mockReset();
  emittedListener = undefined;
});

afterEach(() => {
  cleanup();
});

describe("ReadinessPage", () => {
  it("mirrors the Readiness mockup's governed-focus and operational region structure", async () => {
    mockBridge(readyView());
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
    mockBridge(readyView());
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
    mockBridge(integrityHaltedView());
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
    mockBridge(readyView());
    renderPage();

    await screen.findByTestId("integrity-indicator");
    await waitFor(() => expect(listenMock).toHaveBeenCalledWith("sfwp://event", expect.any(Function)));
    const callsBefore = invokeMock.mock.calls.length;

    // Fire the captured event handler as the host would.
    expect(emittedListener).toBeTypeOf("function");
    emittedListener!();

    await waitFor(() => expect(invokeMock.mock.calls.length).toBeGreaterThan(callsBefore));
  });

  it("opens the committed ledger record for a readiness source, never a code citation", async () => {
    mockBridge(readyView());
    renderPage();

    const source = await screen.findByRole("button", {
      name: "Inspect evidence for Self-model integrity",
    });
    source.click();

    expect(inspectEvidenceMock).toHaveBeenCalledWith(
      expect.objectContaining({
        id: committedSnapshotSource.entry_id,
        kind: "self_model_snapshot",
      }),
    );
    expect(inspectEvidenceMock.mock.calls[0][0].rawPayload).not.toContain("store.rs");
  });

  it("(d) disconnect after one success keeps prior data and marks the source stale", async () => {
    mockBridge(readyView());
    renderPage();

    // First successful load: source is live.
    await screen.findByText(/Source current/i);

    // Next fetch errors (socket disconnected string from the host).
    invokeMock.mockImplementation((command: string) =>
      command === "sfwp_query"
        ? Promise.reject("SocketError::Disconnected")
        : command === "sfwp_identity"
          ? Promise.resolve(resolvedIdentity())
          : Promise.resolve({ root: "/tmp/sea-forge-test-cell", socket_path: "/tmp/test.sock" }),
    );
    emittedListener?.(); // any event triggers the failing refetch

    // Prior data still rendered; badge flips to stale.
    await waitFor(() => expect(screen.getByText(/Stale projection/i)).toBeInTheDocument());
    // The last-known-good view is still on screen (integrity indicator present).
    expect(screen.getByTestId("integrity-indicator")).toBeInTheDocument();
  });

  it("blocks case creation with typed identity_unresolved context when no server-derived actor is available", async () => {
    mockBridge(readyView(), unresolvedIdentity());
    renderPage();

    const button = await screen.findByRole("button", { name: /create case/i });
    expect(button).toBeDisabled();
    expect(screen.getByTestId("disabled-reason")).toHaveTextContent(/identity_unresolved/i);
    expect(screen.getByTestId("disabled-reason")).toHaveTextContent(/no case will be created/i);
  });

  it("makes fresh-cell initialization an explicit, spendable action before the host writes records", async () => {
    mockBridge(readyView(), resolvedIdentity(), {
      root: "/tmp/new-sea-forge-cell",
      socket_path: "/tmp/new-sea-forge-cell/server.sock",
      supervision: { state: "initialization_required" },
    });
    invokeMock.mockImplementation((command: string) => {
      if (command === "sfwp_query") return Promise.resolve(readyView());
      if (command === "sfwp_identity") return Promise.resolve(resolvedIdentity());
      if (command === "sfwp_cell") {
        return Promise.resolve({
          root: "/tmp/new-sea-forge-cell",
          socket_path: "/tmp/new-sea-forge-cell/server.sock",
          supervision: { state: "initialization_required" },
        });
      }
      if (command === "sfwp_initialize_cell") return Promise.resolve({ state: "supervised", pid: 42 });
      return Promise.reject(new Error(`unexpected bridge command: ${command}`));
    });
    renderPage();

    const initialize = await screen.findByRole("button", { name: "Initialize this cell" });
    expect(screen.getByText(/No records have been created yet/i)).toBeInTheDocument();
    initialize.click();

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("sfwp_initialize_cell"),
    );
  });

  it("keeps initialization retryable and explains a host refusal", async () => {
    mockBridge(readyView(), resolvedIdentity(), {
      root: "/tmp/new-sea-forge-cell",
      socket_path: "/tmp/new-sea-forge-cell/server.sock",
      supervision: { state: "initialization_required" },
    });
    invokeMock.mockImplementation((command: string) => {
      if (command === "sfwp_query") return Promise.resolve(readyView());
      if (command === "sfwp_identity") return Promise.resolve(resolvedIdentity());
      if (command === "sfwp_cell") {
        return Promise.resolve({
          root: "/tmp/new-sea-forge-cell",
          socket_path: "/tmp/new-sea-forge-cell/server.sock",
          supervision: { state: "initialization_required" },
        });
      }
      if (command === "sfwp_initialize_cell") {
        return Promise.reject({
          error_class: "cell_initialization_refused",
          error: "the selected root now contains history",
          no_side_effect: true,
          next_lawful_action: "Open its existing history",
        });
      }
      return Promise.reject(new Error(`unexpected bridge command: ${command}`));
    });
    renderPage();

    const initialize = await screen.findByRole("button", { name: "Initialize this cell" });
    initialize.click();

    expect(
      await screen.findByRole("alert"),
    ).toHaveTextContent(/the selected root now contains history/i);
    expect(screen.getByRole("alert")).toHaveTextContent(
      /next lawful action: open its existing history/i,
    );
    expect(screen.getByRole("button", { name: "Initialize this cell" })).not.toBeDisabled();
  });
});
