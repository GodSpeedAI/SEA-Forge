// Provenance: render-target management extracted from the Gargantua donor
// (`makeRT`, the rtScene/rtHalf*/rtQuarter*/rtStreak* set, and `resize` in
// `public/reference/gargantua.html`). Sizes, formats, and DPR cap preserved.
// See `src/core/PROVENANCE.md`.

import * as THREE from "three";

/** Donor DPR cap. Preserved (`Math.min(devicePixelRatio, 1.35)`). */
export const MAX_DEVICE_PIXEL_RATIO = 1.35;

export interface CoreRenderTargets {
  rtScene: THREE.WebGLRenderTarget;
  rtHalfA: THREE.WebGLRenderTarget;
  rtHalfB: THREE.WebGLRenderTarget;
  rtQuarterA: THREE.WebGLRenderTarget;
  rtQuarterB: THREE.WebGLRenderTarget;
  rtStreakA: THREE.WebGLRenderTarget;
  rtStreakB: THREE.WebGLRenderTarget;
}

export function makeRT(w: number, h: number): THREE.WebGLRenderTarget {
  return new THREE.WebGLRenderTarget(Math.max(1, w), Math.max(1, h), {
    minFilter: THREE.LinearFilter,
    magFilter: THREE.LinearFilter,
    format: THREE.RGBAFormat,
    type: THREE.HalfFloatType,
    depthBuffer: false,
    stencilBuffer: false,
  });
}

function disposeAll(t: CoreRenderTargets | null): void {
  if (!t) return;
  t.rtScene.dispose();
  t.rtHalfA.dispose();
  t.rtHalfB.dispose();
  t.rtQuarterA.dispose();
  t.rtQuarterB.dispose();
  t.rtStreakA.dispose();
  t.rtStreakB.dispose();
}

/**
 * Donor target pyramid, preserved: full scene at render scale, half pair,
 * quarter pairs (bloom halo + streak source), streak pair.
 */
export function allocateTargets(sw: number, sh: number): CoreRenderTargets {
  return {
    rtScene: makeRT(sw, sh),
    rtHalfA: makeRT(sw >> 1, sh >> 1),
    rtHalfB: makeRT(sw >> 1, sh >> 1),
    rtQuarterA: makeRT(sw >> 2, sh >> 2),
    rtQuarterB: makeRT(sw >> 2, sh >> 2),
    rtStreakA: makeRT(sw >> 2, sh >> 2),
    rtStreakB: makeRT(sw >> 2, sh >> 2),
  };
}

export class RenderTargets {
  private current: CoreRenderTargets | null = null;

  get targets(): CoreRenderTargets | null {
    return this.current;
  }

  rebuild(sw: number, sh: number): CoreRenderTargets {
    disposeAll(this.current);
    this.current = allocateTargets(sw, sh);
    return this.current;
  }

  dispose(): void {
    disposeAll(this.current);
    this.current = null;
  }
}
