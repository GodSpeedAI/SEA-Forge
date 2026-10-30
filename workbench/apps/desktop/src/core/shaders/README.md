# Core Shaders

## Overview
GLSL shader sources for the CORE renderer as exported TypeScript string constants, extracted verbatim from the Gargantua donor (see `/home/sprime01/projects/sea-rs/workbench/apps/desktop/src/core/PROVENANCE.md`). `post.frag.glsl.ts` holds the shared fragment shaders; sibling files are named shims re-exporting from it, and `core.frag.glsl.ts` holds the scene fragment template.

## Interface
- `QUAD_VERT`, `QUAD_VERT3` — fullscreen-quad vertex shaders (2D and 3D variants) [quad.vert.glsl.ts:QUAD_VERT]
- `NOISE_GLSL` — noise GLSL chunk spliced into the scene shader [noise.glsl.ts:NOISE_GLSL]
- `SCENE_FRAG_TEMPLATE` and `buildSceneFrag()` — Kerr black-hole scene fragment template and assembler [core.frag.glsl.ts:buildSceneFrag]
- `BRIGHT_FRAG`, `BLUR_FRAG`, `STREAK_FRAG`, `FINAL_FRAG` — post-processing fragment shaders [post.frag.glsl.ts:BRIGHT_FRAG]
- `blur.frag.glsl.ts`, `bright.frag.glsl.ts`, `composite.frag.glsl.ts`, `streak.frag.glsl.ts` — re-export shims (e.g. `export { BLUR_FRAG } from "./post.frag.glsl"`) [blur.frag.glsl.ts:BLUR_FRAG]

## Invariants
- ["None"]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
