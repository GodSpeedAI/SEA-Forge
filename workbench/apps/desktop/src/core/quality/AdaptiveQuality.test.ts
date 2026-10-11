import { describe, expect, it, vi } from "vitest";
import { AdaptiveQuality, QUALITY_LADDER } from "./AdaptiveQuality";

describe("AdaptiveQuality (donor hysteresis preserved)", () => {
  it("starts at the donor initial level", () => {
    const q = new AdaptiveQuality();
    expect(q.levelIndex).toBe(1);
    expect(q.level).toEqual(QUALITY_LADDER[1]);
  });

  it("ignores hidden-tab and degenerate frames like the donor guard", () => {
    const onChange = vi.fn();
    const q = new AdaptiveQuality(onChange);
    q.observeFrame(0.016, 3000, true);
    q.observeFrame(0.0005, 3000, false);
    q.observeFrame(0.3, 3000, false);
    expect(q.levelIndex).toBe(1);
    expect(onChange).not.toHaveBeenCalled();
  });

  it("downgrades past 34ms average and upgrades below 15ms, one step per 2200ms", () => {
    const onChange = vi.fn();
    const q = new AdaptiveQuality(onChange);
    // sustain heavy frames: avg climbs past 34ms, single step down
    // (40 frames span 2000ms < the 2200ms cooldown, so exactly one step)
    let now = 3000;
    for (let i = 0; i < 40; i++) {
      now += 50;
      q.observeFrame(0.05, now, false);
    }
    expect(q.levelIndex).toBe(2);
    expect(onChange).toHaveBeenCalledTimes(1);

    // cooldown blocks a second immediate step even under load
    for (let i = 0; i < 10; i++) {
      now += 50;
      q.observeFrame(0.05, now, false);
    }
    expect(q.levelIndex).toBe(2);

    // after the cooldown, sustained load steps down exactly once more
    now += 2500;
    for (let i = 0; i < 10; i++) {
      now += 50;
      q.observeFrame(0.05, now, false);
    }
    expect(q.levelIndex).toBe(3);
    expect(onChange).toHaveBeenCalledTimes(2);
  });

  it("recovers upward on sustained light frames", () => {
    const q = new AdaptiveQuality();
    let now = 3000;
    for (let i = 0; i < 120; i++) {
      now += 16;
      q.observeFrame(0.008, now, false);
    }
    // avg decays 20ms -> 8ms, crossing below 15ms: one step 1 -> 0, then rest
    expect(q.levelIndex).toBe(0);
    expect(q.averageFrameMs).toBeLessThan(15);
  });

  it("setMaxIndex bounds later signals without touching hysteresis", () => {
    const q = new AdaptiveQuality();
    q.setMaxIndex(1);
    let now = 3000;
    for (let i = 0; i < 80; i++) {
      now += 50;
      q.observeFrame(0.05, now, false);
    }
    expect(q.levelIndex).toBeLessThanOrEqual(1);
  });
});
