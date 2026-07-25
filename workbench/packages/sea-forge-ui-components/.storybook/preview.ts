import type { Preview } from "@storybook/react";
import "@sea-forge/ui-tokens/tokens.css";
import "@sea-forge/astryx-theme";

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    backgrounds: {
      default: "dark",
      values: [
        { name: "dark", value: "#0d1117" },
        { name: "light", value: "#ffffff" },
      ],
    },
  },
};

export default preview;
