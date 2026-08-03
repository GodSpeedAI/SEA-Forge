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
  test("matches the reference desktop shell geometry and keeps evidence docked", async ({
    page,
  }) => {
    const browserErrors = captureBrowserErrors(page);
    await installTauriMock(page, degradedReadinessView());
    await page.setViewportSize({ width: 1600, height: 1000 });
    await page.goto("/readiness");

    const shell = page.locator('[data-od-id="application-shell"]');
    const header = page.locator('[data-od-id="global-context-bar"]');
    const navigation = page.locator('[data-od-id="primary-navigation"]');
    const workspace = page.locator('[data-od-id="readiness-workspace"]');
    const drawer = page.getByTestId("evidence-drawer");
    const footer = page.locator('[data-od-id="connection-state-bar"]');

    await expect(shell).toBeVisible();
    await expect(drawer).toBeVisible();
    await expect(page.getByRole("tab", { name: "Why" })).toBeVisible();

    const [headerBox, navigationBox, workspaceBox, drawerBox, footerBox] =
      await Promise.all([
        header.boundingBox(),
        navigation.boundingBox(),
        workspace.boundingBox(),
        drawer.boundingBox(),
        footer.boundingBox(),
      ]);

    expect(headerBox?.height).toBe(56);
    expect(navigationBox?.width).toBe(236);
    expect(workspaceBox?.x).toBe(236);
    expect(drawerBox?.width).toBe(380);
    expect(drawerBox?.x).toBe(1220);
    expect(footerBox?.height).toBe(28);

    await page.setViewportSize({ width: 1280, height: 800 });

    const overlayBox = await drawer.boundingBox();
    const responsiveNavigationBox = await navigation.boundingBox();
    const responsiveWorkspaceBox = await workspace.boundingBox();
    const overlayStyles = await drawer.evaluate((element) => {
      const styles = getComputedStyle(element);
      return {
        position: styles.position,
        transitionDuration: styles.transitionDuration,
        transitionTimingFunction: styles.transitionTimingFunction,
      };
    });
    const backdropStyles = await page
      .getByTestId("evidence-drawer-backdrop")
      .evaluate((element) => {
        const styles = getComputedStyle(element);
        return {
          backgroundColor: styles.backgroundColor,
          width: styles.width,
        };
      });

    expect(overlayBox).toMatchObject({
      x: 880,
      y: 56,
      width: 400,
      height: 744,
    });
    expect(responsiveNavigationBox?.width).toBe(224);
    expect(responsiveWorkspaceBox?.x).toBe(224);
    expect(overlayStyles).toEqual({
      position: "static",
      transitionDuration: "0.18s",
      transitionTimingFunction: "cubic-bezier(0.23, 1, 0.32, 1)",
    });
    expect(backdropStyles).toEqual({
      backgroundColor: "rgba(0, 0, 0, 0)",
      width: "400px",
    });

    await page.getByRole("button", { name: "Close evidence drawer" }).click();
    await expect(drawer).toBeHidden();
    const closedTransform = await drawer.evaluate(
      (element) => getComputedStyle(element).transform,
    );
    expect(closedTransform).not.toBe("none");

    await page
      .getByRole("button", { name: "Toggle active work evidence" })
      .click();
    await expect(drawer).toBeVisible();
    await expect
      .poll(async () => {
        const box = await drawer.boundingBox();
        return box
          ? {
              x: Math.round(box.x),
              y: Math.round(box.y),
              width: Math.round(box.width),
              height: Math.round(box.height),
            }
          : null;
      })
      .toEqual({
        x: 880,
        y: 56,
        width: 400,
        height: 744,
      });
    expect(browserErrors).toEqual([]);
  });

  test("Operate navigation swaps governed views and route context", async ({ page }) => {
    const browserErrors = captureBrowserErrors(page);
    await installTauriMock(page, degradedReadinessView());
    await page.setViewportSize({ width: 1600, height: 1000 });
    await page.goto("/readiness");

    const routes = [
      ["Thoth", "/thoth", "Thoth workspace", "thoth-question-composer", "Thoth"],
      ["Assets", "/assets", "Asset catalog", "asset-catalog-table", "Assets"],
      ["Domain Models", "/models", "Domain models", "domain-model-workbench", "Domain"],
      ["Cases", "/cases", "Case horizon", "case-horizon-board", "Cases"],
      ["Inbox", "/inbox", "Inbox and approvals", "approval-decision-panel", "Inbox"],
      ["Operations", "/operations", "Operations monitor", "execution-console", "Operations"],
    ] as const;

    for (const [label, path, heading, region, journeyStep] of routes) {
      await page.getByRole("link", { name: new RegExp(label, "i") }).click();
      await expect(page).toHaveURL(new RegExp(`${path}$`));
      await expect(page.getByRole("heading", { level: 1, name: heading })).toBeVisible();
      await expect(page.locator(`[data-od-id="${region}"]`)).toBeVisible();
      await expect(
        page.locator('[aria-label="Governed journey"] strong'),
      ).toHaveText(journeyStep);
      await expect(page.getByTestId("evidence-drawer")).toBeVisible();
      const routeA11y = await new AxeBuilder({ page })
        .include("#main-content")
        .analyze();
      expect(routeA11y.violations).toEqual([]);
    }

    await page.getByRole("link", { name: /^Assets$/i }).click();
    const workspaceFits = await page
      .locator('[data-od-id="readiness-workspace"]')
      .evaluate((element) => element.scrollWidth <= element.clientWidth);
    const primaryAction = page.getByRole("button", {
      name: "Inspect selected asset",
    });
    const actionFits = await primaryAction.evaluate(
      (element) => element.scrollWidth <= element.clientWidth,
    );
    const assetGrid = await page.locator(".asset-row").first().evaluate((element) => {
      const styles = getComputedStyle(element);
      return {
        display: styles.display,
        columns: styles.gridTemplateColumns,
      };
    });
    expect(workspaceFits).toBe(true);
    expect(actionFits).toBe(true);
    expect(assetGrid.display).toBe("grid");
    expect(assetGrid.columns.split(" ").length).toBe(4);
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
