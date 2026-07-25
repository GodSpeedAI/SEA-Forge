import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { installTauriMock, degradedReadinessView } from "./tauriMock";

test.describe("Readiness primary-path journey", () => {
  test("launch → degraded readiness → inspect why → disabled case-creation shows reason", async ({
    page,
  }) => {
    await installTauriMock(page, degradedReadinessView());
    await page.goto("/readiness");

    // The readiness workspace renders (route + guard G1 passed).
    const workspace = page.getByTestId("readiness-workspace");
    await expect(workspace).toBeVisible();

    // Degraded readiness: the named blocking/limitation reason is visible, not a
    // generic error — proving the view is sourced from the projection.
    await expect(
      page.getByText(/Endpoint verification has not been recorded/i).first(),
    ).toBeVisible();

    // Why / evidence: open the evidence citation drawer from a WhyStatePanel.
    await page
      .getByRole("button", { name: /Inspect evidence/i })
      .first()
      .click();
    await expect(page.getByTestId("evidence-drawer")).toBeVisible();
    // Close the drawer (Escape) so it does not overlap the axe scan.
    await page.keyboard.press("Escape");
    await expect(page.getByTestId("evidence-drawer")).toBeHidden();

    // Disabled case-creation exposes its reason (never implies it would work).
    const createButton = page.getByRole("button", { name: /create case/i });
    await expect(createButton).toBeDisabled();
    await expect(page.getByTestId("disabled-reason")).toBeVisible();

    // a11y: zero axe violations on the readiness surface.
    const results = await new AxeBuilder({ page })
      .include('[data-testid="readiness-workspace"]')
      .analyze();
    expect(results.violations).toEqual([]);
  });
});
