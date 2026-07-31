import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { GlobalHeader } from "./GlobalHeader";

/**
 * The context bar is on screen for every surface, so a fabricated value here is
 * a claim repeated on every page. It used to default to `actorName="Operator"`,
 * `integrityStatus="verified"`, and `inboxCount=1` — and `AppShell` passed none
 * of them, so a cell that had resolved no actor, verified no integrity, and
 * read no inbox still rendered all three as facts.
 *
 * These tests exist to make that unrepresentable rather than merely fixed.
 */
describe("GlobalHeader governance context", () => {
  it("says nothing is resolved when nothing was passed", () => {
    render(<GlobalHeader />);

    expect(screen.getByLabelText("Active actor and role")).toHaveTextContent("Unresolved");
    expect(screen.getByLabelText("Active cell")).toHaveTextContent("Unresolved");
    expect(screen.getByLabelText("Integrity state")).toHaveTextContent("Integrity unknown");
  });

  it("never claims a verified integrity state it was not given", () => {
    render(<GlobalHeader />);
    expect(screen.getByLabelText("Integrity state")).not.toHaveTextContent("verified");
  });

  /**
   * An unread inbox is not an empty one. Rendering `0 approvals` before the
   * journal is read would tell an approver there is nothing waiting for them,
   * which is the one thing this chip must never get wrong.
   */
  it("distinguishes an unread inbox from an empty one", () => {
    const { rerender } = render(<GlobalHeader />);
    expect(screen.getByRole("button", { name: /Open approval inbox/ })).toHaveTextContent(
      "Unknown",
    );

    rerender(<GlobalHeader inboxCount={0} />);
    expect(screen.getByRole("button", { name: /Open approval inbox/ })).toHaveTextContent(
      "0 approvals",
    );
  });

  it("renders the resolved actor, role, and cell when the cell supplied them", () => {
    render(
      <GlobalHeader
        actorName="operator_local"
        roleName="operator"
        cellName="my-cell"
        integrityStatus="verified"
        inboxCount={2}
      />,
    );

    expect(screen.getByLabelText("Active actor and role")).toHaveTextContent(
      "operator_local · operator",
    );
    expect(screen.getByLabelText("Active cell")).toHaveTextContent("my-cell");
    expect(screen.getByRole("button", { name: /Open approval inbox/ })).toHaveTextContent(
      "2 approvals",
    );
  });

  /** Machine-checkable by anything auditing the DOM, not just by eye. */
  it("marks the actor chip as unresolved for styling and audit", () => {
    const { rerender } = render(<GlobalHeader />);
    expect(screen.getByLabelText("Active actor and role")).toHaveAttribute(
      "data-resolved",
      "false",
    );

    rerender(<GlobalHeader actorName="operator_local" roleName="operator" />);
    expect(screen.getByLabelText("Active actor and role")).toHaveAttribute(
      "data-resolved",
      "true",
    );
  });
});
