import { defineTheme } from "@astryxdesign/core/theme";
import { neutralTheme } from "@astryxdesign/theme-neutral/built";

// Keeps theme.name === "neutral" so theme-neutral's `@scope([data-astryx-theme="neutral"])`
// component CSS still applies; only token *values* are projected onto SEA Forge canonical
// tokens (.agents/specs/frontend/colors_and_type.css) so pixels come from SEA Forge, not
// Astryx defaults. This is the "theme" tier of the swizzle ladder — compose, don't fork.
export const seaForgeTheme = defineTheme({
  name: "neutral",
  extends: neutralTheme,
  tokens: {
    "--color-background-body": "var(--surface-workspace)",
    "--color-background-surface": "var(--surface-panel)",
    "--color-background-card": "var(--surface-panel-elevated)",
    "--color-background-popover": "var(--surface-overlay)",
    "--color-background-muted": "var(--surface-muted)",
    "--color-text-primary": "var(--fg-primary)",
    "--color-text-secondary": "var(--fg-secondary)",
    "--color-text-disabled": "var(--fg-tertiary)",
    "--color-border": "var(--surface-muted)",
    "--color-accent": "var(--color-focus-primary)",
    "--color-on-accent": "var(--fg-inverse)",
    "--color-success": "var(--color-authority-allowed)",
    "--color-error": "var(--color-danger)",
    "--color-warning": "var(--color-attention)",
    "--font-family-body": "var(--font-sans)",
    "--font-family-heading": "var(--font-sans)",
    "--font-family-code": "var(--font-mono)",
  },
});
