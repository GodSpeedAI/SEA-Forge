// White-labeled donor overlay: ContextReadout.
// Donor source: `#readout` block in `public/reference/gargantua.html`
// (position, type scale, tabular numerals, amber emphasis). The demo's
// Schwarzschild/ISCO/plasma/dilation semantics are replaced by orientation
// truth: focus state, surface, settlement posture. No invented domain data:
// every row renders a passed value or an explicit em-dash.

import styles from "./CoreChrome.module.css";

export interface ContextReadoutProps {
  focus?: string;
  surface?: string;
  settlement?: string;
  activity?: string;
}

function Row({ label, value, accent }: { label: string; value?: string; accent?: boolean }) {
  return (
    <div>
      {label} <b className={accent ? styles.amber : undefined}>{value ?? "—"}</b>
    </div>
  );
}

export function ContextReadout({ focus, surface, settlement, activity }: ContextReadoutProps) {
  return (
    <div className={styles.readout} data-testid="context-readout">
      <Row label="FOCUS" value={focus} />
      <Row label="SURFACE" value={surface} />
      <Row label="SETTLEMENT" value={settlement} />
      <Row label="ACTIVITY" value={activity} accent />
    </div>
  );
}
