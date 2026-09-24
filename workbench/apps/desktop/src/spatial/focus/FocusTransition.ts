// FocusTransition: cinematic travel between focus states. The transition is
// representational (arrangement + emphasis), never a renderer remount: the
// WebGL context persists and only camera intent + visual state change.

export interface FocusTransitionSpec {
  from: string;
  to: string;
  /** Donor-derived pacing: idle drift is 0.028 rad/s; transitions complete in ~1.2s. */
  durationMs: number;
}

export const FOCUS_TRANSITION_MS = 1200;

export function describeTransition(from: string, to: string): FocusTransitionSpec {
  return { from, to, durationMs: FOCUS_TRANSITION_MS };
}

/** Ease used for focus travel (smoothstep; no dependency). */
export function easeFocusTravel(t: number): number {
  const c = Math.min(1, Math.max(0, t));
  return c * c * (3 - 2 * c);
}
