// Provenance: pass-construction helper extracted from the Gargantua donor
// (`makePass`, `quadGeo`, `orthoCam` in `public/reference/gargantua.html`).
// Donor construction preserved; the shared fullscreen quad + orthographic
// camera are owned here. See `src/core/PROVENANCE.md`.
//
// Deviation D-02 (documented): `THREE.PlaneBufferGeometry` (donor, r128) is
// `THREE.PlaneGeometry` (still buffer-backed in modern three). Verified the
// same 2x2 fullscreen triangle strip.

import * as THREE from "three";
import { QUAD_VERT, QUAD_VERT3 } from "../shaders/quad.vert.glsl";

export interface RenderPass {
  readonly mat: THREE.ShaderMaterial;
  readonly scene: THREE.Scene;
  readonly uniforms: Record<string, THREE.IUniform>;
}

/** Shared fullscreen quad + orthographic camera (donor module-level pair). */
export function createQuadStage(): {
  quadGeo: THREE.PlaneGeometry;
  orthoCam: THREE.OrthographicCamera;
} {
  const quadGeo = new THREE.PlaneGeometry(2, 2);
  const orthoCam = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
  return { quadGeo, orthoCam };
}

export function makePass(
  quadGeo: THREE.BufferGeometry,
  frag: string,
  uniforms: Record<string, THREE.IUniform>,
  glsl3: boolean,
): RenderPass {
  const opts: THREE.ShaderMaterialParameters = {
    vertexShader: glsl3 ? QUAD_VERT3 : QUAD_VERT,
    fragmentShader: frag,
    uniforms,
    depthTest: false,
    depthWrite: false,
  };
  if (glsl3) opts.glslVersion = THREE.GLSL3;
  const mat = new THREE.ShaderMaterial(opts);
  const scene = new THREE.Scene();
  scene.add(new THREE.Mesh(quadGeo, mat));
  return { mat, scene, uniforms };
}
