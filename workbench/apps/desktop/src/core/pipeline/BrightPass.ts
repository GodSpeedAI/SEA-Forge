// Provenance: bright-extract pass construction extracted from the Gargantua
// donor (`brightPass` in `public/reference/gargantua.html`). Threshold 0.55
// preserved. See `src/core/PROVENANCE.md`.

import * as THREE from "three";
import { BRIGHT_FRAG } from "../shaders/bright.frag.glsl";
import { createQuadStage, makePass, type RenderPass } from "./RenderPass";

/** Donor bright threshold. Preserved (`uThreshold: 0.55`). */
export const BRIGHT_THRESHOLD = 0.55;

export function createBrightPass(quadGeo?: THREE.BufferGeometry): RenderPass {
  const geo = quadGeo ?? createQuadStage().quadGeo;
  return makePass(
    geo,
    BRIGHT_FRAG,
    {
      tSrc: { value: null },
      uThreshold: { value: BRIGHT_THRESHOLD },
    },
    false,
  );
}
