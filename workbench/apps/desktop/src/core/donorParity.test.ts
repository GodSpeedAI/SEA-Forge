// Donor-parity gate: the extracted CORE renderer must remain substantially
// verbatim Gargantua donor code. These tests compare every shader module
// against the untouched golden reference (`public/reference/gargantua.html`)
// and pin the preserved numerical contracts. If an extraction edit changes
// donor behavior, this suite fails first.

import { describe, expect, it } from "vitest";
import goldenHtml from "../../public/reference/gargantua.html?raw";
import { NOISE_GLSL } from "./shaders/noise.glsl";
import { QUAD_VERT, QUAD_VERT3 } from "./shaders/quad.vert.glsl";
import { SCENE_FRAG_TEMPLATE, buildSceneFrag } from "./shaders/core.frag.glsl";
import {
  BLUR_FRAG,
  BRIGHT_FRAG,
  FINAL_FRAG,
  STREAK_FRAG,
} from "./shaders/post.frag.glsl";
import { QUALITY_LADDER, INITIAL_QUALITY_INDEX } from "./quality/AdaptiveQuality";
import { CORE_FOV, CORE_TAN_HALF_FOV, HOME_CAMERA_STATE } from "./camera/CoreCamera";
import { DEFAULT_VISUAL_PARAMS } from "./CoreVisualState";

function normalize(s: string): string {
  return s.replace(/\s+/g, " ").trim();
}

/** Extract a donor `const NAME = ...` template body from the golden HTML. */
function donorBlock(name: string): string {
  const marker = `const ${name} = /* glsl */\``;
  const start = goldenHtml.indexOf(marker);
  if (start === -1) throw new Error(`donor block ${name} not found in golden reference`);
  const bodyStart = start + marker.length;
  const end = goldenHtml.indexOf("`;", bodyStart);
  if (end === -1) throw new Error(`donor block ${name} unterminated`);
  return goldenHtml.slice(bodyStart, end);
}

describe("donor parity: shaders are verbatim Gargantua code", () => {
  it("NOISE_GLSL matches the donor block exactly (modulo whitespace)", () => {
    expect(normalize(NOISE_GLSL)).toBe(normalize(donorBlock("NOISE_GLSL")));
  });

  it("fullscreen quad shaders match the donor blocks", () => {
    expect(normalize(QUAD_VERT)).toBe(normalize(donorBlock("QUAD_VERT")));
    expect(normalize(QUAD_VERT3)).toBe(normalize(donorBlock("QUAD_VERT3")));
  });

  it("scene template matches the donor SCENE_FRAG (same ${NOISE} splice point)", () => {
    const donorScene = donorBlock("SCENE_FRAG").replace("${'${NOISE}'}", "${NOISE}");
    expect(normalize(SCENE_FRAG_TEMPLATE)).toBe(normalize(donorScene));
  });

  it("buildSceneFrag performs the donor splice and nothing else", () => {
    const donorScene = donorBlock("SCENE_FRAG").replace("${'${NOISE}'}", "${NOISE}");
    const donorNoise = donorBlock("NOISE_GLSL");
    const expected = donorScene.replace("${NOISE}", donorNoise);
    expect(normalize(buildSceneFrag())).toBe(normalize(expected));
  });

  it("post shaders match the donor blocks", () => {
    expect(normalize(BRIGHT_FRAG)).toBe(normalize(donorBlock("BRIGHT_FRAG")));
    expect(normalize(BLUR_FRAG)).toBe(normalize(donorBlock("BLUR_FRAG")));
    expect(normalize(STREAK_FRAG)).toBe(normalize(donorBlock("STREAK_FRAG")));
    expect(normalize(FINAL_FRAG)).toBe(normalize(donorBlock("FINAL_FRAG")));
  });

  it("no Gargantua demo copy leaks into product shader modules", () => {
    const all = [NOISE_GLSL, QUAD_VERT, QUAD_VERT3, SCENE_FRAG_TEMPLATE, BRIGHT_FRAG, BLUR_FRAG, STREAK_FRAG, FINAL_FRAG].join("\n");
    expect(all).not.toMatch(/GARGANTUA/i);
  });
});

describe("donor parity: preserved numerical contracts", () => {
  it("adaptive quality ladder and initial index are donor values", () => {
    expect(QUALITY_LADDER).toEqual([
      { scale: 0.8, steps: 280 },
      { scale: 0.66, steps: 230 },
      { scale: 0.54, steps: 190 },
      { scale: 0.44, steps: 150 },
    ]);
    expect(INITIAL_QUALITY_INDEX).toBe(1);
    for (const entry of [
      "scale: 0.80, steps: 280",
      "scale: 0.66, steps: 230",
      "scale: 0.54, steps: 190",
      "scale: 0.44, steps: 150",
    ]) {
      expect(goldenHtml).toContain(entry);
    }
  });

  it("camera framing (FOV + Home orbit) is donor values", () => {
    expect(CORE_FOV).toBe(60);
    expect(CORE_TAN_HALF_FOV).toBeCloseTo(Math.tan(30 * (Math.PI / 180)), 12);
    expect(HOME_CAMERA_STATE).toEqual({ theta: -0.6, phi: 1.22, radius: 9.4, vTheta: 0, vPhi: 0 });
    expect(goldenHtml).toContain("theta: -0.6, phi: 1.22, radius: 9.4");
  });

  it("renderer defaults are donor params", () => {
    expect(DEFAULT_VISUAL_PARAMS).toEqual({ mass: 0.72, spin: 0.65, temp: 0.42, bloom: 1.0 });
    expect(goldenHtml).toContain("mass: 0.72, spin: 0.65, temp: 0.42, bloom: 1.0");
  });

  it("golden reference itself is untouched (still the demo page)", () => {
    expect(goldenHtml).toContain("GARGANTUA");
    expect(goldenHtml).toContain("EVENT CONTROL");
  });
});
