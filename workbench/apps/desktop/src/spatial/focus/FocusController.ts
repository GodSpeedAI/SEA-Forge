// FocusController owns semantic intent: which object is the current local
// center of attention. CoreCamera/CoreRenderer execute visual intent.
// Home is the canonical CORE-centered orientation state: no secondary object
// owns focus, CORE occupies the canonical center.

import type { CoreFocus } from "../../core/CoreVisualState";

export type FocusTarget = { kind: "core" } | { kind: "object"; objectId: string } | { kind: "anchor" };

export class FocusController {
  private target: FocusTarget = { kind: "core" };
  private readonly listeners = new Set<(target: FocusTarget) => void>();

  get current(): FocusTarget {
    return this.target;
  }

  isHome(): boolean {
    return this.target.kind === "core";
  }

  focus(target: FocusTarget): void {
    this.target = target;
    for (const fn of this.listeners) fn(target);
  }

  /** Returning to CORE restores the canonical Home state. */
  returnHome(): void {
    this.focus({ kind: "core" });
  }

  toCoreFocus(): CoreFocus {
    if (this.target.kind === "object") {
      return { kind: "object", objectId: this.target.objectId, emphasis: 1 };
    }
    if (this.target.kind === "anchor") {
      return { kind: "anchor", emphasis: 0.5 };
    }
    return { kind: "core", emphasis: 0 };
  }

  subscribe(fn: (target: FocusTarget) => void): () => void {
    this.listeners.add(fn);
    return () => {
      this.listeners.delete(fn);
    };
  }
}

/** Singleton for the application lifetime (renderer outlives navigation). */
export const focusController = new FocusController();
