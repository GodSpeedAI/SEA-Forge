import * as THREE from 'three'
import type { Theme } from '../model/types'

// The only WebGL in the app: one canvas, one context. It draws the singularities (Core and
// any case acting as a center of gravity), their accretion particles, and a sparse dust field.
// It knows nothing about the world model; the scene runtime feeds it screen-space values
// derived from the same camera as the DOM layer every frame.

export interface Singularity {
  /** CSS px, top-left origin */
  x: number
  y: number
  /** void radius in CSS px */
  r: number
  /** 0 = face-on swirl (Home), ~1.3 = edge-on lensed disk (focused case) */
  incl: number
  /** 0..1 */
  strength: number
}

export interface DustField {
  x: number
  y: number
  /** CSS px per mock px */
  scale: number
  opacity: number
}

const MAX_S = 3

const quadVert = /* glsl */ `
out vec2 vUv;
void main() { vUv = uv; gl_Position = vec4(position.xy, 0.0, 1.0); }
`

const quadFrag = /* glsl */ `
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform vec2 uRes;          // CSS px
uniform float uTime;
uniform float uDark;
uniform float uWake;
uniform vec4 uS[${MAX_S}];   // x, y (px, top-left), r, incl
uniform float uStrength[${MAX_S}];

float hash(vec3 p) { p = fract(p * 0.3183099 + 0.1); p *= 17.0; return fract(p.x * p.y * p.z * (p.x + p.y + p.z)); }
float noise(vec3 x) {
  vec3 i = floor(x); vec3 f = fract(x); f = f * f * (3.0 - 2.0 * f);
  return mix(mix(mix(hash(i + vec3(0,0,0)), hash(i + vec3(1,0,0)), f.x),
                 mix(hash(i + vec3(0,1,0)), hash(i + vec3(1,1,0)), f.x), f.y),
             mix(mix(hash(i + vec3(0,0,1)), hash(i + vec3(1,0,1)), f.x),
                 mix(hash(i + vec3(0,1,1)), hash(i + vec3(1,1,1)), f.x), f.y), f.z);
}
float fbm(vec3 p) { float a = 0.5, s = 0.0; for (int i = 0; i < 4; i++) { s += a * noise(p); p *= 2.03; a *= 0.5; } return s; }

// Streak density of the accretion disk at disk-plane polar coords.
// Returns (density, highlight). Streaks follow a log spiral: noise varies fast across the
// spiral angle and slowly along it, so every feature is a long curved hair.
uniform float uWind;
vec3 streaks(float r, float th, float t, float seed) {
  float lr = log(r);
  float spin = t * 0.22 / pow(r, 1.5);
  // Tight winding: streaks run nearly tangential, spiralling in.
  float u0 = th + spin + uWind * lr;
  // Domain warp breaks the lattice so fine streaks never read as a regular comb.
  float warp = noise(vec3(cos(th) * 2.3, sin(th) * 2.3, lr * 1.0 + seed)) - 0.5;
  float u = u0 + warp * 0.14;
  vec2 cs = vec2(cos(u), sin(u));
  // Two broad spiral arms, softened by noise.
  float armWave = 0.5 + 0.5 * cos(2.0 * u0 + 1.1 + (noise(vec3(cs * 1.7, lr * 0.6 + seed)) - 0.5) * 2.4);
  float arms = smoothstep(0.25, 0.95, armWave);
  float mid = noise(vec3(cs * 7.3, lr * 0.35 + seed * 2.3));
  float fine = noise(vec3(cs * 19.7, lr * 0.5 + seed * 3.1));
  float hair = noise(vec3(cs * 53.1, lr * 0.6 + seed * 1.7));
  float hair2 = noise(vec3(cs * 121.3, lr * 0.8 + seed * 0.7));
  float body = exp(-(r - 1.0) / 0.55) * (0.62 + 0.38 * hair) * (0.75 + 0.25 * fine);
  float strand = (0.3 + 0.7 * mid) * (0.45 + 0.55 * fine) * (0.4 + 0.6 * hair);
  // smoky fill between the arms, thinning outward
  float smoke = 0.3 * mid * (1.0 - smoothstep(1.2, 3.2, r));
  float d = max(max(body, arms * strand), smoke) + 0.1 * smoothstep(0.65, 0.95, hair2) * (0.4 + 0.6 * arms);
  float hi = smoothstep(0.45, 0.85, fine * 0.45 + hair * 0.35 + mid * 0.2);
  // Soft density (no hairs) for luminous dark-mode filaments.
  float soft = max(exp(-(r - 1.0) / 0.5) * 0.75, arms * (0.3 + 0.7 * mid) * (0.6 + 0.4 * fine));
  return vec3(d, hi, soft);
}


// Edge-on centers: a small ray march with Newtonian light bending (units: r_s = 1).
// Rays that fall in are the void; rays crossing the disk plane sample the streak disk.
// This produces the lensed far-side arc over the void and the near disk crossing in front.
// Lensed disk texture: symmetric concentric streaks (no spiral arms), so the eye stays balanced.
vec2 ringStreaks(float rd, float phi, float t, float seed) {
  float a = phi + t * 0.25 / pow(rd, 1.5);
  vec2 cs = vec2(cos(a), sin(a));
  float n1 = noise(vec3(cs * 1.4, rd * 11.0 + seed));
  float n2 = noise(vec3(cs * 2.2, rd * 31.0 + seed * 2.0));
  float n3 = noise(vec3(cs * 3.0, rd * 70.0 + seed * 3.0));
  float d = exp(-(rd - 1.0) / 0.95) * (0.45 + 0.55 * n1) * (0.6 + 0.4 * n2) + 0.12 * smoothstep(0.6, 0.95, n3);
  float hi = smoothstep(0.5, 0.88, n1 * 0.5 + n2 * 0.5);
  return vec2(d, hi);
}

vec4 diskSample(float rr, float phi, float t, float seed) {
  if (rr < 2.5) return vec4(0.0);
  // Palette and density in units of the visible void radius (~3.43 r_s after calibration),
  // so "hot" means "close to the edge you can see".
  float rv = rr / 3.43;
  if (rv > 3.1) return vec4(0.0);
  vec2 sd = ringStreaks(max(rv, 1.0), phi, t, seed);
  float prof = 1.0 - smoothstep(1.4, 3.0, rv);
  float d = clamp(sd.x * prof * 2.2, 0.0, 1.0);
  float heat = 1.0 - smoothstep(0.9, 2.4, rv);
  d = max(d, exp(-(rv - 0.9) / 1.1) * prof * (0.5 + 0.5 * sd.y));
  vec3 c;
  float a;
  if (uDark < 0.5) {
    vec3 peach = vec3(0.98, 0.74, 0.52);
    vec3 cream = vec3(1.0, 0.95, 0.89);
    vec3 bronze = vec3(0.55, 0.43, 0.36);
    vec3 smoke = vec3(0.46, 0.43, 0.42);
    c = mix(peach, cream, smoothstep(0.35, 0.8, sd.y + heat * 0.35));
    c = mix(c, bronze, smoothstep(1.9, 2.6, rv) * (1.0 - sd.y * 0.5));
    c = mix(c, smoke, smoothstep(2.5, 3.3, rv));
    a = smoothstep(0.02, 0.3, d) * mix(0.75, 1.0, heat) * (0.45 + 0.7 * sd.y);
  } else {
    c = mix(vec3(0.08, 0.32, 1.0), vec3(0.55, 0.82, 1.0), heat * 0.7 + sd.y * 0.35) * (1.1 + 1.3 * heat);
    a = smoothstep(0.02, 0.3, d) * (0.65 + 0.35 * heat);
  }
  return vec4(c * a, a);
}

vec4 lensed(vec2 p, float incl, float t, float seed) {
  float D = 16.0;
  vec3 C = D * vec3(0.0, cos(incl), -sin(incl));
  vec3 fw = normalize(-C);
  vec3 rt = vec3(1.0, 0.0, 0.0);
  vec3 up = cross(fw, rt);
  float k = 2.6 / D * 1.32;                   // calibrated so the captured region lands at |p| = 1
  vec3 dir = normalize(fw + (p.x * rt - p.y * up) * k);
  vec3 pos = C;
  vec3 h = cross(pos, dir);
  float h2 = dot(h, h);
  vec4 acc = vec4(0.0);
  bool fell = false;
  for (int i = 0; i < 72; i++) {
    float r2 = dot(pos, pos);
    float rl = sqrt(r2);
    float dt = clamp(0.075 * rl, 0.025, 1.4);
    vec3 accel = -1.5 * h2 * pos / (r2 * r2 * rl);
    vec3 npos = pos + dir * dt;
    dir = normalize(dir + accel * dt);
    if (pos.y * npos.y < 0.0) {
      float f = pos.y / (pos.y - npos.y);
      vec3 hit = mix(pos, npos, f);
      vec4 sm = diskSample(length(hit.xz), atan(hit.z, hit.x), t, seed);
      acc += (1.0 - acc.a) * sm;
    }
    pos = npos;
    if (dot(pos, pos) < 1.0) { fell = true; break; }
    if (rl > D * 1.3 && dot(pos, dir) > 0.0) break;
    if (acc.a > 0.98) break;
  }
  // Rays still circling the hole when the step budget runs out are treated as captured.
  if (!fell && dot(pos, pos) < 16.0) fell = true;
  // The void stays clean (as in the mocks): captured rays are pure black.
  if (fell) acc = vec4(0.0, 0.0, 0.0, 1.0);
  return acc;
}

vec4 singularity(vec2 px, vec4 s, float strength, float seed) {
  vec2 p = (px - s.xy) / s.z;             // units of void radius, y down
  float R = length(p);
  if (R > 4.6 || strength <= 0.0) return vec4(0.0);
  float incl = s.w;
  float edge = smoothstep(0.5, 1.2, incl);  // 0 face-on (Home) .. 1 edge-on (focused case)
  float ci = max(cos(incl), 0.12);
  // Disk plane coordinates (tilted about the x axis).
  // Edge-on, lensing pinches the disk toward points at its far ends (the mock's "eye").
  float pinch = mix(1.0, max(0.22, 1.12 - 0.3 * abs(p.x)), smoothstep(0.5, 1.2, incl));
  vec2 q = vec2(p.x, p.y / (ci * pinch));
  float r = length(q);
  float th = atan(q.y, q.x);
  float t = uTime * (0.55 + 0.45 * uWake);
  float ang = atan(p.y, p.x);

  if (edge > 0.5) {
    // Inside the calibrated void it is simply black (avoids numerically escaping rays).
    if (R < 0.975) return vec4(0.0, 0.0, 0.0, 1.0) * strength;
    vec4 L = lensed(p, incl, t, seed);
    // luminous rim hugging the void, then a wide warm haze
    float rim = exp(-max(R - 1.0, 0.0) / 0.26) * step(1.0, R);
    vec3 rimC = uDark < 0.5 ? mix(vec3(0.99, 0.74, 0.52), vec3(1.0, 0.97, 0.93), exp(-max(R - 1.0, 0.0) / 0.05)) : vec3(0.55, 0.8, 1.0);
    vec4 rimL = vec4(rimC * rim * 0.95, rim * 0.95);
    L = rimL * (1.0 - L.a * 0.5) + L * (1.0 - rimL.a * 0.5);
    // The photon ring hugging the void is bright, never the grey of skimming rays.
    float hug = (1.0 - smoothstep(1.0, 1.12, R)) * step(0.985, R);
    L = mix(L, vec4(rimC, 1.0) * 0.97, hug * 0.9);
    float haze = exp(-R * 0.55) * 0.55 * (1.0 - L.a);
    vec3 hc = uDark < 0.5 ? vec3(0.99, 0.85, 0.78) : vec3(0.03, 0.09, 0.3);
    if (uDark < 0.5) L += vec4(hc * haze * 0.75, haze * 0.75);
    else L += vec4(hc * haze * 0.5, 0.0);
    return L * strength;
  }
  vec3 sd = streaks(max(r, 1.0), th, t, seed);
  // Face-on radial profile: dense near the void, wispy tail, lopsided.
  float inner = smoothstep(0.98, 1.1, r);
  float outer = 1.0 - smoothstep(1.4, mix(3.3, 3.6, edge), r);
  float lobe = mix(0.78 + 0.4 * cos(th * 2.0 + 1.57) + 0.1 * cos(th + 2.1), 1.0, edge);
  float prof = inner * outer * outer * lobe;
  float faceDisk = clamp(sd.x * prof * 1.35, 0.0, 1.0);

  // Edge-on needs no special geometry: the same swirl projected at high inclination flattens
  // into the concentric "eye" arcs of the mock; the smoky outer disk becomes the lids.
  float disk = faceDisk;
  float hi = sd.y * prof;
  // Inside the void's outline, the near half of the disk passes in front of it.
  float front = step(0.0, p.y) * step(R, 1.0) * smoothstep(1.0, 1.06, r) * edge;

  // Photon ring + inner glow (both views) and the lensed far side of the disk (edge-on).
  float ring = exp(-pow((R - 1.015) / mix(0.035, 0.028, edge), 2.0));
  float glow = exp(-max(R - 1.0, 0.0) / mix(0.2, 0.16, edge)) * step(1.0, R);
  float lensC = 1.13 + 0.035 * sin(ang * 2.0);
  float lensBand = exp(-pow((R - lensC) / 0.15, 2.0)) + 0.55 * exp(-pow((R - 1.38) / 0.14, 2.0)) * smoothstep(0.1, 0.8, -p.y / max(R, 0.001));
  float lensTex = 0.35 + 0.5 * noise(vec3(cos(ang) * 9.0, sin(ang) * 9.0, R * 4.0 + t * 0.2 + seed)) + 0.45 * smoothstep(0.45, 0.9, noise(vec3(cos(ang) * 38.0, sin(ang) * 38.0, R * 9.0 + seed)));
  float lens = lensBand * lensTex * edge * (0.65 + 0.35 * abs(sin(ang)));
  float haze = mix(exp(-R * 0.9) * 0.5, exp(-R * 0.55) * 0.62, edge);

  float voidA = 1.0 - smoothstep(0.975, 1.005, R);
  vec4 col;
  if (uDark < 0.5) {
    vec3 peach = vec3(0.98, 0.8, 0.62);
    vec3 cream = vec3(1.0, 0.95, 0.88);
    vec3 bronze = vec3(0.56, 0.42, 0.33);
    vec3 smoke = vec3(0.52, 0.47, 0.45);
    // near the void: white-hot cream streaks interleaved with peach
    // colour zones follow disk radius face-on, but screen distance edge-on (the eye glows near the void)
    float rc = mix(r, 1.0 + (R - 1.0) * 0.62, edge);
    vec3 innerC = mix(peach, cream, smoothstep(0.1, 0.5, sd.y + 0.6 * (1.0 - smoothstep(1.0, 1.5, rc))));
    innerC = mix(innerC, bronze, (1.0 - sd.y) * mix(0.22, 0.1, edge));
    vec3 faceC = mix(innerC, bronze, smoothstep(mix(1.2, 1.45, edge), mix(1.5, 1.95, edge), rc));
    faceC = mix(faceC, smoke, smoothstep(1.8, 2.6, rc));
    float faceA = smoothstep(0.02, 0.34, faceDisk) * (1.0 - 0.5 * smoothstep(1.8, 3.0, r));
    faceA *= clamp(0.35 + 1.1 * (1.0 - sd.y) * step(1.25, r) + sd.y * (1.0 - step(1.25, r)) + 0.3 * sd.x, 0.0, 1.0);
    faceC = mix(faceC, vec3(1.0, 0.95, 0.9), sd.y * smoothstep(1.2, 1.5, r) * (1.0 - smoothstep(1.9, 2.4, r)) * 0.7);
    // white-hot glints along the inner arms (mock 01's brightest streaks)
    float glint = sd.y * sd.y * (1.0 - smoothstep(1.15, 2.2, rc)) * smoothstep(0.35, 0.7, faceDisk);
    faceC = mix(faceC, vec3(1.0, 0.985, 0.96), clamp(glint * 1.4, 0.0, 0.95));
    vec3 c = faceC;
    float a = faceA;
    // rim glow, lensed ring and front arc: luminous cream fading to peach
    vec3 glowC = mix(vec3(0.99, 0.76, 0.52), vec3(1.0, 0.98, 0.94), clamp(ring + lens * 1.2, 0.0, 1.0));
    float g = clamp(glow * 0.9 + ring + lens * 1.5, 0.0, 1.0);
    c = mix(c, glowC, g * 0.85);
    a = max(a, g * 0.92);
    // warm haze behind everything
    vec3 hc = mix(vec3(0.98, 0.86, 0.78), vec3(0.99, 0.84, 0.78), edge);
    float ha = haze * mix(0.28, 0.5, edge) * (1.0 - voidA);
    c = mix(hc, c, clamp(a / max(a + ha, 0.001), 0.0, 1.0));
    a = clamp(a + ha * (1.0 - a), 0.0, 1.0);
    // the void, with a faint warm inner rim and the front arc crossing it
    vec3 vc = mix(vec3(0.0), vec3(0.16, 0.08, 0.04), smoothstep(0.86, 1.0, R));
    float inVoid = 1.0 - smoothstep(0.975, 1.005, R);
    // near disk in front of the lower void
    vc = mix(vc, faceC, front * faceA * 0.25 * smoothstep(0.7, 0.98, R));
    c = mix(c, vc, inVoid);
    a = mix(a, 1.0, inVoid);
    col = vec4(c * a, a);
  } else {
    // Dark: luminous blue-fire vortex, additive on near-black.
    float heat = 1.0 - smoothstep(1.0, 2.3, r);
    vec3 blue = mix(vec3(0.05, 0.2, 0.9), vec3(0.3, 0.62, 1.0), heat);
    // Filaments, not a solid ring: streak contrast carries the brightness (mock 13).
    vec3 lite = vec3(0.42, 0.72, 1.0);
    float softD = clamp(sd.z * prof * 1.4, 0.0, 1.0);
    vec3 e = mix(blue, lite, clamp(heat * 0.5 + sd.y * 0.5, 0.0, 1.0)) * (pow(softD, 1.25) * 1.35 + pow(disk, 1.6) * 0.5);
    e += vec3(0.35, 0.62, 1.0) * (glow * 0.25 + ring * 0.7 + lens * 1.1);
    e += vec3(0.03, 0.08, 0.3) * haze * 0.6;
    float inVoid = 1.0 - smoothstep(0.975, 1.005, R);
    e = mix(e, blue * disk * front * 1.4, inVoid);
    float a = clamp(max(max(e.r, e.g), e.b), 0.0, 1.0);
    a = max(a, inVoid);
    col = vec4(e, a);
  }
  return col * strength;
}

void main() {
  vec2 px = vec2(vUv.x, 1.0 - vUv.y) * uRes;
  vec4 acc = vec4(0.0);
  for (int i = 0; i < ${MAX_S}; i++) {
    vec4 c = singularity(px, uS[i], uStrength[i], float(i) * 7.31);
    acc = c + acc * (1.0 - c.a);  // premultiplied "over"
  }
  outColor = acc;
}
`

