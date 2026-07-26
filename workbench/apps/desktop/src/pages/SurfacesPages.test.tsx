import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  AssetsPage,
  CasesPage,
  InboxPage,
  ModelsPage,
  OperationsPage,
  ThothPage,
} from "./SurfacesPages";

describe.each([
  {
    Page: ThothPage,
    heading: "Thoth workspace",
    region: "thoth-question-composer",
  },
  {
    Page: AssetsPage,
    heading: "Asset catalog",
    region: "asset-catalog-table",
  },
  {
    Page: ModelsPage,
    heading: "Domain models",
    region: "domain-model-workbench",
  },
  {
    Page: CasesPage,
    heading: "Case horizon",
    region: "case-horizon-board",
  },
  {
    Page: InboxPage,
    heading: "Inbox and approvals",
    region: "approval-decision-panel",
  },
  {
    Page: OperationsPage,
    heading: "Operations monitor",
    region: "execution-console",
  },
])("$heading mockup projection", ({ Page, heading, region }) => {
  it("renders the route-specific governed view instead of a generic placeholder", () => {
    const { container } = render(<Page />);

    expect(screen.getByRole("heading", { level: 1, name: heading })).toBeVisible();
    expect(container.querySelector(`[data-od-id="${region}"]`)).toBeInTheDocument();
  });
});
