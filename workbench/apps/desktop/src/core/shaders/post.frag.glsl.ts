// Provenance: extracted verbatim from the Gargantua donor single-file
// implementation. See `src/core/PROVENANCE.md` and the untouched golden
// reference at `public/reference/gargantua.html`.
// Original blocks: `BRIGHT_FRAG`, `BLUR_FRAG`, `STREAK_FRAG`, `FINAL_FRAG`
// (passes 2–5). Copied exactly; only import/export wrapping added.

export const BRIGHT_FRAG = /* glsl */`
precision highp float;
varying vec2 vUv;
uniform sampler2D tSrc;
uniform float uThreshold;
void main(){
  vec3 c = texture2D(tSrc, vUv).rgb;
  float l = max(c.r, max(c.g, c.b));
  float w = smoothstep(uThreshold, uThreshold + 0.7, l);
  gl_FragColor = vec4(c * w, 1.0);
}
`;

export const BLUR_FRAG = /* glsl */`
precision highp float;
varying vec2 vUv;
uniform sampler2D tSrc;
uniform vec2 uDir;         // (texel.x,0) or (0,texel.y), pre-scaled
void main(){
  vec3 c = texture2D(tSrc, vUv).rgb * 0.227027;
  vec2 o1 = uDir * 1.3846153846;
  vec2 o2 = uDir * 3.2307692308;
  c += texture2D(tSrc, vUv + o1).rgb * 0.3162162162;
  c += texture2D(tSrc, vUv - o1).rgb * 0.3162162162;
  c += texture2D(tSrc, vUv + o2).rgb * 0.0702702703;
  c += texture2D(tSrc, vUv - o2).rgb * 0.0702702703;
  gl_FragColor = vec4(c, 1.0);
}
`;

export const STREAK_FRAG = /* glsl */`
precision highp float;
varying vec2 vUv;
uniform sampler2D tSrc;
uniform float uTexelX;
uniform float uStretch;
void main(){
  vec3 acc = vec3(0.0);
  float wsum = 0.0;
  for(int i = -14; i <= 14; i++){
    float fi = float(i);
    float w = exp(-fi * fi * 0.011);
    acc += texture2D(tSrc, vUv + vec2(fi * uTexelX * uStretch, 0.0)).rgb * w;
    wsum += w;
  }
  gl_FragColor = vec4(acc / wsum, 1.0);
}
`;

export const FINAL_FRAG = /* glsl */`
precision highp float;
varying vec2 vUv;
uniform sampler2D tScene;
uniform sampler2D tBloomA;    // tight bloom
uniform sampler2D tBloomB;    // wide bloom
uniform sampler2D tStreak;
uniform float uBloom;
uniform float uTime;
uniform vec2  uBHScreen;      // black hole centre, uv
uniform float uLensR;         // apparent lensing radius, uv units
uniform vec2  uResolution;

vec3 aces(vec3 x){
  const float a = 2.51, b = 0.03, c = 2.43, d = 0.59, e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}
float hash21(vec2 p){
  p = fract(p * vec2(123.34, 456.21));
  p += dot(p, p + 45.32);
  return fract(p.x * p.y);
}

void main(){
  vec2 uv = vUv;
  float aspect = uResolution.x / uResolution.y;

  // --- chromatic aberration : strong near the gravity well ---
  vec2 toBH = (uv - uBHScreen) * vec2(aspect, 1.0);
  float dBH = length(toBH);
  float grav = smoothstep(uLensR * 3.2, uLensR * 0.8, dBH);      // near-hole zone
  vec2 edge = uv - 0.5;
  float caAmt = 0.0012 + 0.0035 * dot(edge, edge) + 0.0042 * grav;
  vec2 caDir = normalize(dBH > 1e-4 ? toBH : edge + 1e-4) / vec2(aspect, 1.0);

  vec3 scene;
  scene.r = texture2D(tScene, uv + caDir * caAmt).r;
  scene.g = texture2D(tScene, uv).g;
  scene.b = texture2D(tScene, uv - caDir * caAmt).b;

  // --- bloom + anamorphic streak ---
  vec3 bloom = texture2D(tBloomA, uv).rgb * 0.60
             + texture2D(tBloomB, uv).rgb * 0.80;
  vec3 streak = texture2D(tStreak, uv).rgb;
  vec3 col = scene
           + bloom * uBloom
           + streak * vec3(0.55, 0.72, 1.0) * uBloom * 0.35;

  // --- lens flare ghosts, mirrored through screen centre ---
  vec2 fvec = 0.5 - uv;
  float ghostFade = 0.0;
  vec3 ghosts = vec3(0.0);
  for(int i = 1; i <= 3; i++){
    vec2 guv = uv + fvec * (0.55 * float(i));
    float fall = pow(max(0.0, 1.0 - length(guv - 0.5) * 1.6), 2.0);
    ghosts += texture2D(tBloomB, guv).rgb * fall * 0.035;
  }
  ghosts *= vec3(0.65, 0.8, 1.0);
  col += ghosts * uBloom;

  // --- tonemap, vignette, grain ---
  col = aces(col * 1.0);
  col = pow(col, vec3(0.92));                       // slight lift
  float vig = 1.0 - 0.42 * pow(length(edge) * 1.32, 2.4);
  col *= clamp(vig, 0.0, 1.0);
  float grain = (hash21(uv * uResolution.xy * 0.5 + fract(uTime) * 731.0) - 0.5) * 0.015;
  col += grain;

  gl_FragColor = vec4(col, 1.0);
}
`;
