import { describe, expect, it } from "vitest";
import referenceStylesSource from "../../../../.agents/specs/frontend/ui_kits/app/styles.css?raw";
import referenceMarkupSource from "../../../../.agents/specs/frontend/ui_kits/app/index.html?raw";
import productionStylesSource from "./mockup.css?raw";
import appShellSource from "./shell/AppShell.tsx?raw";
import headerSource from "./shell/GlobalHeader.tsx?raw";
import sidebarSource from "./shell/Sidebar.tsx?raw";
import readinessSource from "./pages/ReadinessPage.tsx?raw";

function normalize(source: string): string {
  return source.replace(/\r\n/g, "\n").trim();
}

describe("Readiness mockup fidelity", () => {
  it("keeps the production workbench stylesheet aligned with the reference kit", () => {
    const reference = normalize(referenceStylesSource).replace(
      /^@import url\("\.\.\/\.\.\/colors_and_type\.css"\);\n+/,
      "",
    );
    const production = normalize(productionStylesSource).replace(
      /^\/\* Reference projection: canonical tokens are imported by main\.tsx\. \*\/\n+/,
      "",
    );

    expect(production).toBe(reference);
  });

  it("keeps the reference shell regions represented in React source", () => {
    const reference = normalize(referenceMarkupSource);
    const production = [
      appShellSource,
      headerSource,
      sidebarSource,
      readinessSource,
    ].join("\n");

    for (const region of [
      "application-shell",
      "global-context-bar",
      "primary-navigation",
      "readiness-workspace",
      "readiness-governed-focus",
      "intended-work-selector",
      "critical-foundations",
      "operational-capabilities",
      "current-affordance-rail",
      "connection-state-bar",
    ]) {
      expect(reference).toContain(`data-od-id="${region}"`);
      expect(production).toContain(`data-od-id="${region}"`);
    }
  });
});
