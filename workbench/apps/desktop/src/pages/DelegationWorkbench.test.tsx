import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
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

import { DelegationWorkbench } from "./DelegationWorkbench";

const ENDPOINT_ROW = {
  asset_id: "agent_endpoint:local",
  kind: "agent_endpoint",
  name: "local",
  standing: "demonstrated",
  evidence_refs: ["run:run-1"],
};

function contract(overrides: Record<string, unknown> = {}) {
  return {
    provider_kind: "openai_compatible",
    endpoint_digest: `sha256:${"e".repeat(64)}`,
    model: { value: "m1", source: "endpoint_default" },
    transcript_retention: { value: "summarized", source: "cell_default" },
    instruction_sha256: `sha256:${"a".repeat(64)}`,
    instruction_bytes: 5,
    max_request_bytes: 4096,
    max_response_bytes: 2048,
    timeout_secs: 45,
    max_turns: 4,
    required_authority: "agent_task",
    contract_digest: `sha256:${"c".repeat(64)}`,
    ...overrides,
  };
}

function preview(overrides: Record<string, unknown> = {}) {
  return {
    endpoint: "local",
    eligible: true,
    standing: "demonstrated",
    evidence_refs: ["run:run-1"],
    contract: contract(),
    ...overrides,
  };
}

/**
 * Route by verb: the page issues `asset_list` for the endpoint picker and
 * `delegation_preview` for the contract, and the tests care which is which.
 */
function respondWith(previewBody: unknown, endpointRows: unknown[] = [ENDPOINT_ROW]) {
  invokeMock.mockImplementation((_command: string, args: { query: { verb: string } }) => {
    if (args.query.verb === "asset_list") {
      return Promise.resolve({ assets: endpointRows, unreadable: [] });
    }
    // The page also renders the delegation roster; it is not what these tests
    // are about, so it gets an honest empty answer rather than the preview body.
    if (args.query.verb === "delegation_list") {
      return Promise.resolve({ delegations: [], unreadable: [] });
    }
    return Promise.resolve(previewBody);
  });
}

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  return render(
    <QueryClientProvider client={client}>
      <DelegationWorkbench />
    </QueryClientProvider>,
  );
}

/** Fill the form with a valid request and ask for the contract. */
async function inspect() {
  await screen.findByRole("option", { name: /local/ });
  fireEvent.change(screen.getByLabelText("Endpoint"), { target: { value: "local" } });
  fireEvent.change(screen.getByLabelText("Instruction"), { target: { value: "hello" } });
  fireEvent.click(screen.getByRole("button", { name: "Inspect job contract" }));
}

beforeEach(() => {
  invokeMock.mockReset();
});

afterEach(cleanup);

