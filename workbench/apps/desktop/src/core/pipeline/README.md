# Core Render Pipeline

## Overview
WebGL2 post-processing pipeline for the CORE renderer, extracted from the Gargantua donor (see `/home/sprime01/projects/sea-rs/workbench/apps/desktop/src/core/PROVENANCE.md`). `RenderPipeline` orchestrates a scene pass, bright pass, bloom (blur), streak, and composite pass over shared render targets. All passes are built as fullscreen-quad stages around the `RenderPass` contract.

## Interface
- `RenderPass` — common pass contract (`setEnabled`, `render`, `dispose`) [RenderPass.ts:RenderPass]
- `createQuadStage()` / `makePass()` — fullscreen quad staging and pass construction helpers [RenderPass.ts:createQuadStage]
- `CoreFrameParams` and `RenderPipeline` — per-frame parameters and the pass orchestrator [RenderPipeline.ts:RenderPipeline]
- `MAX_DEVICE_PIXEL_RATIO`, `CoreRenderTargets`, `makeRT`, `allocateTargets`, `RenderTargets` — render-target allocation and disposal [RenderTargets.ts:RenderTargets]
- `ScenePassHandles`, `createScenePass` — scene rendering pass [ScenePass.ts:createScenePass]
- `BRIGHT_THRESHOLD = 0.55`, `createBrightPass` — bright-pass extraction [BrightPass.ts:createBrightPass]
- `BLUR_DIR_SCALE = 1.6`, `BloomPass`, `createBloomPass` — gaussian bloom [BloomPass.ts:createBloomPass]
- `STREAK_STRETCH_FIRST = 2.6`, `STREAK_STRETCH_SECOND = 5.2`, `StreakPass`, `createStreakPass` — anamorphic streak [StreakPass.ts:createStreakPass]
- `CompositePass`, `createCompositePass` — final ACES-style composite [CompositePass.ts:createCompositePass]

## Invariants
- ["None"]

## Side Effects
- Allocates and resizes `THREE.WebGLRenderTarget` resources on the GPU; `disposeAll`/`RenderTargets.dispose` release them [RenderTargets.ts:allocateTargets]
- Each pass issues WebGL draw calls via three.js every frame [RenderPipeline.ts:RenderPipeline]

## Verification
- Run: `bun run test` (from workbench/)
