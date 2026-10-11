import { describe, expect, it } from "vitest";

import { seaForgeTheme } from "@sea-forge/astryx-theme";

describe("seaForgeTheme", () => {
  it("uses a prebuilt Astryx projection for Tauri CSP compatibility", () => {
    expect(seaForgeTheme.__built).toBe(true);
  });
});
