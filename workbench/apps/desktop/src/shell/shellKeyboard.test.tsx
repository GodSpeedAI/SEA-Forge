import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import axe from "axe-core";
import { evaluateGuard } from "../guards/guards";
import { GovernedDenialSurface } from "../guards/GovernedDenialSurface";
import { Sidebar } from "./Sidebar";
import { GlobalHeader } from "./GlobalHeader";

// Mock TanStack Router Link and useLocation for testing Sidebar
vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children, className, ...props }: any) => (
    <a href={to} className={className} {...props}>
      {children}
    </a>
  ),
  useLocation: () => ({ pathname: "/readiness" }),
}));

describe("Shell & Keyboard Navigation", () => {
  it("renders Sidebar with all 13 top-level surfaces", () => {
    render(<Sidebar />);
    const nav = screen.getByTestId("primary-navigation");
    expect(nav).toBeInTheDocument();

    const surfaces = [
      "Readiness",
      "Thoth",
      "Assets",
      "Domain Models",
      "Cases",
      "Inbox",
      "Operations",
      "Evidence",
      "Memory",
      "Capabilities",
      "Artifacts",
      "Federation",
      "Administration",
    ];

    for (const surface of surfaces) {
      expect(screen.getByText(surface)).toBeInTheDocument();
    }
  });

  it("renders GlobalHeader context chips and search button", () => {
    render(<GlobalHeader actorName="Operator" roleName="Sponsor" />);
    expect(screen.getByTestId("global-context-bar")).toBeInTheDocument();
    expect(screen.getByText("Operator · Sponsor")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Open command search" })).toBeInTheDocument();
  });

  it("evaluates Route Guards G1 to G9 correctly", () => {
    const g1Pass = evaluateGuard("G1", { cellId: "cell_1" });
    expect(g1Pass.verdict).toBe("passed");
    expect(g1Pass.passed).toBe(true);

    const g1Fail = evaluateGuard("G1", { cellId: "" });
    expect(g1Fail.verdict).toBe("failed");
    expect(g1Fail.reason).toContain("Active cell context");

    const g9Fail = evaluateGuard("G9", { readinessState: "blocked" });
    expect(g9Fail.verdict).toBe("failed");
    expect(g9Fail.passed).toBe(false);
  });

  /**
   * The distinction the fabricated `mockGuardContext` erased. A guard nothing
   * answered must not read as a pass — and must not read as a failure either,
   * because "we did not ask" and "the cell said no" call for different operator
   * responses (epic invariant 6).
   */
  it("reports a guard with no source as indeterminate, never as passed", () => {
    for (const guard of ["G1", "G4", "G5", "G6", "G7", "G8", "G9", "G10"] as const) {
      const result = evaluateGuard(guard, {});
      expect(result.verdict, `${guard} with no evidence`).toBe("indeterminate");
      expect(result.passed, `${guard} must not claim a pass`).toBe(false);
      expect(result.reason, `${guard} must say what is missing`).toBeTruthy();
    }
  });

  /** Sponsorship governs automated actors; a human operator is out of scope. */
  it("reports sponsorship as not applicable for a self-sponsoring role", () => {
    const operator = evaluateGuard("G3", { actorRole: "operator" });
    expect(operator.verdict).toBe("not_applicable");
    expect(operator.passed).toBe(false);

    const agent = evaluateGuard("G3", { actorRole: "R-AA" });
    expect(agent.verdict).toBe("indeterminate");
  });

  it("renders GovernedDenialSurface when a guard fails (no blank screen)", () => {
    const failedGuard = evaluateGuard("G9", { readinessState: "blocked" });
    render(<GovernedDenialSurface guardResult={failedGuard} />);

    const denial = screen.getByTestId("governed-denial-surface");
    expect(denial).toBeInTheDocument();
    expect(screen.getByText(/Governed Access Denial — G9 Readiness/)).toBeInTheDocument();
    expect(screen.getByText("Open readiness console")).toBeInTheDocument();
  });

  it("passes accessibility check for shell header and sidebar", async () => {
    const { container } = render(
      <div>
        <GlobalHeader />
        <Sidebar />
      </div>
    );
    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
  });
});
