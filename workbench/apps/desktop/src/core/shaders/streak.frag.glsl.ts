// Re-export shim: canonical donor source lives in `./post.frag.glsl.ts`
// (verbatim `STREAK_FRAG`). This alias exists so the pipeline can import the
// task-specified module path `shaders/streak.frag.glsl` directly.
export { STREAK_FRAG } from "./post.frag.glsl";