const pointVert = /* glsl */ `
precision highp float;
attribute float aRadius;
attribute float aAngle;
attribute float aSize;
attribute float aKind;
attribute float aLift;
uniform vec2 uRes;
uniform vec4 uS;          // primary singularity x, y, r, incl
uniform float uStrength;
uniform float uTime;
uniform float uPixelRatio;
varying float vKind;
varying float vAlpha;
void main() {
  // Rotate with the shader's arm pattern (which moves toward -theta) so debris stays on the arms.
  float ang = aAngle - uTime * 0.12 / pow(aRadius, 1.5);
  vec2 d = vec2(cos(ang), sin(ang)) * aRadius;
  float ci = max(cos(uS.w), 0.12);
  // edge-on: particles hug the disk plane but keep a little vertical scatter
  vec2 p = vec2(d.x, d.y * ci + aLift * mix(0.05, 0.4, ci));
  vec2 px = uS.xy + p * uS.z;
  // hide particles behind the void
  float behind = step(length(p), 1.02);
  vAlpha = uStrength * (1.0 - behind) * smoothstep(3.9, 2.4, aRadius + 0.0);
  vKind = aKind;
  vec2 clip = vec2(px.x / uRes.x * 2.0 - 1.0, 1.0 - px.y / uRes.y * 2.0);
  gl_Position = vec4(clip, 0.0, 1.0);
  gl_PointSize = aSize * uPixelRatio * clamp(uS.z / 90.0, 0.55, 1.6);
}
`

