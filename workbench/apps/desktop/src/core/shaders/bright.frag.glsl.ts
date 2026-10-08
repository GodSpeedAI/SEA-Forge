// Re-export shim: canonical donor source lives in `./post.frag.glsl.ts`
// (verbatim `BRIGHT_FRAG`). This alias exists so the pipeline can import the
// task-specified module path `shaders/bright.frag.glsl` directly.
export { BRIGHT_FRAG } from "./post.frag.glsl";
