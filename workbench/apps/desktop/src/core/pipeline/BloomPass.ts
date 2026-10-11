// Provenance: gaussian bloom helper extracted from the Gargantua donor
// (`blurPass` + `gaussian(src, tmp, out, w, h)` in
// `public/reference/gargantua.html`). Separable 9-tap weights live in the
// preserved `BLUR_FRAG`; the 1.6-texel direction scale is preserved here.
// See `src/core/PROVENANCE.md`.

import * as THREE from "three";
import { BLUR_FRAG } from "../shaders/blur.frag.glsl";
import { createQuadStage, makePass, type RenderPass } from "./RenderPass";

/** Donor blur direction scale. Preserved (`1.6 / w`, `1.6 / h`). */
export const BLUR_DIR_SCALE = 1.6;

export interface BloomPass extends RenderPass {
  /** Donor `gaussian(src, tmp, out, w, h)`: H blur into tmp, V blur to out. */
  gaussian(
    render: (pass: RenderPass, target: THREE.WebGLRenderTarget) => void,
    src: THREE.WebGLRenderTarget,
    tmp: THREE.WebGLRenderTarget,
    out: THREE.WebGLRenderTarget,
    w: number,
    h: number,
  ): void;
  setSource(texture: THREE.Texture | null): void;
  setDirection(x: number, y: number): void;
}

export function createBloomPass(quadGeo?: THREE.BufferGeometry): BloomPass {
  const geo = quadGeo ?? createQuadStage().quadGeo;
  const pass = makePass(
    geo,
    BLUR_FRAG,
    {
      tSrc: { value: null },
      uDir: { value: new THREE.Vector2() },
    },
    false,
  );
  return {
    ...pass,
    gaussian(render, src, tmp, out, w, h) {
      pass.uniforms.tSrc.value = src.texture;
      (pass.uniforms.uDir.value as THREE.Vector2).set(BLUR_DIR_SCALE / w, 0);
      render(pass, tmp);
      pass.uniforms.tSrc.value = tmp.texture;
      (pass.uniforms.uDir.value as THREE.Vector2).set(0, BLUR_DIR_SCALE / h);
      render(pass, out);
    },
    setSource(texture: THREE.Texture | null) {
      pass.uniforms.tSrc.value = texture;
    },
    setDirection(x: number, y: number) {
      (pass.uniforms.uDir.value as THREE.Vector2).set(x, y);
    },
  };
}
