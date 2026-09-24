// Provenance: adaptive-quality ladder + frame-time hysteresis extracted from
// the Gargantua donor (`QUALITY`, `qi`, `qAvg`, `lastQChange` and the per-frame
// block in `public/reference/gargantua.html`). Behavior preserved verbatim;
// globals converted to class state with an `onQualityChange` boundary so the
// renderer can rebuild render targets. See `src/core/PROVENANCE.md`.

export interface QualityLevel {
  scale: number;
  steps: number;
}

/** Donor adaptive quality ladder: render scale + march steps. Preserved. */
export const QUALITY_LADDER: readonly QualityLevel[] = [
  { scale: 0.8, steps: 280 },
  { scale: 0.66, steps: 230 },
  { scale: 0.54, steps: 190 },
  { scale: 0.44, steps: 150 },
];

/** Donor initial level index. Preserved (`let qi = 1`). */
export const INITIAL_QUALITY_INDEX = 1;

const AVG_ALPHA = 0.92;
const AVG_BETA = 0.08;
const INITIAL_AVG_MS = 20;
const DOWNGRADE_MS = 34;
const UPGRADE_MS = 15;
const CHANGE_COOLDOWN_MS = 2200;

/**
 * Donor adaptive-quality controller, preserved. Tracks an exponential moving
 * average of real frame cost and steps the ladder with hysteresis:
 * downgrade when avg > 34ms, upgrade when avg < 15ms, at most one step per
 * 2200ms. Skips hidden-tab and degenerate/outlier frames exactly as donor.
 *
 * Extension boundary (task §9): additional signals (visibility, device class,
 * reduced motion, CORE focus) can clamp `maxIndex` / force a level through
 * `requestLevel` without touching the hysteresis core.
 */
export class AdaptiveQuality {
  private readonly onChange: ((index: number) => void) | undefined;
  private index: number = INITIAL_QUALITY_INDEX;
  private avgMs: number = INITIAL_AVG_MS;
  private lastChangeMs = 0;
  private maxIndex: number = QUALITY_LADDER.length - 1;

  constructor(onChange?: (index: number) => void) {
    this.onChange = onChange;
  }

  get levelIndex(): number {
    return this.index;
  }

  get level(): QualityLevel {
    return QUALITY_LADDER[this.index];
  }

  get averageFrameMs(): number {
    return this.avgMs;
  }

  /** Clamp the worst allowed level (future signals: battery, background). */
  setMaxIndex(max: number): void {
    this.maxIndex = Math.max(0, Math.min(QUALITY_LADDER.length - 1, max));
    if (this.index > this.maxIndex) this.setIndex(this.maxIndex, performance.now());
  }

  /** Force a level (future signals: reduced motion, hidden CORE). */
  requestLevel(index: number, nowMs = performance.now()): void {
    this.setIndex(Math.max(0, Math.min(this.maxIndex, index)), nowMs);
  }

  /**
   * Donor per-frame block, preserved. `dtSeconds` is the clamped frame delta
   * in seconds; `hidden` mirrors `document.hidden`.
   */
  observeFrame(dtSeconds: number, nowMs: number, hidden: boolean): void {
    if (hidden || dtSeconds <= 0.001 || dtSeconds >= 0.25) return;
    this.avgMs = this.avgMs * AVG_ALPHA + dtSeconds * 1000 * AVG_BETA;
    if (nowMs - this.lastChangeMs > CHANGE_COOLDOWN_MS) {
      if (this.avgMs > DOWNGRADE_MS && this.index < this.maxIndex) {
        this.setIndex(this.index + 1, nowMs);
      } else if (this.avgMs < UPGRADE_MS && this.index > 0) {
        this.setIndex(this.index - 1, nowMs);
      }
    }
  }

  private setIndex(next: number, nowMs: number): void {
    if (next === this.index) return;
    this.index = next;
    this.lastChangeMs = nowMs;
    this.onChange?.(next);
  }
}
