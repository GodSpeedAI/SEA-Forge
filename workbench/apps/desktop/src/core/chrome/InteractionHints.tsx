// White-labeled donor overlay: InteractionHints.
// Donor source: `#hint` block in `public/reference/gargantua.html`
// (key-chip styling, bottom-left placement, fade). Wording adapted to the
// spatial grammar (focus/zoom/compose); the drag/scroll discovery stays.

import styles from "./CoreChrome.module.css";

export function InteractionHints() {
  return (
    <div className={styles.hint} data-testid="interaction-hints">
      <span className={styles.key}>drag</span> orbit &nbsp; <span className={styles.key}>scroll</span>{" "}
      zoom &nbsp; <span className={styles.key}>H</span> hide ui
    </div>
  );
}
