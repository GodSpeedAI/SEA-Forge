// White-labeled donor overlay: RenderFailureSurface.
// Donor source: `#fatal` card in `public/reference/gargantua.html`
// (fullscreen black card, amber Michroma heading, dim body, cyan detail).
// Same failure taxonomy and messages as the donor's `bhFatal`; the demo's
// "SIGNAL LOST" default heading is replaced per-incident by the renderer's
// title. Rendered by CoreViewport on `onFatal`, never by product code.

import type { CoreFatalInfo } from "../CoreRenderer";
import styles from "./CoreChrome.module.css";

export function RenderFailureSurface({ info }: { info: CoreFatalInfo }) {
  return (
    <div className={styles.fatal} role="alert" data-testid="render-failure">
      <div className={styles.fatalCard}>
        <h2>{info.title}</h2>
        <p>{info.message}</p>
        {info.detail ? <code>{info.detail}</code> : null}
      </div>
    </div>
  );
}