const fieldVert = /* glsl */ `
precision highp float;
attribute vec2 aOffset;   // mock px around the center
attribute float aSize;
attribute float aKind;
attribute float aDepth;
uniform vec2 uRes;
uniform vec3 uField;      // x, y, scale
uniform float uOpacity;
uniform float uPixelRatio;
uniform float uTime;
varying float vKind;
varying float vAlpha;
void main() {
  vec2 drift = vec2(sin(uTime * 0.05 + aDepth * 9.0), cos(uTime * 0.04 + aDepth * 7.0)) * 3.0;
  vec2 px = uField.xy + (aOffset + drift) * uField.z * (0.85 + 0.3 * aDepth);
  vKind = aKind;
  vAlpha = uOpacity;
  gl_Position = vec4(px.x / uRes.x * 2.0 - 1.0, 1.0 - px.y / uRes.y * 2.0, 0.0, 1.0);
  gl_PointSize = aSize * uPixelRatio * clamp(uField.z, 0.6, 1.4);
}
`

const pointFrag = /* glsl */ `
precision highp float;
uniform float uDark;
varying float vKind;
varying float vAlpha;
void main() {
  vec2 c = gl_PointCoord - 0.5;
  float d = length(c);
  float a = smoothstep(0.5, 0.2, d) * vAlpha;
  if (a <= 0.001) discard;
  vec3 col;
  if (uDark < 0.5) {
    col = vKind < 0.5 ? vec3(0.13, 0.10, 0.09) : (vKind < 1.5 ? vec3(0.55, 0.27, 0.12) : vec3(0.95, 0.62, 0.36));
    a *= vKind < 0.5 ? 0.85 : 0.9;
  } else {
    col = vKind < 0.5 ? vec3(0.55, 0.75, 1.0) : (vKind < 1.5 ? vec3(0.3, 0.6, 1.0) : vec3(0.85, 0.95, 1.0));
    a = min(1.0, a * 1.35);
  }
  gl_FragColor = vec4(col * a, a);
}
`

