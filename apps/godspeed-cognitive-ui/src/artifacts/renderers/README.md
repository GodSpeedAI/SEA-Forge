# Artifact Renderers

## Overview
The nine source-level artifact renderers (diff, text, markdown, table, chart, json, graph, trace, timeline). Each is a separate lazy chunk so a session that never opens a graph never downloads the graph renderer (VAR-006, enforced by `assertRendererChunks` in `src/build/rendererChunkContract.ts` via the Vite build plugin). All implement the shared `SourceRendererProps` contract from `src/artifacts/registry.tsx` and run inside an error boundary (`RendererBoundary`), so a failing renderer is isolated and the source reference stays on screen (RECOV-004).

## Interface
- One renderer per payload model, each a default export taking `SourceRendererProps` (`model`, `payload`, `descriptor`, `query`, `onMatches`, `onFocusObject`, `onSetTime`):
  - `DiffRenderer` [DiffRenderer.tsx:DiffRenderer]
  - `TextRenderer` [TextRenderer.tsx:TextRenderer]
  - `MarkdownRenderer` [MarkdownRenderer.tsx:MarkdownRenderer]
  - `TableRenderer` [TableRenderer.tsx:TableRenderer]
  - `ChartRenderer` [ChartRenderer.tsx:ChartRenderer]
  - `JsonRenderer` [JsonRenderer.tsx:JsonRenderer]
  - `GraphRenderer` [GraphRenderer.tsx:GraphRenderer]
  - `TraceRenderer` [TraceRenderer.tsx:TraceRenderer]
  - `TimelineRenderer` [TimelineRenderer.tsx:TimelineRenderer]
- Shared helpers: `highlight(text, query)` — search-match segmentation used by renderers to highlight query matches and report counts [highlight.ts:highlight]; `layeredLayout(nodes, edges)` — layered graph layout for the graph renderer [graphLayout.ts:layeredLayout]

## Invariants
- Renderers never fetch: all content comes from the already-resolved `ArtifactPayload`/`ArtifactModel` handed to them [registry.tsx:SourceRendererProps]
- Every renderer runs inside `RendererBoundary`; a throw renders a bounded error while the source reference stays visible [registry.tsx:RendererBoundary]
- The build fails unless each of the nine renderer modules is emitted as exactly one distinct dynamic-entry chunk, dynamically imported by the registry chunk [build/rendererChunkContract.ts:assertRendererChunks]

## Side Effects
- ["None"]

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; renderer helpers: `src/artifacts/renderers/highlight.test.ts`, `graphLayout.test.ts`; chunk contract: `src/build/rendererChunkContract.test.ts`)
