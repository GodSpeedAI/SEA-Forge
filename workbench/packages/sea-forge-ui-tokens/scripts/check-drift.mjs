// ponytail: byte-diff is the whole check — this file is a copy-projection, not a transform.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL(".", import.meta.url));
const source = readFileSync(`${here}../../../../.agents/specs/frontend/colors_and_type.css`, "utf8");
const projected = readFileSync(`${here}../sea-forge.tokens.css`, "utf8");

if (source !== projected) {
  console.error(
    "sea-forge.tokens.css has drifted from .agents/specs/frontend/colors_and_type.css — re-copy the source file.",
  );
  process.exit(1);
}
console.log("sea-forge.tokens.css matches spec source.");