describe("DelegationWorkbench", () => {
  it("asks for delegation.preview with the flat params the host bridge accepts", async () => {
    respondWith(preview());
    renderPage();
    await inspect();

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("sfwp_query", {
        query: {
          verb: "delegation_preview",
          endpoint: "local",
          instruction: "hello",
          max_turns: 4,
        },
      }),
    );
  });

  /**
   * An omitted optional must be absent, not `null` — the bridge's
   * `Option<String>`/`Option<u64>` reject an explicit null, and "no token cap"
   * would come back as a transport error instead of as the fact it is.
   */
  it("omits model and token budget rather than sending nulls", async () => {
    respondWith(preview());
    renderPage();
    await inspect();

    // Find the preview call rather than counting calls: the page issues other
    // queries of its own, and a count would break every time one is added.
    await waitFor(() =>
      expect(
        invokeMock.mock.calls.some(
          ([, a]) => (a as { query: { verb: string } }).query.verb === "delegation_preview",
        ),
      ).toBe(true),
    );
    const [, args] = invokeMock.mock.calls.find(
      ([, a]) => (a as { query: { verb: string } }).query.verb === "delegation_preview",
    )!;
    const query = (args as { query: Record<string, unknown> }).query;
    expect("model" in query).toBe(false);
    expect("token_budget" in query).toBe(false);
  });

  /**
   * The invariant the surface exists to protect. `eligible` says only that no
   * precondition is unmet; the authority decision has not been made, and a page
   * that implied otherwise would be showing a grant with no record behind it.
   */
  it("names the required authority and states that no decision has been made", async () => {
    respondWith(preview());
    renderPage();
    await inspect();

    const panel = await screen.findByRole("region", { name: "Job contract" });
    expect(within(panel).getByText("agent_task")).toBeVisible();
    expect(within(panel).getByText(/No decision has been made/)).toBeVisible();
    expect(within(panel).getByText(/not the same as authorized/)).toBeVisible();
  });

  /** There is no command on this surface; the delegation itself is not shipped here. */
  it("offers no button that would commit the delegation", async () => {
    respondWith(preview());
    renderPage();
    await inspect();
    await screen.findByRole("region", { name: "Job contract" });

    for (const label of [/^Delegate$/, /Run delegation/i, /^Commit$/]) {
      expect(screen.queryByRole("button", { name: label })).toBeNull();
    }
  });

  /**
   * Provenance is the reason a resolved value is not a bare string: `m1` chosen
   * by the operator and `m1` supplied by the endpoint are different facts.
   */
  it("qualifies each resolved value with who decided it", async () => {
    respondWith(
      preview({
        contract: contract({
          model: { value: "m1", source: "requested" },
          transcript_retention: { value: "full", source: "endpoint_default" },
        }),
      }),
    );
    renderPage();
    await inspect();

    const panel = await screen.findByRole("region", { name: "Job contract" });
    expect(within(panel).getByText(/you asked for this/)).toBeVisible();
    expect(within(panel).getByText(/from the endpoint descriptor/)).toBeVisible();
  });

  /** Absent is not zero: a zero budget would mean "spend nothing". */
  it("reports an absent token budget as no cap rather than as zero", async () => {
    respondWith(preview());
    renderPage();
    await inspect();

    const panel = await screen.findByRole("region", { name: "Job contract" });
    expect(within(panel).getByText(/no token cap/)).toBeVisible();
    expect(within(panel).queryByText("0")).toBeNull();
  });

  it("lists every blocking reason and marks the delegation blocked", async () => {
    respondWith(
      preview({
        eligible: false,
        standing: "probed",
        blocking_reasons: [
          "last probe settled rejected: endpoint returned 503",
          "agent_task max_turns must be >= 1",
        ],
      }),
    );
    renderPage();
    await inspect();

    const panel = await screen.findByRole("region", { name: "Job contract" });
    expect(within(panel).getByText(/endpoint returned 503/)).toBeVisible();
    expect(within(panel).getByText(/max_turns must be >= 1/)).toBeVisible();
    const variants = [...panel.querySelectorAll("[data-variant]")].map((el) =>
      el.getAttribute("data-variant"),
    );
    expect(variants).toContain("blocked");
  });

  /**
   * A blocked endpoint must stay selectable. Filtering it out would report an
   * endpoint that exists and is refused as one that is absent — and its reason
   * for being refused is what the operator came to read.
   */
  it("keeps a blocked endpoint in the picker instead of hiding it", async () => {
    respondWith(preview(), [
      {
        ...ENDPOINT_ROW,
        standing: "probed",
        blocking_reason: "last probe settled rejected: endpoint returned 503",
      },
    ]);
    renderPage();

    const option = await screen.findByRole("option", { name: /local/ });
    expect(option).toBeVisible();
    expect(option.textContent).toContain("blocked");
  });

  /**
   * A contract describes one exact request. Editing after reading must not leave
   * the old contract presenting itself as a description of the new form.
   */
  it("marks the contract as describing the earlier request once the form changes", async () => {
    respondWith(preview());
    renderPage();
    await inspect();
    await screen.findByRole("region", { name: "Job contract" });
    expect(screen.queryByRole("status")).toBeNull();

    fireEvent.change(screen.getByLabelText("Instruction"), {
      target: { value: "hello again" },
    });
    expect(await screen.findByRole("status")).toHaveTextContent(
      /describes the earlier request/,
    );
  });

  it("reads nothing until an endpoint is chosen and the contract is asked for", async () => {
    respondWith(preview());
    renderPage();
    await screen.findByRole("option", { name: /local/ });

    expect(
      invokeMock.mock.calls.some(
        ([, a]) => (a as { query: { verb: string } }).query.verb === "delegation_preview",
      ),
    ).toBe(false);
    expect(screen.getByRole("button", { name: "Inspect job contract" })).toBeDisabled();
  });

  it("reports a cell with no endpoints as a fact rather than an empty picker", async () => {
    respondWith(preview(), []);
    renderPage();
    expect(
      await screen.findByText(/No agent endpoint is configured in this cell/),
    ).toBeVisible();
  });

  /** A governed error body is a denial, not a malformed response. */
  it("surfaces a governed error envelope as an alert rather than a contract failure", async () => {
    respondWith({
      error: "disclosure not permitted for delegation preview",
      error_class: "disclosure_denied",
    });
    renderPage();
    await inspect();

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("disclosure not permitted for delegation preview");
    expect(alert).not.toHaveTextContent(/contract validation/);
  });

  /**
   * An endpoint that is not configured has no standing at all. Showing
   * `declared` would put it on a ladder it was never on.
   */
  it("reports a missing endpoint as having no standing rather than the lowest one", async () => {
    respondWith({
      endpoint: "nowhere",
      eligible: false,
      blocking_reasons: ["agent endpoint 'nowhere' is not configured in this cell"],
    });
    renderPage();
    await inspect();

    const panel = await screen.findByRole("region", { name: "Job contract" });
    expect(within(panel).getByText(/has no standing/)).toBeVisible();
    expect(within(panel).getByText(/No contract can be built/)).toBeVisible();
  });
});
