// White-labeled donor overlay: ContextIdentity.
// Donor source: `#title` block in `public/reference/gargantua.html`
// (typography, spacing, rule, fade). Only the semantics changed: product
// identity copy replaces the demo's GARGANTUA/Kerr-singularity wording.
// Visual treatment (Michroma/plex, letterspacing, amber rule) preserved.

import styles from "./CoreChrome.module.css";

export interface ContextIdentityProps {
  name?: string;
  tagline?: string;
}

export function ContextIdentity({
  name = "CORE",
  tagline = "Governed capability kernel · live orientation",
}: ContextIdentityProps) {
  return (
    <div className={styles.title} data-testid="context-identity">
      <h1>{name}</h1>
      <div className={styles.sub}>{tagline}</div>
      <div className={styles.rule} />
    </div>
  );
}
