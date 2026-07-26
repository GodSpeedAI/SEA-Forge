import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import axe from "axe-core";

import {
  GovernedStatusPill,
  DualStateIndicator,
  SourceFreshnessBadge,
  IntegrityIndicator,
  ProtectedActionButton,
  WhyStatePanel,
  EvidenceDrawer,
  AuthorityBoundaryPanel,
  AvailabilityLadder,
} from "./index";

describe("GovernedStatusPill", () => {
  it("renders known variants correctly", () => {
    render(<GovernedStatusPill variant="ready" />);
    const pill = screen.getByTestId("governed-status-pill");
    expect(pill).toHaveTextContent("ready");
    expect(pill).toHaveAttribute("data-variant", "ready");
  });

  it("CRITICAL INVARIANT: fallback renders 'unknown' for invalid/unknown variant", () => {
    // @ts-expect-error testing invalid input string
    render(<GovernedStatusPill variant="magic_success_state" />);
    const pill = screen.getByTestId("governed-status-pill");
    expect(pill).toHaveTextContent("unknown");
    expect(pill).toHaveAttribute("data-variant", "unknown");
    expect(pill.className).toContain("status-pill--unknown");
  });

  it("CRITICAL INVARIANT: fallback renders 'unknown' when variant is null or undefined", () => {
    render(<GovernedStatusPill variant={null} />);
    const pill = screen.getByTestId("governed-status-pill");
    expect(pill).toHaveTextContent("unknown");
    expect(pill).toHaveAttribute("data-variant", "unknown");
  });
});

describe("DualStateIndicator", () => {
  it("displays execution and settlement state side-by-side", () => {
    render(<DualStateIndicator executionState="running" settlementState="pending" />);
    const indicator = screen.getByTestId("dual-state-indicator");
    expect(indicator).toHaveTextContent("Execution State");
    expect(indicator).toHaveTextContent("Settlement State");
    expect(indicator).toHaveTextContent("running");
    expect(indicator).toHaveTextContent("pending");
  });
});

describe("SourceFreshnessBadge", () => {
  it("renders live freshness badge", () => {
    render(<SourceFreshnessBadge mode="live" sourceName="Proof Engine" lastVerifiedAt="10:45:00" />);
    const badge = screen.getByTestId("source-freshness-badge");
    expect(badge).toHaveTextContent("Source current · Proof Engine");
    expect(badge).toHaveTextContent("(10:45:00)");
  });

  it("triggers onClick callback when clickable", () => {
    const handleClick = vi.fn();
    render(<SourceFreshnessBadge mode="stale" onClick={handleClick} />);
    const badge = screen.getByTestId("source-freshness-badge");
    fireEvent.click(badge);
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});

describe("IntegrityIndicator", () => {
  it("renders verified status correctly", () => {
    render(<IntegrityIndicator status="verified" />);
    const indicator = screen.getByTestId("integrity-indicator");
    expect(indicator).toHaveAttribute("data-status", "verified");
    expect(indicator).toHaveTextContent("Integrity verified");
  });
});

describe("ProtectedActionButton", () => {
  it("handles normal execution when allowed", async () => {
    const handleClick = vi.fn();
    render(<ProtectedActionButton label="Commit Case" onClick={handleClick} isAllowed />);
    const button = screen.getByRole("button", { name: "Commit Case" });
    await act(async () => {
      fireEvent.click(button);
    });
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it("displays disabled reason when disallowed or lacking standing", () => {
    render(
      <ProtectedActionButton
        label="Commit Case"
        onClick={() => {}}
        isAllowed={false}
        disabledReason="Requires higher authority level"
      />
    );
    expect(screen.getByTestId("disabled-reason")).toHaveTextContent("Requires higher authority level");
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("prompts for confirmation when requiresConfirmation is set", async () => {
    const handleClick = vi.fn();
    render(
      <ProtectedActionButton
        label="Destructive Action"
        onClick={handleClick}
        requiresConfirmation
        confirmationMessage="Proceed with purge?"
      />
    );
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Destructive Action" }));
    });
    expect(screen.getByText("Proceed with purge?")).toBeInTheDocument();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Confirm" }));
    });
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});

describe("WhyStatePanel", () => {
  it("renders panel with conditions and evidence refs", () => {
    render(
      <WhyStatePanel
        statusVariant="degraded"
        summary="Endpoint probe timed out"
        conditions={[
          { id: "c1", name: "SFWP Transport", status: "pass", explanation: "Socket listening" },
          { id: "c2", name: "Agent Probe", status: "fail", explanation: "Peer unreachable" },
        ]}
        evidenceRef="ev_ulid_12345"
      />
    );
    expect(screen.getByTestId("why-state-panel")).toHaveTextContent("Why this state");
    expect(screen.getByText("Endpoint probe timed out")).toBeInTheDocument();
    expect(screen.getByText(/ev_ulid_12345/)).toBeInTheDocument();
  });
});

describe("EvidenceDrawer", () => {
  it("renders drawer when open", () => {
    render(
      <EvidenceDrawer
        isOpen
        onClose={() => {}}
        evidence={{
          id: "ev_01",
          kind: "ledger_proof",
          ledgerUlid: "01HQXYZ",
          digest: "sha256:abc",
          disclosureStatus: "permitted",
        }}
      />
    );
    expect(screen.getByTestId("evidence-drawer")).toBeInTheDocument();
    expect(screen.getByText("ev_01")).toBeInTheDocument();
  });

  it("handles restricted disclosure status cleanly", () => {
    render(
      <EvidenceDrawer
        isOpen
        onClose={() => {}}
        evidence={{
          id: "ev_02",
          kind: "secret_transcript",
          disclosureStatus: "restricted",
        }}
      />
    );
    expect(screen.getByText(/Disclosure Restricted/)).toBeInTheDocument();
  });

  it("keeps the panel mounted while closed so the exit transition can complete", () => {
    render(
      <EvidenceDrawer
        isOpen={false}
        onClose={() => {}}
        evidence={{
          id: "ev_03",
          kind: "readiness_evaluation_summary",
          disclosureStatus: "permitted",
        }}
      />
    );

    expect(screen.getByTestId("evidence-drawer")).toBeInTheDocument();
    expect(screen.getByTestId("evidence-drawer-backdrop")).toHaveAttribute(
      "aria-hidden",
      "true",
    );
  });
});

describe("AuthorityBoundaryPanel", () => {
  it("renders authority fields and boundary chips", () => {
    render(<AuthorityBoundaryPanel actorName="Alice" roleName="Governor" />);
    expect(screen.getByTestId("authority-boundary-panel")).toHaveTextContent("Alice");
    expect(screen.getByTestId("authority-boundary-panel")).toHaveTextContent("Governor");
  });
});

describe("AvailabilityLadder", () => {
  it("renders current level correctly", () => {
    render(<AvailabilityLadder currentLevel={2} />);
    expect(screen.getByTestId("availability-ladder")).toHaveTextContent("L2: Local SFWP Transport (Current State)");
  });
});

describe("Accessibility (axe-core)", () => {
  it("passes basic accessibility audit for components", async () => {
    const { container } = render(
      <div>
        <GovernedStatusPill variant="ready" />
        <DualStateIndicator executionState="running" settlementState="committed" />
        <SourceFreshnessBadge mode="live" />
        <IntegrityIndicator status="verified" />
        <WhyStatePanel statusVariant="ready" summary="All checks green" />
      </div>
    );
    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
  });
});
