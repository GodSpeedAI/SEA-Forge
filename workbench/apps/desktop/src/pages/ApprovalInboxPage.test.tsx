import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();
const identityMock = vi.fn();

const governance = {
  approval_source: {
    ledger_id: "case-case-1",
    entry_id: "led_approval",
    record_kind: "approval_request",
    record_id: "ap-1",
    digest: "sha256:approval",
    freshness: "current" as const,
    rebuild_standing: "verified" as const,
  },
  decision_source: {
    ledger_id: "case-case-1",
    entry_id: "led_decision",
    record_kind: "authority_decision",
    record_id: "dec-1",
    digest: "sha256:decision",
    freshness: "current" as const,
    rebuild_standing: "verified" as const,
  },
  reason: "External endpoint access requires independent review",
  reason_codes: ["external_boundary"],
  matched_rule: "review-external-endpoint",
  policy_refs: ["policy:active"],
  boundary_constraints: { network: ["review_required"] },
  requester: "operator_requester",
  operation_kind: "agent_probe",
  resource_ref: "endpoint:external",
  purpose_context: { plan_item_id: "task", channel: "cli" },
  evidence_refs: ["evidence_01"],
  required_next_steps: ["Approve or reject after reviewing the policy boundary"],
  side_effect_standing: "not_executed_pending_approval",
  eligible_actors: [],
  eligibility_standing: "resolved_per_connection",
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

vi.mock("../hooks/useIdentity", () => ({
  useIdentity: () => identityMock(),
  selectedActorId: () => window.sessionStorage.getItem("sea-forge.acting-identity") ?? undefined,
}));

import { ApprovalInboxPage } from "./ApprovalInboxPage";

function approval(overrides: Record<string, unknown> = {}) {
  return {
    approval_id: "ap-1",
    case_id: "case-1",
    run_id: "run-1",
    plan_item_id: "Permit external endpoint probe",
    decision_id: "dec-1",
    requested_at: "2026-07-26T09:00:00Z",
    expires_at: "2099-01-01T00:00:00Z",
    expired: false,
    criteria_ref: "criteria/settle.yaml",
    criteria_sha256: "sha256:abc123",
    governance,
    ...overrides,
  };
}

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <ApprovalInboxPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  invokeMock.mockClear();
  window.sessionStorage.clear();
  identityMock.mockReturnValue({
    identity: {
      available: [{ actor_id: "operator_a", roles: ["operator"] }],
      actor: { actorId: "operator_a", role: "operator" },
      configured: true,
    },
  });
});
afterEach(() => {
  window.sessionStorage.clear();
  cleanup();
});

describe("approval inbox", () => {
  /**
   * The reason this surface exists: `approve`/`reject` need a case id and an
   * approval id, and before `approval.list` nothing could produce them.
   */
  it("shows the identifiers a decision requires", async () => {
    invokeMock.mockResolvedValue({ approvals: [approval()] });
    renderPage();

    expect(await screen.findByText("Permit external endpoint probe")).toBeVisible();
    expect(screen.getByText("ap-1")).toBeVisible();
    expect(screen.getByText("case-1")).toBeVisible();
  });

  it("shows the criteria hash the decision is judged against", async () => {
    invokeMock.mockResolvedValue({ approvals: [approval()] });
    renderPage();

    expect(await screen.findByText("sha256:abc123")).toBeVisible();
  });

  it("shows the committed reason, purpose, source, and lawful next action for a decision", async () => {
    invokeMock.mockResolvedValue({ approvals: [approval()] });
    renderPage();

    expect(await screen.findByText(/External endpoint access requires independent review/)).toBeVisible();
    expect(screen.getByText(/task/)).toBeVisible();
    expect(screen.getByText("led_decision")).toBeVisible();
    expect(screen.getByText(/Approve or reject after reviewing the policy boundary/)).toBeVisible();
  });

  it("makes an unavailable approval source visible rather than synthesizing a reason", async () => {
    invokeMock.mockResolvedValue({ approvals: [approval({ governance: undefined })] });
    renderPage();

    expect(await screen.findByRole("alert")).toHaveTextContent(/no resolvable committed governance context/i);
  });

  it("distinguishes an empty queue from an unread one", async () => {
    invokeMock.mockResolvedValue({ approvals: [] });
    renderPage();

    expect(await screen.findByText(/empty queue, not an unread one/)).toBeVisible();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("reports an unreadable journal instead of showing it as empty", async () => {
    invokeMock.mockResolvedValue({ approvals: [], unreadable: "parse approval line: bad json" });
    renderPage();

    expect(await screen.findByRole("alert")).toHaveTextContent(/could not be parsed/);
    expect(screen.queryByText(/empty queue, not an unread one/)).not.toBeInTheDocument();
  });

  /** Expiry explains why work is parked; hiding the row hides the explanation. */
  it("flags an expired approval rather than dropping it from the list", async () => {
    invokeMock.mockResolvedValue({ approvals: [approval({ expired: true })] });
    renderPage();

    expect(await screen.findByText("Window closed")).toBeVisible();
    expect(screen.getByText("Permit external endpoint probe")).toBeVisible();
  });

  /**
   * The kernel can refuse a decision. Removing the row on click would tell the
   * approver the opposite of what happened.
   */
  it("keeps the row and reports the refusal when the kernel rejects a decision", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "sfwp_command") {
        return Promise.resolve({ error: "approval window has closed" });
      }
      return Promise.resolve({ approvals: [approval()] });
    });
    renderPage();

    fireEvent.click(await screen.findByRole("button", { name: "Approve" }));

    expect(await screen.findByText(/was not approved: approval window has closed/)).toBeVisible();
    // The approval is still listed — nothing was optimistically removed.
    expect(screen.getByText("Permit external endpoint probe")).toBeVisible();
  });

  it("sends the verdict the operator chose, never a default", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "sfwp_command") return Promise.resolve({ ok: true });
      return Promise.resolve({ approvals: [approval()] });
    });
    renderPage();

    fireEvent.click(await screen.findByRole("button", { name: "Reject" }));

    await screen.findByText(/Recorded as rejected/);
    const commandCall = invokeMock.mock.calls.find(([name]) => name === "sfwp_command");
    expect(commandCall?.[1]).toMatchObject({
      command: {
        verb: "approval_decide",
        case_id: "case-1",
        approval_id: "ap-1",
        decision: "reject",
      },
    });
  });

  it("forwards the explicit server-advertised actor selection to the host", async () => {
    window.sessionStorage.setItem("sea-forge.acting-identity", "operator_b");
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "sfwp_command") return Promise.resolve({ ok: true });
      return Promise.resolve({ approvals: [approval()] });
    });
    renderPage();

    fireEvent.click(await screen.findByRole("button", { name: "Approve" }));
    await screen.findByText(/Recorded as approved/);

    const commandCall = invokeMock.mock.calls.find(([name]) => name === "sfwp_command");
    expect(commandCall?.[1]).toMatchObject({ actAs: "operator_b" });
  });

  it("blocks approval decisions with identity_unresolved before a host call", async () => {
    identityMock.mockReturnValue({ identity: undefined });
    invokeMock.mockResolvedValue({ approvals: [approval()] });
    renderPage();

    expect(await screen.findByRole("button", { name: "Approve" })).toBeDisabled();
    expect(screen.getByRole("alert")).toHaveTextContent(/identity_unresolved/i);
    expect(screen.getByRole("link", { name: "Inspect identity bindings" })).toHaveAttribute(
      "href",
      "/admin",
    );
    expect(invokeMock).not.toHaveBeenCalledWith("sfwp_command", expect.anything());
  });
});
