import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

const navigateMock = vi.fn();
vi.mock("@tanstack/react-router", async () => {
  const actual =
    await vi.importActual<typeof import("@tanstack/react-router")>("@tanstack/react-router");
  return { ...actual, useNavigate: () => navigateMock };
});

import { CaseCreationWorkbench } from "./CaseCreationWorkbench";

function renderWorkbench() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <CaseCreationWorkbench />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  invokeMock.mockReset();
  navigateMock.mockReset();
  invokeMock.mockImplementation(async (cmd: string, args: any) => {
    if (cmd === "sfwp_query" && args.query.verb === "case_entry_options") {
      return {
        templates: [
          {
            template_ref: "demo@0.1.0",
            description: "Demo template",
            parameters: { greeting: { param_type: "string", required: true } },
          },
        ],
      };
    }
    if (cmd === "sfwp_query" && args.query.verb === "case_preflight") {
      return {
        ok: true,
        template_ref: args.query.template_ref,
        items: [{ plan_item_id: "item_01", name: "Item", item_kind: "SandboxedTask" }],
        precondition: { ref: "template:demo@0.1.0", expected_digest: "sha256:abc" },
      };
    }
    if (cmd === "sfwp_command" && args.command.verb === "case_commit") {
      return { case_id: "case_1", state: "active", exit_code: 0 };
    }
    throw new Error(`unexpected invoke: ${cmd} ${JSON.stringify(args)}`);
  });
});

afterEach(() => {
  cleanup();
});

describe("CaseCreationWorkbench", () => {
  it("draft -> preflight -> commit end to end, then navigates on success", async () => {
    renderWorkbench();

    const templateOption = await screen.findByRole("radio", { name: /demo@0\.1\.0/i });
    fireEvent.click(templateOption);

    const paramField = await screen.findByLabelText(/greeting/i);
    fireEvent.change(paramField, { target: { value: "hello" } });

    const preflightButton = screen.getByRole("button", { name: /run preflight/i });
    fireEvent.click(preflightButton);

    await screen.findByText(/preflight passed/i);

    const commitButton = screen.getByRole("button", { name: /commit case/i });
    await waitFor(() => expect(commitButton).not.toBeDisabled());
    fireEvent.click(commitButton);

    await waitFor(() => expect(navigateMock).toHaveBeenCalledWith({ to: "/cases" }));

    const commitCall = invokeMock.mock.calls.find(
      ([cmd, args]) => cmd === "sfwp_command" && args.command.verb === "case_commit",
    );
    expect(commitCall?.[1].command.params).toEqual({ greeting: "hello" });
    expect(commitCall?.[1].command.preconditions.records).toEqual([
      { ref: "template:demo@0.1.0", expected_digest: "sha256:abc" },
    ]);
  });
});
