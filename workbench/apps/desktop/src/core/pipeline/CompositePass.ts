// Provenance: composite-pass construction extracted from the Gargantua donor
// (`finalPass` in `public/reference/gargantua.html`). Texture slots, BH-screen
// default (0.5, 0.5), lens default 0.1, and donor param default bloom 1.0
// preserved. See `src/core/PROVENANCE.md`.

import * as THREE from "three";
import { FINAL_FRAG } from "../shaders/composite.frag.glsl";
import { createQuadStage, makePass, type RenderPass } from "./RenderPass";

export interface CompositePass extends RenderPass {
  setFrame(
    scene: THREE.Texture | null,
    bloomA: THREE.Texture | null,
    bloomB: THREE.Texture | null,
    streak: THREE.Texture | null,
    bloom: number,
    time: number,
  ): void;
  setResolution(w: number, h: number): void;
  setBlackHoleScreen(u: number, v: number, lensR: number): void;
}

export function createCompositePass(quadGeo?: THREE.BufferGeometry): CompositePass {
  const geo = quadGeo ?? createQuadStage().quadGeo;
  const pass = makePass(
    geo,
    FINAL_FRAG,
    {
      tScene: { value: null },
      tBloomA: { value: null },
      tBloomB: { value: null },
      tStreak: { value: null },
      uBloom: { value: 1.0 },
      uTime: { value: 0 },
      uBHScreen: { value: new THREE.Vector2(0.5, 0.5) },
      uLensR: { value: 0.1 },
      uResolution: { value: new THREE.Vector2(1, 1) },
    },
    false,
  );
  return {
    ...pass,
    setFrame(scene, bloomA, bloomB, streak, bloom, time) {
      pass.uniforms.tScene.value = scene;
      pass.uniforms.tBloomA.value = bloomA;
      pass.uniforms.tBloomB.value = bloomB;
      pass.uniforms.tStreak.value = streak;
      pass.uniforms.uBloom.value = bloom;
      pass.uniforms.uTime.value = time;
    },
    setResolution(w: number, h: number) {
      (pass.uniforms.uResolution.value as THREE.Vector2).set(w, h);
    },
    setBlackHoleScreen(u: number, v: number, lensR: number) {
      (pass.uniforms.uBHScreen.value as THREE.Vector2).set(u, v);
      pass.uniforms.uLensR.value = lensR;
    },
  };
}
