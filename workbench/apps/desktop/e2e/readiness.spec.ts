import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import {
  installTauriMock,
  degradedReadinessView,
  readyReadinessView,
  unresolvedIdentity,
} from "./tauriMock";

function captureBrowserErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  page.on("pageerror", (error) => errors.push(error.message));
  return errors;
}

test.describe("Readiness primary-path journey", () => {
  test("spatial shell keeps CORE mounted with rail, composer, and chrome", async ({
    page,
  }) => {
    const browserErrors = captureBrowserErrors(page);
    await installTauriMock(page, degradedReadinessView());
    await page.setViewportSize({ width: 1600, height: 1000 });
    await page.goto("/readiness");

    // CoreViewport is persistent: canvas mounted once, never remounted by
    // surface changes. (Headless CI has no WebGL2, so the failure surface —
    // not a black screen — may render over it; either way the viewport owns
    // the lifetime.)
    await expect(page.getByTestId("core-viewport")).toBeAttached();
    await expect(page.getByTestId("core-canvas")).toBeAttached();

    // White-labeled donor chrome, no demo copy.
    await expect(page.getByTestId("context-identity")).toBeVisible();
    await expect(page.getByTestId("context-readout")).toBeVisible();
    await expect(page.getByTestId("interaction-hints")).toBeVisible();
    await expect(page.getByTestId("surface-rail")).toBeVisible();
    await expect(page.getByTestId("composer")).toBeVisible();
    await expect(page.getByTestId("connection-state-bar")).toBeVisible();

    // Nine spatial surfaces, CORE first — the legacy thirteen-link page nav
    // is gone and must not reappear.
    const railLinks = page.getByTestId("surface-rail").getByRole("link");
    await expect(railLinks).toHaveCount(9);
    await expect(railLinks.first()).toHaveText(/CORE/);

    // Evidence drawer still opens by default, as before the migration.
    await expect(page.getByTestId("evidence-drawer")).toBeVisible();

    // Composer offers only projected affordances for this context.
    await expect(page.getByTestId("composer")).toContainText(/CORE|Compose|Review|Return/);
    expect(browserErrors).toEqual([]);
  });

  test("Surface rail swaps governed surfaces without remounting CORE", async ({ page }) => {
    const browserErrors = captureBrowserErrors(page);
    await installTauriMock(page, degradedReadinessView());
    await page.setViewportSize({ width: 1600, height: 1000 });
    await page.goto("/readiness");

    const viewport = page.getByTestId("core-viewport");
    await expect(viewport).toBeAttached();

    const routes = [
      ["Beat", "/thoth", "Thoth workspace", "thoth-question-composer"],
      ["Case", "/cases", "Case horizon", "case-horizon-board"],
      ["Judge", "/inbox", "Inbox and approvals", "approval-decision-panel"],
      ["Time", "/operations", "Operations monitor", "execution-console"],
    ] as const;

    for (const [label, path, heading, region] of routes) {
      await page.getByTestId("surface-rail").getByRole("link", { name: label }).click();
      await expect(page).toHaveURL(new RegExp(`${path}$`));
      await expect(page.getByRole("heading", { level: 1, name: heading })).toBeVisible();
      await expect(page.locator(`[data-od-id="${region}"]`)).toBeVisible();
      await expect(page.getByTestId("evidence-drawer")).toBeVisible();
      // CORE persists across surface changes: same viewport element, still
      // attached — no WebGL context recreation on navigation.
      await expect(viewport).toBeAttached();
      const routeA11y = await new AxeBuilder({ page })
        .include("#main-content")
        .analyze();
      expect(routeA11y.violations).toEqual([]);
    }

    expect(browserErrors).toEqual([]);
  });

  test("launch → degraded readiness → inspect why → disabled case-creation shows reason", async ({
    page,
  }) => {
    const browserErrors = captureBrowserErrors(page);
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

    // The reference opens evidence by default. Close the overlay at this
    // viewport, then prove a source-backed row can reopen it.
    await page.getByRole("button", { name: /Close evidence drawer/i }).click();
    await expect(page.getByTestId("evidence-drawer")).toBeHidden();

    // Why / evidence: open the evidence citation drawer from a readiness row.
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
    expect(browserErrors).toEqual([]);
  });

  test("unresolved identity blocks case creation and routes to the repair surface", async ({ page }) => {
    await installTauriMock(page, degradedReadinessView(), unresolvedIdentity());
    await page.setViewportSize({ width: 1600, height: 1000 });
    await page.goto("/readiness");

    await expect(page.getByRole("button", { name: "Create case" })).toBeDisabled();
    await expect(page.getByTestId("disabled-reason")).toContainText("identity_unresolved");
    await page.getByRole("button", { name: "Inspect identity bindings" }).click();
    await expect(page).toHaveURL(/\/admin$/);
  });

  test("resolved identity can reach case creation while evidence is open", async ({ page }) => {
    await installTauriMock(page, readyReadinessView());
    await page.setViewportSize({ width: 1600, height: 1000 });
    await page.goto("/readiness");

    await expect(page.getByTestId("evidence-drawer")).toBeVisible();
    await expect(page.getByRole("button", { name: "Create case" })).toBeEnabled();
    await page.getByRole("button", { name: "Create case" }).click();
    await expect(page).toHaveURL(/\/cases\/new$/);
  });

  test("Inspect all capabilities focuses readiness evidence while the drawer is open", async ({ page }) => {
    const browserErrors = captureBrowserErrors(page);
    await installTauriMock(page, degradedReadinessView());
    await page.setViewportSize({ width: 1600, height: 1000 });
    await page.goto("/readiness");

    const drawer = page.getByTestId("evidence-drawer");
    await expect(drawer).toBeVisible();
    await page.getByRole("button", { name: "Inspect all capabilities" }).click();
    await drawer.getByRole("tab", { name: "Evidence" }).click();
    await expect(drawer).toContainText("readiness_capability_summary");
    await expect(browserErrors).toEqual([]);
  });
});
