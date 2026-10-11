import { describe, expect, it } from "vitest";
import { FocusController } from "./focus/FocusController";
import { describeTransition, easeFocusTravel, FOCUS_TRANSITION_MS } from "./focus/FocusTransition";
import { ZoomController } from "./zoom/ZoomController";
import { TimeVersionController } from "./time/TimeVersionController";

describe("spatial grammar controllers", () => {
  it("FocusController: Home is the default; return restores it", () => {
    const fc = new FocusController();
    expect(fc.isHome()).toBe(true);
    expect(fc.toCoreFocus()).toEqual({ kind: "core", emphasis: 0 });
    fc.focus({ kind: "object", objectId: "case-1" });
    expect(fc.isHome()).toBe(false);
    expect(fc.toCoreFocus()).toEqual({ kind: "object", objectId: "case-1", emphasis: 1 });
    fc.returnHome();
    expect(fc.isHome()).toBe(true);
  });

  it("FocusController notifies subscribers; unsubscribe stops delivery", () => {
    const fc = new FocusController();
    const seen: string[] = [];
    const unsub = fc.subscribe((t) => seen.push(t.kind));
    fc.focus({ kind: "anchor" });
    unsub();
    fc.returnHome();
    expect(seen).toEqual(["anchor"]);
  });

  it("FocusTransition describes bounded cinematic travel with smoothstep ease", () => {
    expect(describeTransition("core", "case-1")).toEqual({
      from: "core",
      to: "case-1",
      durationMs: FOCUS_TRANSITION_MS,
    });
    expect(easeFocusTravel(0)).toBe(0);
    expect(easeFocusTravel(1)).toBe(1);
    expect(easeFocusTravel(0.5)).toBeCloseTo(0.5, 6);
    expect(easeFocusTravel(-1)).toBe(0);
    expect(easeFocusTravel(2)).toBe(1);
  });

  it("ZoomController is depth, clamped at zero, observable", () => {
    const zc = new ZoomController();
    expect(zc.current).toBe(0);
    zc.zoomIn(2);
    expect(zc.current).toBe(2);
    zc.zoomOut(5);
    expect(zc.current).toBe(0);
    const seen: number[] = [];
    const unsub = zc.subscribe((d) => seen.push(d));
    zc.setDepth(3);
    unsub();
    zc.setDepth(4);
    expect(seen).toEqual([3]);
  });

  it("TimeVersionController rests at live head; pins are opaque cursors", () => {
    const tc = new TimeVersionController();
    expect(tc.isLive()).toBe(true);
    tc.pin({ cursor: "led_01HX", label: "evt 1182" });
    expect(tc.isLive()).toBe(false);
    expect(tc.current).toEqual({ cursor: "led_01HX", label: "evt 1182" });
    tc.returnToLive();
    expect(tc.isLive()).toBe(true);
    expect(tc.current).toBeNull();
  });
});
