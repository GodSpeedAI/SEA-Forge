// Provenance: extracted verbatim from the Gargantua donor single-file
// implementation. See `src/core/PROVENANCE.md` and the untouched golden
// reference at `public/reference/gargantua.html`.
// Original block: `SCENE_FRAG` (PASS 1 — the black hole itself: geodesic
// raymarch, accretion disk, lensed starfield). Copied exactly, including the
// `${NOISE}` injection token. Only import/export wrapping added.
//
// In the donor, the noise chunk was spliced at pass-construction time via
// `SCENE_FRAG.replace('${NOISE}', NOISE_GLSL)` where the source spelled the
// token as `${'${NOISE}'}` to survive the outer template literal. Here the
// token is a plain `${NOISE}` substring and `buildSceneFrag` performs the
// same replacement.

import { NOISE_GLSL } from "./noise.glsl";

export const SCENE_FRAG_TEMPLATE = /* glsl */`
precision highp float;
in vec2 vUv;
out vec4 fragColor;
uniform vec2  uResolution;
uniform float uTime;
uniform vec3  uCamPos;
uniform mat3  uCamBasis;      // columns: right, up, forward
uniform float uTanHalfFov;
uniform float uRs;            // schwarzschild radius (scene units)
uniform float uSpin;          // 0 .. 0.98
uniform float uTemp;          // 0 .. 1  (cool red -> blue-hot)
uniform int   uSteps;

${"${NOISE}"}

#define ESCAPE_R  34.0

// blackbody-ish ramp. t: 0 cool -> 1 blue-hot, with local modulation
vec3 plasmaColor(float t){
  t = clamp(t, 0.0, 1.6);
  vec3 c1 = vec3(1.00, 0.22, 0.03);   // deep red
  vec3 c2 = vec3(1.00, 0.52, 0.12);   // orange
  vec3 c3 = vec3(1.00, 0.83, 0.55);   // warm white
  vec3 c4 = vec3(0.95, 0.96, 1.00);   // white
  vec3 c5 = vec3(0.62, 0.78, 1.00);   // blue-white
  vec3 c = mix(c1, c2, smoothstep(0.00, 0.30, t));
  c = mix(c, c3, smoothstep(0.28, 0.58, t));
  c = mix(c, c4, smoothstep(0.55, 0.85, t));
  c = mix(c, c5, smoothstep(0.82, 1.25, t));
  return c;
}

// --- procedural starfield + faint nebula, sampled by escaped ray dir ---
float hash13(vec3 p){
  p = fract(p * 0.1031);
  p += dot(p, p.yzx + 33.33);
  return fract((p.x + p.y) * p.z);
}
vec3 hash33(vec3 p){
  p = fract(p * vec3(0.1031, 0.1030, 0.0973));
  p += dot(p, p.yxz + 33.33);
  return fract((p.xxy + p.yxx) * p.zyx);
}
vec3 starLayer(vec3 rd, float scale, float bright){
  vec3 p = rd * scale;
  vec3 id = floor(p);
  vec3 col = vec3(0.0);
  vec3 rnd = hash33(id);
  vec3 sp = id + 0.15 + 0.7 * rnd;
  float d = length(p - sp);
  float mag = pow(hash13(id + 7.1), 22.0);           // few bright, many dim
  float star = smoothstep(0.13, 0.0, d) * mag * bright;
  // subtle temperature variation between stars
  vec3 tint = mix(vec3(0.72, 0.82, 1.0), vec3(1.0, 0.85, 0.68), rnd.x);
  return star * tint;
}
vec3 background(vec3 rd){
  vec3 col = vec3(0.0);
  col += starLayer(rd, 62.0, 14.0);
  col += starLayer(rd, 31.0, 26.0);
  col += starLayer(rd, 17.0, 38.0);
  // galactic dust band, tilted
  vec3 bn = normalize(vec3(0.22, 1.0, 0.31));
  float band = exp(-pow(dot(rd, bn) * 3.4, 2.0));
  float neb = fbm(rd * 3.1 + vec3(4.2)) * 0.5 + 0.5;
  float neb2 = fbm(rd * 7.3 - vec3(1.7)) * 0.5 + 0.5;
  col += band * (0.028 * vec3(0.62, 0.55, 0.72) * neb
               + 0.020 * vec3(0.82, 0.60, 0.44) * neb * neb2);
  // faint cold ambient nebulosity
  col += 0.0045 * vec3(0.35, 0.48, 0.72) * (fbm(rd * 2.1) * 0.5 + 0.5);
  return col;
}

// --- accretion disk volumetric sample ---
// returns rgb emission (pre-multiplied) in .rgb, absorption in .a
vec4 diskSample(vec3 p, vec3 rd, float rInner, float rOuter){
  float r = length(p.xz);
  if(r < rInner * 0.92 || r > rOuter) return vec4(0.0);

  float H = uRs * (0.032 + 0.085 * smoothstep(rInner, rOuter, r)); // flaring height
  float vert = exp(-(p.y * p.y) / (H * H));
  if(vert < 0.004) return vec4(0.0);

  // Keplerian differential rotation (prograde with spin)
  float omega = 0.55 / (pow(r / uRs, 1.5) * 0.35 + 0.05);
  float ang = uTime * omega;
  float ca = cos(ang), sa = sin(ang);
  vec2 q = mat2(ca, -sa, sa, ca) * p.xz;   // rewind rotation -> shearing noise

  // turbulence: stretched azimuthally for streaky gas lanes
  vec3 np = vec3(q.x, p.y * 5.0, q.y) * (2.4 / uRs);
  float turb = fbm3(np);
  float lanes = fbm3(vec3(np.xz * 0.5, uTime * 0.05).xzy);
  float density = vert
    * smoothstep(rInner * 0.92, rInner * 1.25, r)
    * pow(smoothstep(rOuter, rOuter * 0.42, r), 1.4)
    * (0.15 + 0.85 * clamp(turb * 0.85 + 0.5, 0.0, 1.0))
    * (0.28 + 0.72 * clamp(lanes * 1.1 + 0.5, 0.0, 1.0));
  if(density < 0.003) return vec4(0.0);

  // orbital velocity for doppler (beta in c units, capped)
  float beta = clamp(0.5 * sqrt(uRs / max(r, uRs * 1.05)), 0.0, 0.72);
  vec3 vdir = normalize(vec3(-p.z, 0.0, p.x));            // prograde tangent
  float dshift = 1.0 / (1.0 + beta * dot(vdir, rd));      // >1 approaching
  float boost = mix(1.0, min(dshift * dshift * dshift, 5.0), 0.65);

  // temperature: hot inside, cool outside; doppler-shifted; grav redshift
  float g = sqrt(clamp(1.0 - uRs / max(r, uRs * 1.02), 0.0, 1.0));
  float tRad = pow(rInner * 1.35 / r, 0.75);
  float t = (0.12 + 1.05 * uTemp) * tRad * dshift * (0.35 + 0.65 * g);

  vec3 emit = plasmaColor(t) * density * boost * (0.40 + 2.6 * tRad) * g;
  return vec4(emit, density);
}

void main(){
  vec2 ndc = vUv * 2.0 - 1.0;
  float aspect = uResolution.x / uResolution.y;
  vec3 rd = normalize(uCamBasis * vec3(ndc.x * aspect * uTanHalfFov,
                                       ndc.y * uTanHalfFov, 1.0));
  vec3 p = uCamPos;

  float rInner = uRs * (2.6 - 1.55 * uSpin);   // ISCO shrinks with spin
  float rOuter = uRs * 9.5;

  // conserved angular momentum^2 for photon (schwarzschild bend strength)
  vec3 hv = cross(p, rd);
  float h2 = dot(hv, hv);

  vec3 col = vec3(0.0);
  float trans = 1.0;
  bool captured = false;
  bool escaped = false;

  // uniform loop bound: keeps ANGLE/D3D from unrolling the march
  for(int i = 0; i < uSteps; i++){
    float r2 = dot(p, p);
    float r = sqrt(r2);

    if(r < uRs * 1.01){ captured = true; break; }
    if(r2 > ESCAPE_R * ESCAPE_R && dot(p, rd) > 0.0){ escaped = true; break; }

    // adaptive step: fine near hole & disk plane, coarse far away
    float dt = clamp(r * 0.085, 0.015, 0.42);
    float rxz = length(p.xz);
    if(abs(p.y) < uRs * 0.7 && rxz < rOuter * 1.25) dt = min(dt, 0.028 * uRs + 0.12 * abs(p.y));
    dt *= (0.7 + 0.6 * uRs);

    // geodesic bend:  a = -3/2 h^2 r_vec / r^5   (schwarzschild null ray)
    vec3 accel = -1.5 * h2 * uRs * p / (r2 * r2 * r);
    // frame dragging: tangential pull, ~1/r^3 like the Kerr metric
    accel += uSpin * uRs * uRs * 0.3 * cross(vec3(0.0, 1.0, 0.0), p) / (r2 * r2);

    rd = normalize(rd + accel * dt);
    p += rd * dt;

    // volumetric disk accumulation
    vec4 dsk = diskSample(p, rd, rInner, rOuter);
    if(dsk.a > 0.0){
      col += trans * dsk.rgb * dt * 4.5;
      trans *= exp(-dsk.a * dt * 9.0);
      if(trans < 0.01){ captured = true; break; }   // fully occluded
    }
  }

  if(escaped){
    col += trans * background(normalize(rd));
  }
  // rays that ran out of steps near the hole -> shadow (correct look)

  fragColor = vec4(col, 1.0);
}
`;

/** Donor splice, preserved: scene template with the noise chunk injected. */
export function buildSceneFrag(): string {
  return SCENE_FRAG_TEMPLATE.replace("${NOISE}", NOISE_GLSL);
}
