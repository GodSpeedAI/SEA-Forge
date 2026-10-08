// ZoomController boundary: zoom changes representational depth (surface →
// detail), never mere pixel magnification. The renderer maps a depth step to
// a modest orbit-radius change around the Home framing (see
// `CoreRenderer.setZoom`).

export class ZoomController {
  private depth = 0;
  private readonly listeners = new Set<(depth: number) => void>();

  get current(): number {
    return this.depth;
  }

  setDepth(depth: number): void {
    this.depth = Math.max(0, depth);
    for (const fn of this.listeners) fn(this.depth);
  }

  zoomIn(step = 1): void {
    this.setDepth(this.depth + step);
  }

  zoomOut(step = 1): void {
    this.setDepth(this.depth - step);
  }

  subscribe(fn: (depth: number) => void): () => void {
    this.listeners.add(fn);
    return () => {
      this.listeners.delete(fn);
    };
  }
}

/** Singleton for the application lifetime. */
export const zoomController = new ZoomController();
