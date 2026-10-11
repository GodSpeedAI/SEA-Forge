// Semantic visual state for CORE. This is *visual intent*, not domain truth:
// the renderer consumes it; nothing authoritative is derived from it.
// Domain state reaches here only through `projections/CoreVisualProjection`.
// See `src/core/PROVENANCE.md` and `docs/CORE_ARCHITECTURE.md` §6.

/** Donor renderer defaults. Preserved (`{ mass: 0.72, spin: 0.65, ... }`). */
export const DEFAULT_VISUAL_PARAMS = {
  mass: 0.72,
  spin: 0.65,
  temp: 0.42,
  bloom: 1.0,
} as const;

export interface CoreVisualParams {
  mass: number;
  spin: number;
  temp: number;
  bloom: number;
}

export type CoreFocusKind = "core" | "object" | "anchor";

export interface CoreFocus {
  kind: CoreFocusKind;
  /** Focused object id when kind === "object"; undefined for CORE/Home. */
  objectId?: string;
  /** 0 = fully CORE-centered (Home); 1 = attention fully on the object. */
  emphasis: number;
}

export interface CoreCameraIntent {
  /** Desired orbit radius multiplier relative to Home (1 = canonical). */
  distance?: number;
  /** Desired inclination offset in radians relative to Home. */
  inclination?: number;
  /** Request a smooth return to the canonical Home framing. */
  returnHome?: boolean;
}

export interface CoreVisualState {
  params: CoreVisualParams;
  focus: CoreFocus;
  /** Representational depth: 0 surface … N deep. Not pixel magnification. */
  zoom: number;
  /** 0 = quiescent … 1 = highly active. Visual projection only. */
  activity: number;
}

export const HOME_VISUAL_STATE: CoreVisualState = {
  params: { ...DEFAULT_VISUAL_PARAMS },
  focus: { kind: "core", emphasis: 0 },
  zoom: 0,
  activity: 0.2,
};

export function homeVisualState(): CoreVisualState {
  return {
    params: { ...DEFAULT_VISUAL_PARAMS },
    focus: { kind: "core", emphasis: 0 },
    zoom: 0,
    activity: 0.2,
  };
}
