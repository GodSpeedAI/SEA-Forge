// Provenance: anamorphic-streak pass construction extracted from the Gargantua
// donor (`streakPass` in `public/reference/gargantua.html`). Two-stage
// stretches 2.6 then 5.2 preserved at the call site (RenderPipeline). See
// `src/core/PROVENANCE.md`.

import * as THREE from "three";
import { STREAK_FRAG } from "../shaders/streak.frag.glsl";
import { createQuadStage, makePass, type RenderPass } from "./RenderPass";

/** Donor streak stretches, first then second stage. Preserved. */
export const STREAK_STRETCH_FIRST = 2.6;
export const STREAK_STRETCH_SECOND = 5.2;

export interface StreakPass extends RenderPass {
  setSource(texture: THREE.Texture | null): void;
  setTexel(stretch: number, width: number): void;
}

export function createStreakPass(quadGeo?: THREE.BufferGeometry): StreakPass {
  const geo = quadGeo ?? createQuadStage().quadGeo;
  const pass = makePass(
    geo,
    STREAK_FRAG,
    {
      tSrc: { value: null },
      uTexelX: { value: 0 },
      uStretch: { value: 3.0 },
    },
    false,
  );
  return {
    ...pass,
    setSource(texture: THREE.Texture | null) {
      pass.uniforms.tSrc.value = texture;
    },
    setTexel(stretch: number, width: number) {
      pass.uniforms.uTexelX.value = 1 / width;
      pass.uniforms.uStretch.value = stretch;
    },
  };
}
