// Provenance: scene-pass construction extracted from the Gargantua donor
// (`sceneFragFinal` + `scenePass` in `public/reference/gargantua.html`).
// Uniform set, defaults, and GLSL3 selection preserved. See PROVENANCE.md.

import * as THREE from "three";
import { buildSceneFrag } from "../shaders/core.frag.glsl";
import { CORE_TAN_HALF_FOV } from "../camera/CoreCamera";
import { createQuadStage, makePass, type RenderPass } from "./RenderPass";

export interface ScenePassHandles extends RenderPass {
  setResolution(w: number, h: number): void;
  setTime(t: number): void;
  setSteps(steps: number): void;
  setParams(rs: number, spin: number, temp: number): void;
}

export function createScenePass(quadGeo?: THREE.BufferGeometry): ScenePassHandles {
  const geo = quadGeo ?? createQuadStage().quadGeo;
  const pass = makePass(
    geo,
    buildSceneFrag(),
    {
      uResolution: { value: new THREE.Vector2(1, 1) },
      uTime: { value: 0 },
      uCamPos: { value: new THREE.Vector3() },
      uCamBasis: { value: new THREE.Matrix3() },
      uTanHalfFov: { value: CORE_TAN_HALF_FOV },
      // Donor defaults: { mass: 0.72, spin: 0.65, temp: 0.42, bloom: 1.0 }
      uRs: { value: 0.72 },
      uSpin: { value: 0.65 },
      uTemp: { value: 0.42 },
      uSteps: { value: 260 },
    },
    true,
  );
  return {
    ...pass,
    setResolution(w: number, h: number) {
      (pass.uniforms.uResolution.value as THREE.Vector2).set(w, h);
    },
    setTime(t: number) {
      pass.uniforms.uTime.value = t;
    },
    setSteps(steps: number) {
      pass.uniforms.uSteps.value = steps;
    },
    setParams(rs: number, spin: number, temp: number) {
      pass.uniforms.uRs.value = rs;
      pass.uniforms.uSpin.value = spin;
      pass.uniforms.uTemp.value = temp;
    },
  };
}
