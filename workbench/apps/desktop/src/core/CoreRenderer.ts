// Provenance: renderer orchestration extracted from the Gargantua donor
// (`bhMain`, `blit`, `schedule`/`frame`/`step`, FPS meter, context handling
// in `public/reference/gargantua.html`). Donor lifecycle, loop, timing, and
// error semantics preserved; DOM lookups replaced by injected references and
// the public surface narrowed to start/stop/resize/dispose + visual-intent
// setters. Outside `src/core/`, application code must use this API — never
// Three.js targets, uniforms, or camera vectors directly.
// See `src/core/PROVENANCE.md`.
//
// Deviation D-01 (documented): THREE is an npm import, not a CDN script tag.
// Deviation D-03 (documented): the fatal-error card is a callback
// (`onFatal`) rendering the white-labeled `RenderFailureSurface`, not the
// donor's `bhFatal` DOM writes. Same failure taxonomy, same messages.

import * as THREE from "three";
import { CoreCamera, HOME_CAMERA_STATE } from "./camera/CoreCamera";
import { OrbitInteraction } from "./camera/OrbitInteraction";
import { AdaptiveQuality } from "./quality/AdaptiveQuality";
import { RenderPipeline } from "./pipeline/RenderPipeline";
import {
  DEFAULT_VISUAL_PARAMS,
  type CoreCameraIntent,
  type CoreFocus,
  type CoreVisualParams,
  type CoreVisualState,
} from "./CoreVisualState";

export interface CoreFatalInfo {
  title: string;
  message: string;
  detail: string;
}

export interface CoreRendererOptions {
  canvas: HTMLCanvasElement;
  onFatal?: (info: CoreFatalInfo) => void;
  onFps?: (fps: number) => void;
  /** Hidden-tab scheduling fallback delay. Donor: 40ms. Preserved. */
  hiddenFrameDelayMs?: number;
}

function fatal(
  onFatal: ((info: CoreFatalInfo) => void) | undefined,
  title: string,
  message: string,
  detail: string,
): void {
  // Donor logged `[GARGANTUA] title — detail`; keep the console taxonomy.
  console.error(`[CORE] ${title} — ${detail || message}`);
  onFatal?.({ title, message, detail });
}

/**
 * CORE renderer. Narrow public API:
 * start / stop / resize / dispose / setVisualState / setFocus /
 * setCameraIntent / setZoom. Everything GPU-internal stays inside.
 */
export class CoreRenderer {
  private readonly options: CoreRendererOptions;
  private renderer: THREE.WebGLRenderer | null = null;
  private readonly pipeline = new RenderPipeline();
  private readonly camera = new CoreCamera();
  private readonly quality: AdaptiveQuality;
  private interaction: OrbitInteraction | null = null;
  private running = false;
  private rafId = 0;
  private timeoutId = 0;
  private prevMs = 0;
  private frames = 0;
  private fpsWindowStart = 0;
  private disposed = false;
  private lastInputMs = 0;
  private resizeListener: (() => void) | null = null;
  private contextLostListener: ((e: Event) => void) | null = null;

  private params: CoreVisualParams = { ...DEFAULT_VISUAL_PARAMS };
  private focus: CoreFocus = { kind: "core", emphasis: 0 };
  private zoom = 0;
  private activity = 0.2;

  constructor(options: CoreRendererOptions) {
    this.options = options;
    this.quality = new AdaptiveQuality(() => {
      if (this.renderer) this.resizeTargets();
    });
  }

  get canvas(): HTMLCanvasElement {
    return this.options.canvas;
  }

  get cameraState(): { theta: number; phi: number; radius: number } {
    return { ...this.camera.state };
  }

  get qualityIndex(): number {
    return this.quality.levelIndex;
  }

  get focusState(): CoreFocus {
    return { ...this.focus };
  }

  get zoomDepth(): number {
    return this.zoom;
  }

  get activityLevel(): number {
    return this.activity;
  }

  /** Donor preflight + context creation, preserved in order and messages. */
  start(): boolean {
    if (this.running || this.disposed) return this.running;
    const canvas = this.options.canvas;

    // --- preflight: the two things that actually stop this from running ---
    if (!canvas.getContext("webgl2")) {
      fatal(
        this.options.onFatal,
        "WEBGL2 UNAVAILABLE",
        "This renderer needs WebGL2 — the scene pass is a GLSL3 shader. Try a current Chrome, Firefox, Edge or Safari 15+, and make sure hardware acceleration is enabled.",
        `navigator: ${navigator.userAgent}`,
      );
      return false;
    }

    try {
      this.renderer = new THREE.WebGLRenderer({
        canvas,
        antialias: false,
        powerPreference: "high-performance",
      });
    } catch (e) {
      fatal(
        this.options.onFatal,
        "CONTEXT CREATION FAILED",
        "The browser refused to create a WebGL context. This usually means hardware acceleration is disabled or the GPU driver is blocklisted.",
        String((e as Error)?.message ?? e),
      );
      return false;
    }
    this.renderer.setClearColor(0x000000, 1);

    const onContextLost = (e: Event) => {
      e.preventDefault();
      fatal(
        this.options.onFatal,
        "CONTEXT LOST",
        "The GPU dropped the WebGL context — usually a driver reset or the tab being starved of GPU memory. Reload the page to restart the render.",
        "webglcontextlost",
      );
    };
    canvas.addEventListener("webglcontextlost", onContextLost, false);
    this.contextLostListener = onContextLost;

    this.interaction = new OrbitInteraction(canvas, this.camera, {
      onFirstInteraction: () => {
        this.lastInputMs = performance.now();
      },
      onToggleChrome: () => {
        window.dispatchEvent(new CustomEvent("core:toggle-chrome"));
      },
    });
    this.interaction.attach();
    this.lastInputMs = performance.now();

    const onResize = () => this.resizeTargets();
    window.addEventListener("resize", onResize);
    this.resizeListener = onResize;
    this.resizeTargets();

    this.running = true;
    this.prevMs = performance.now();
    this.fpsWindowStart = this.prevMs;
    this.frames = 0;
    this.schedule();
    return true;
  }

