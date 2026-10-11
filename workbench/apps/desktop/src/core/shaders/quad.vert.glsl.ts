// Provenance: extracted verbatim from the Gargantua donor single-file
// implementation. See `src/core/PROVENANCE.md` and the untouched golden
// reference at `public/reference/gargantua.html`.
// Original blocks: `QUAD_VERT` (GLSL1 fullscreen quad) and `QUAD_VERT3`
// (GLSL3 fullscreen quad). Copied exactly; only import/export wrapping added.

export const QUAD_VERT = /* glsl */`
varying vec2 vUv;
void main(){
  vUv = uv;
  gl_Position = vec4(position.xy, 0.0, 1.0);
}
`;

export const QUAD_VERT3 = /* glsl */`
out vec2 vUv;
void main(){
  vUv = uv;
  gl_Position = vec4(position.xy, 0.0, 1.0);
}
`;