function rng(seed: number) {
  let s = seed >>> 0
  return () => {
    s = (s + 0x6d2b79f5) >>> 0
    let t = s
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

export class CoreCanvas {
  private renderer: THREE.WebGLRenderer
  private scene = new THREE.Scene()
  private camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1)
  private quad: THREE.ShaderMaterial
  private disk: THREE.ShaderMaterial
  private field: THREE.ShaderMaterial
  private w = 1
  private h = 1

  constructor(canvas: HTMLCanvasElement) {
    this.renderer = new THREE.WebGLRenderer({ canvas, alpha: true, premultipliedAlpha: true, antialias: false })
    this.renderer.setClearColor(0x000000, 0)
    const blend = {
      transparent: true,
      depthTest: false,
      depthWrite: false,
      blending: THREE.CustomBlending,
      blendSrc: THREE.OneFactor,
      blendDst: THREE.OneMinusSrcAlphaFactor,
    } as const

    const sUniform = Array.from({ length: MAX_S }, () => new THREE.Vector4())
    this.quad = new THREE.ShaderMaterial({
      glslVersion: THREE.GLSL3,
      vertexShader: quadVert,
      fragmentShader: quadFrag,
      uniforms: {
        uRes: { value: new THREE.Vector2(1, 1) },
        uTime: { value: 0 },
        uDark: { value: 0 },
        uWake: { value: 0 },
        uWind: { value: Number(new URLSearchParams(location.search).get('wind') ?? -2.6) },
        uS: { value: sUniform },
        uStrength: { value: new Array(MAX_S).fill(0) },
      },
      ...blend,
    })
    const quadMesh = new THREE.Mesh(new THREE.PlaneGeometry(2, 2), this.quad)
    quadMesh.frustumCulled = false
    quadMesh.renderOrder = 1

    // Field dust first (behind), then the disk particles on top of the swirl.
    this.field = new THREE.ShaderMaterial({
      vertexShader: fieldVert,
      fragmentShader: pointFrag,
      uniforms: {
        uRes: { value: new THREE.Vector2(1, 1) },
        uField: { value: new THREE.Vector3() },
        uOpacity: { value: 0 },
        uPixelRatio: { value: 1 },
        uTime: { value: 0 },
        uDark: { value: 0 },
      },
      ...blend,
    })
    const fieldPts = new THREE.Points(this.fieldGeometry(), this.field)
    fieldPts.frustumCulled = false
    fieldPts.renderOrder = 0

    this.disk = new THREE.ShaderMaterial({
      vertexShader: pointVert,
      fragmentShader: pointFrag,
      uniforms: {
        uRes: { value: new THREE.Vector2(1, 1) },
        uS: { value: new THREE.Vector4() },
        uStrength: { value: 0 },
        uTime: { value: 0 },
        uPixelRatio: { value: 1 },
        uDark: { value: 0 },
      },
      ...blend,
    })
    const diskPts = new THREE.Points(this.diskGeometry(), this.disk)
    diskPts.frustumCulled = false
    diskPts.renderOrder = 2

    this.scene.add(fieldPts, quadMesh, diskPts)
  }

  private diskGeometry() {
    const n = 1100
    const r = rng(7)
    const radius = new Float32Array(n)
    const angle = new Float32Array(n)
    const size = new Float32Array(n)
    const kind = new Float32Array(n)
    const lift = new Float32Array(n)
    for (let i = 0; i < n; i++) {
      // most particles in the swirl, a sparse halo of debris further out
      const far = r() < 0.12
      radius[i] = far ? 2.2 + r() * 0.9 : 1.15 + Math.pow(r(), 1.2) * 1.55
      // Most debris follows the two spiral arms (same phase as the shader's arms), the rest is scattered.
      const onArm = r() < 0.45
      const gauss = (r() + r() + r() - 1.5) * 0.45
      angle[i] = onArm ? (r() < 0.5 ? 0 : Math.PI) - 0.55 + 2.6 * Math.log(radius[i]!) + gauss : r() * Math.PI * 2
      const big = r() < 0.05
      size[i] = big ? 2.8 + r() * 1.8 : 0.9 + r() * 1.4
      const k = r()
      kind[i] = k < 0.25 ? 0 : k < 0.8 ? 1 : 2
      lift[i] = (r() - 0.5) * 2
    }
    const g = new THREE.BufferGeometry()
    g.setAttribute('position', new THREE.BufferAttribute(new Float32Array(n * 3), 3))
    g.setAttribute('aRadius', new THREE.BufferAttribute(radius, 1))
    g.setAttribute('aAngle', new THREE.BufferAttribute(angle, 1))
    g.setAttribute('aSize', new THREE.BufferAttribute(size, 1))
    g.setAttribute('aKind', new THREE.BufferAttribute(kind, 1))
    g.setAttribute('aLift', new THREE.BufferAttribute(lift, 1))
    return g
  }

  private fieldGeometry() {
    const n = 170
    const r = rng(11)
    const off = new Float32Array(n * 2)
    const size = new Float32Array(n)
    const kind = new Float32Array(n)
    const depth = new Float32Array(n)
    for (let i = 0; i < n; i++) {
      // scattered across an ellipse around the center, avoiding the very middle
      const a = r() * Math.PI * 2
      const rr = 0.22 + Math.sqrt(r()) * 0.85
      off[i * 2] = Math.cos(a) * rr * 900
      off[i * 2 + 1] = Math.sin(a) * rr * 470
      const big = r() < 0.12
      size[i] = big ? 7 + r() * 6 : 2.2 + r() * 2.4
      kind[i] = 0
      depth[i] = r()
    }
    const g = new THREE.BufferGeometry()
    g.setAttribute('position', new THREE.BufferAttribute(new Float32Array(n * 3), 3))
    g.setAttribute('aOffset', new THREE.BufferAttribute(off, 2))
    g.setAttribute('aSize', new THREE.BufferAttribute(size, 1))
    g.setAttribute('aKind', new THREE.BufferAttribute(kind, 1))
    g.setAttribute('aDepth', new THREE.BufferAttribute(depth, 1))
    return g
  }

  private pr = 1
  private maxPr = 1
  private ema = 16
  private lastAdjust = 0
  /** ?quality=high pins full resolution (verification screenshots on software GL). */
  private fixed = new URLSearchParams(location.search).get('quality') === 'high'

  resize(w: number, h: number) {
    this.w = w
    this.h = h
    this.maxPr = Math.min(window.devicePixelRatio || 1, 2)
    this.pr = Math.min(this.pr, this.maxPr) || this.maxPr
    this.applyPixelRatio()
  }

  /** Dynamic resolution: trade canvas sharpness for frame rate on slow GPUs. */
  adapt(frameMs: number, now: number) {
    if (this.fixed) return
    this.ema = this.ema * 0.9 + frameMs * 0.1
    if (now - this.lastAdjust < 700) return
    let next = this.pr
    if (this.ema > 40 && this.pr > 0.45) next = Math.max(0.45, this.pr * 0.75)
    else if (this.ema < 20 && this.pr < this.maxPr) next = Math.min(this.maxPr, this.pr * 1.15)
    if (next !== this.pr) {
      this.pr = next
      this.lastAdjust = now
      this.applyPixelRatio()
    }
  }

  private applyPixelRatio() {
    const pr = this.pr
    const w = this.w
    const h = this.h
    this.renderer.setPixelRatio(pr)
    this.renderer.setSize(w, h, false)
    for (const m of [this.quad, this.disk, this.field]) {
      m.uniforms.uRes!.value.set(w, h)
      if (m.uniforms.uPixelRatio) m.uniforms.uPixelRatio.value = pr
    }
  }

  render(time: number, theme: Theme, wake: number, singularities: Singularity[], field: DustField | null) {
    const dark = theme === 'dark' ? 1 : 0
    const q = this.quad.uniforms
    q.uTime!.value = time
    q.uDark!.value = dark
    q.uWake!.value = wake
    const sv = q.uS!.value as THREE.Vector4[]
    const st = q.uStrength!.value as number[]
    for (let i = 0; i < MAX_S; i++) {
      const s = singularities[i]
      if (s && s.x > -s.r * 5 && s.x < this.w + s.r * 5 && s.y > -s.r * 5 && s.y < this.h + s.r * 5) {
        sv[i]!.set(s.x, s.y, Math.max(s.r, 0.5), s.incl)
        st[i] = s.strength
      } else st[i] = 0
    }
    const primary = singularities.reduce<Singularity | null>((best, s) => (!best || s.strength * s.r > best.strength * best.r ? s : best), null)
    const d = this.disk.uniforms
    d.uTime!.value = time
    d.uDark!.value = dark
    if (primary) {
      ;(d.uS!.value as THREE.Vector4).set(primary.x, primary.y, primary.r, primary.incl)
      d.uStrength!.value = primary.strength
    } else d.uStrength!.value = 0
    const f = this.field.uniforms
    f.uTime!.value = time
    f.uDark!.value = dark
    if (field) {
      ;(f.uField!.value as THREE.Vector3).set(field.x, field.y, field.scale)
      f.uOpacity!.value = field.opacity
    } else f.uOpacity!.value = 0
    this.renderer.render(this.scene, this.camera)
  }

  dispose() {
    this.renderer.dispose()
  }
}
