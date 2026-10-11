// Re-export shim: canonical donor source lives in `./post.frag.glsl.ts`
// (verbatim `FINAL_FRAG`). This alias exists so the pipeline can import the
// task-specified module path `shaders/composite.frag.glsl` directly.
export { FINAL_FRAG } from "./post.frag.glsl";
