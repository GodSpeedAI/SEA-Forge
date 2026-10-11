// Re-export shim: canonical donor source lives in `./post.frag.glsl.ts`
// (verbatim `BLUR_FRAG`). This alias exists so the pipeline can import the
// task-specified module path `shaders/blur.frag.glsl` directly.
export { BLUR_FRAG } from "./post.frag.glsl";
