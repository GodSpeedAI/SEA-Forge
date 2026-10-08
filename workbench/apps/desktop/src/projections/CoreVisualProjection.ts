// CoreVisualProjection: semantic state → visual intent for CORE.
// This is the ONLY path by which application state may influence the
// renderer, and it produces visual intent — never domain truth.
//
// Restraint rule (task §16): no arbitrary mappings of domain conditions onto
// renderer physics. Mass and spin stay at donor defaults (dev-tunable only
// via `dev/CoreTuningPanel`). Activity modulates bloom/temperature within a
// narrow expressive band around the donor look; focus emphasis shifts the
// camera framing, never the black-hole physics.

import type { CoreVisualParams } from "../core/CoreVisualState";
import { DEFAULT_VISUAL_PARAMS } from "../core/CoreVisualState";

export interface CoreVisualSignals {
  /** 0 = quiescent … 1 = highly active. Derived from real pending work. */
  activity: number;
  /** True while attention rests on a non-CORE object. */
  focusDisplaced: boolean;
}

export interface CoreVisualIntent {
  params: CoreVisualParams;
  activity: number;
}

/** Activity band: bloom ±0.15, temp ±0.05 around donor defaults. */
const BLOOM_BAND = 0.15;
const TEMP_BAND = 0.05;

export function projectCoreVisual(signals: CoreVisualSignals): CoreVisualIntent {
  const activity = Math.min(1, Math.max(0, signals.activity));
  return {
    params: {
      // Donor physics preserved: mass/spin are not semantic dials.
      mass: DEFAULT_VISUAL_PARAMS.mass,
      spin: DEFAULT_VISUAL_PARAMS.spin,
      temp: DEFAULT_VISUAL_PARAMS.temp + (activity - 0.2) * TEMP_BAND,
      bloom:
        DEFAULT_VISUAL_PARAMS.bloom +
        (activity - 0.2) * BLOOM_BAND -
        (signals.focusDisplaced ? 0.1 : 0),
    },
    activity,
  };
}

/** Derive the activity signal from real pending-work counts. */
export function activityFromPendingWork(input: {
  pendingApprovals: number | null;
  unreadableCases: number | null;
}): number {
  const approvals = input.pendingApprovals ?? 0;
  const unreadable = input.unreadableCases ?? 0;
  if (approvals > 0 || unreadable > 0) return 0.75;
  return 0.2;
}
