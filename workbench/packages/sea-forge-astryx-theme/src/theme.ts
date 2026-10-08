import { neutralTheme } from "@astryxdesign/theme-neutral/built";

// Astryx injects styles for themes created with `defineTheme` at runtime. That
// path is incompatible with the desktop application's strict Tauri CSP. Keep
// the generated neutral projection (which has `__built: true`) and project SEA
// Forge's canonical token values through the static stylesheet beside this file.
// The theme name remains "neutral", so Astryx's generated component CSS applies.
export const seaForgeTheme = neutralTheme;
