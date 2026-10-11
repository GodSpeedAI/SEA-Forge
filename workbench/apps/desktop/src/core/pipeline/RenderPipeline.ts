// Provenance: frame ordering extracted from the Gargantua donor (`step(now)`
// in `public/reference/gargantua.html`). The five-stage pipeline and its
// exact sequencing are preserved:
//
//   scene raymarch -> bright extraction -> bloom pyramid -> anamorphic streak
//   -> final composite -> screen
//
// See `src/core/PROVENANCE.md`. This module owns pass objects and per-frame
// uniform writes; GL context, camera, quality, and the rAF loop live in
// `CoreRenderer` so each donor seam stays independently testable.

import * as THREE from "three";
import { createQuadStage, type RenderPass } from "./RenderPass";
import { MAX_DEVICE_PIXEL_RATIO, RenderTargets, type CoreRenderTargets } from "./RenderTargets";
import { createScenePass, type ScenePassHandles } from "./ScenePass";
import { createBrightPass } from "./BrightPass";
import { createBloomPass, type BloomPass, BLUR_DIR_SCALE } from "./BloomPass";
import {
  createStreakPass,
  type StreakPass,
  STREAK_STRETCH_FIRST,
  STREAK_STRETCH_SECOND,
} from "./StreakPass";
import { createCompositePass, type CompositePass } from "./CompositePass";

export interface CoreFrameParams {
  mass: number;
  spin: number;
  temp: number;
  bloom: number;
}

export class RenderPipeline {
  readonly scenePass: ScenePassHandles;
  readonly brightPass: RenderPass;
  readonly bloomPass: BloomPass;
  readonly streakPass: StreakPass;
  readonly compositePass: CompositePass;
  readonly orthoCam: THREE.OrthographicCamera;
  readonly targets = new RenderTargets();

  constructor() {
    // Donor shared one quadGeo across all passes; preserved.
    const { quadGeo, orthoCam } = createQuadStage();
    this.orthoCam = orthoCam;
    this.scenePass = createScenePass(quadGeo);
    this.brightPass = createBrightPass(quadGeo);
    this.bloomPass = createBloomPass(quadGeo);
    this.streakPass = createStreakPass(quadGeo);
    this.compositePass = createCompositePass(quadGeo);
  }

  /**
   * Donor `resize()` target portion, preserved: render size from the
   * DPR-capped canvas size times the quality scale; pyramid by bit-shift.
   * Returns the canvas pixel size for the composite pass + renderer sizing.
   */
  resize(renderer: THREE.WebGLRenderer, qualityScale: number): { w: number; h: number } {
    const dpr = Math.min(window.devicePixelRatio || 1, MAX_DEVICE_PIXEL_RATIO);
    const w = Math.floor(window.innerWidth * dpr);
    const h = Math.floor(window.innerHeight * dpr);
    renderer.setSize(w, h, false);

    const sw = Math.max(1, Math.floor(w * qualityScale));
    const sh = Math.max(1, Math.floor(h * qualityScale));
    this.targets.rebuild(sw, sh);

    this.scenePass.setResolution(sw, sh);
    this.compositePass.setResolution(w, h);
    return { w, h };
  }

  /**
   * Donor `step` render portion, preserved in order and constants:
   *  1. raymarch scene
   *  2. bright extract at half res
   *  3. bloom pyramid (tight gaussian; downsample to quarter via a single
   *     horizontal blur; two double-gaussians for the wide halo)
   *  4. anamorphic streak from the quarter bloom (2.6, then 5.2)
   *  5. composite to screen
   */
  renderFrame(
    renderer: THREE.WebGLRenderer,
    params: CoreFrameParams,
    time: number,
    steps: number,
  ): void {
    const t = this.targets.targets as CoreRenderTargets;
    const blit = (pass: RenderPass, target: THREE.WebGLRenderTarget | null) => {
      renderer.setRenderTarget(target);
      renderer.render(pass.scene, this.orthoCam);
    };

    this.scenePass.setTime(time);
    this.scenePass.setSteps(steps);
    this.scenePass.setParams(params.mass, params.spin, params.temp);

    // 1 — raymarch scene
    blit(this.scenePass, t.rtScene);

    // 2 — bright extract at half res
    this.brightPass.uniforms.tSrc.value = t.rtScene.texture;
    blit(this.brightPass, t.rtHalfA);

    // 3 — bloom pyramid
    const hw = t.rtHalfA.width;
    const hh = t.rtHalfA.height;
    this.bloomPass.gaussian(blit, t.rtHalfA, t.rtHalfB, t.rtHalfA, hw, hh); // tight
    // downsample into quarter, blur twice for the wide halo
    this.bloomPass.setSource(t.rtHalfA.texture);
    this.bloomPass.setDirection(BLUR_DIR_SCALE / hw, 0);
    blit(this.bloomPass, t.rtQuarterA);
    this.bloomPass.gaussian(
      blit,
      t.rtQuarterA,
      t.rtQuarterB,
      t.rtQuarterA,
      t.rtQuarterA.width,
      t.rtQuarterA.height,
    );
    this.bloomPass.gaussian(
      blit,
      t.rtQuarterA,
      t.rtQuarterB,
      t.rtQuarterA,
      t.rtQuarterA.width,
      t.rtQuarterA.height,
    );

    // 4 — anamorphic streak from the quarter bloom
    this.streakPass.setSource(t.rtQuarterA.texture);
    this.streakPass.setTexel(STREAK_STRETCH_FIRST, t.rtQuarterA.width);
    blit(this.streakPass, t.rtStreakA);
    this.streakPass.setSource(t.rtStreakA.texture);
    this.streakPass.setTexel(STREAK_STRETCH_SECOND, t.rtQuarterA.width);
    blit(this.streakPass, t.rtStreakB);

    // 5 — composite to screen
    this.compositePass.setFrame(
      t.rtScene.texture,
      t.rtHalfA.texture,
      t.rtQuarterA.texture,
      t.rtStreakB.texture,
      params.bloom,
      time,
    );
    blit(this.compositePass, null);
  }

  dispose(): void {
    this.targets.dispose();
    for (const pass of [
      this.scenePass,
      this.brightPass,
      this.bloomPass,
      this.streakPass,
      this.compositePass,
    ]) {
      pass.mat.dispose();
      pass.scene.clear();
    }
  }
}
