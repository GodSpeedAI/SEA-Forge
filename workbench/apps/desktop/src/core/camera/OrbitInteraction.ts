// Provenance: pointer/wheel/keyboard handling extracted from the Gargantua
// donor (`bhMain` input section in `public/reference/gargantua.html`). Donor
// deltas, factors, and key behavior preserved; DOM lookups replaced by
// constructor-injected references. See `src/core/PROVENANCE.md`.

import type { CoreCamera } from "./CoreCamera";

export interface OrbitInteractionCallbacks {
  onFirstInteraction?: () => void;
  onToggleChrome?: () => void;
}

const DRAG_FACTOR = 0.0032;

/**
 * Donor input wiring, preserved:
 * - pointerdown captures + tracks; pointermove applies dx*0.0032 /
 *   -dy*0.0032 impulses through the camera; pointerup/cancel ends drag
 * - wheel zooms exponentially (handled by CoreCamera.applyWheelZoom),
 *   non-passive with preventDefault
 * - `H` toggles surrounding chrome visibility (callback; the renderer never
 *   touches product DOM itself)
 */
export class OrbitInteraction {
  private readonly canvas: HTMLCanvasElement;
  private readonly camera: CoreCamera;
  private readonly callbacks: OrbitInteractionCallbacks;
  private dragging = false;
  private lastX = 0;
  private lastY = 0;
  private lastInputMs = 0;
  private readonly disposeFns: Array<() => void> = [];

  constructor(
    canvas: HTMLCanvasElement,
    camera: CoreCamera,
    callbacks: OrbitInteractionCallbacks = {},
  ) {
    this.canvas = canvas;
    this.camera = camera;
    this.callbacks = callbacks;
  }

  get lastInput(): number {
    return this.lastInputMs;
  }

  markInput(nowMs: number): void {
    this.lastInputMs = nowMs;
  }

  attach(): void {
    const canvas = this.canvas;
    const onPointerDown = (e: PointerEvent) => {
      this.dragging = true;
      this.camera.setDragging(true);
      this.lastX = e.clientX;
      this.lastY = e.clientY;
      canvas.classList.add("dragging");
      try {
        canvas.setPointerCapture(e.pointerId);
      } catch {
        // pointer capture is best-effort (donor had no guard; integration
        // must not throw on platforms without capture support)
      }
      this.callbacks.onFirstInteraction?.();
    };
    const onPointerMove = (e: PointerEvent) => {
      if (!this.dragging) return;
      const dx = e.clientX - this.lastX;
      const dy = e.clientY - this.lastY;
      this.lastX = e.clientX;
      this.lastY = e.clientY;
      this.camera.addImpulse(dx * DRAG_FACTOR, -dy * DRAG_FACTOR);
      this.lastInputMs = performance.now();
    };
    const endDrag = () => {
      this.dragging = false;
      this.camera.setDragging(false);
      canvas.classList.remove("dragging");
    };
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      this.camera.applyWheelZoom(e.deltaY);
      this.lastInputMs = performance.now();
      this.callbacks.onFirstInteraction?.();
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key.toLowerCase() === "h") this.callbacks.onToggleChrome?.();
    };

    canvas.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", endDrag);
    // stylus/gesture interruption (donor comment preserved)
    window.addEventListener("pointercancel", endDrag);
    canvas.addEventListener("wheel", onWheel, { passive: false });
    window.addEventListener("keydown", onKeyDown);

    this.disposeFns.push(
      () => canvas.removeEventListener("pointerdown", onPointerDown),
      () => window.removeEventListener("pointermove", onPointerMove),
      () => window.removeEventListener("pointerup", endDrag),
      () => window.removeEventListener("pointercancel", endDrag),
      () => canvas.removeEventListener("wheel", onWheel),
      () => window.removeEventListener("keydown", onKeyDown),
    );
  }

  dispose(): void {
    for (const fn of this.disposeFns.splice(0)) {
      try {
        fn();
      } catch {
        // dispose must not throw
      }
    }
    this.dragging = false;
  }
}