  stop(): void {
    this.running = false;
    if (this.rafId) {
      cancelAnimationFrame(this.rafId);
      this.rafId = 0;
    }
    if (this.timeoutId) {
      clearTimeout(this.timeoutId);
      this.timeoutId = 0;
    }
  }

  /** Rebuild render targets for the current canvas + quality level. */
  resize(): void {
    if (this.renderer) this.resizeTargets();
  }

  dispose(): void {
    this.stop();
    this.disposed = true;
    if (this.resizeListener) {
      window.removeEventListener("resize", this.resizeListener);
      this.resizeListener = null;
    }
    if (this.contextLostListener) {
      this.canvas.removeEventListener("webglcontextlost", this.contextLostListener);
      this.contextLostListener = null;
    }
    this.interaction?.dispose();
    this.interaction = null;
    this.pipeline.dispose();
    this.renderer?.dispose();
    this.renderer = null;
  }

  setVisualState(state: Partial<CoreVisualState>): void {
    if (state.params) this.params = { ...this.params, ...state.params };
    if (state.focus) this.focus = { ...state.focus };
    if (state.zoom !== undefined) this.zoom = state.zoom;
    if (state.activity !== undefined) this.activity = state.activity;
  }

  setFocus(focus: CoreFocus): void {
    this.focus = { ...focus };
    // Focus is semantic intent; the canonical Home framing restores when
    // attention returns to CORE. Camera travel itself is owned by the
    // FocusController -> setCameraIntent path; the renderer only records it.
    if (focus.kind === "core") this.camera.resetToHome();
  }

  setCameraIntent(intent: CoreCameraIntent): void {
    if (intent.returnHome) {
      this.camera.resetToHome();
      return;
    }
    if (intent.distance !== undefined) {
      this.camera.setRadius(HOME_CAMERA_STATE.radius * intent.distance);
    }
    if (intent.inclination !== undefined) {
      this.camera.state.phi = Math.min(
        Math.PI - 0.12,
        Math.max(0.12, HOME_CAMERA_STATE.phi + intent.inclination),
      );
    }
  }

  setZoom(zoom: number): void {
    // Zoom is representational depth; the donor zoomed the orbit radius, so a
    // depth step maps to a modest radius change around the Home framing.
    this.zoom = zoom;
    const depthScale = Math.pow(1.12, -zoom);
    this.camera.setRadius(HOME_CAMERA_STATE.radius * depthScale);
  }

  // -- internals (donor `resize` / `schedule` / `frame` / `step`) --

  private resizeTargets(): void {
    if (!this.renderer) return;
    this.pipeline.resize(this.renderer, this.quality.level.scale);
  }

  // rAF normally; setTimeout fallback keeps rendering if the tab is hidden
  private schedule(): void {
    if (!this.running || this.disposed) return;
    if (document.hidden) {
      this.timeoutId = window.setTimeout(
        () => this.frame(performance.now()),
        this.options.hiddenFrameDelayMs ?? 40,
      );
    } else {
      this.rafId = requestAnimationFrame((now) => this.frame(now));
    }
  }

  private frame(now: number): void {
    if (!this.running || this.disposed) return;
    this.schedule();
    this.step(now);
  }

  /** Test seam: advance one frame with an explicit timestamp. */
  stepForTest(now: number): void {
    this.step(now);
  }

  private step(now: number): void {
    if (!this.renderer) return;
    // Donor clamp: min(0.05, delta). Preserved.
    const dt = Math.min(0.05, (now - this.prevMs) / 1000);
    this.prevMs = now;
    const t = now / 1000;

    this.camera.update(dt, now, this.interaction?.lastInput ?? this.lastInputMs);
    const uniforms = this.pipeline.scenePass.uniforms;
    (uniforms.uCamPos.value as THREE.Vector3).copy(this.camera.camPos);
    (uniforms.uCamBasis.value as THREE.Matrix3).copy(this.camera.basis);

    // project BH centre (world origin) to screen uv for CA / flare
    const aspect = window.innerWidth / Math.max(1, window.innerHeight);
    const proj = this.camera.projectOrigin(aspect, this.params.mass);
    if (proj) this.pipeline.compositePass.setBlackHoleScreen(proj.u, proj.v, proj.lensR);

    // adaptive quality: track real frame cost, step ladder with hysteresis
    this.quality.observeFrame(dt, now, document.hidden);

    this.pipeline.renderFrame(this.renderer, this.params, t, this.quality.level.steps);

    // fps (donor 800ms window; callback replaces the donor #fps DOM write)
    this.frames++;
    if (now - this.fpsWindowStart > 800) {
      const fps = Math.round((this.frames * 1000) / (now - this.fpsWindowStart));
      this.options.onFps?.(fps);
      this.frames = 0;
      this.fpsWindowStart = now;
    }
  }
}
