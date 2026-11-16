# Artifacts

## Overview
Source-backed artifact handling: parsing the contract `ArtifactPayload` into typed renderer models, resolving payloads through the port with per-ref caching and attached failures, and the lazy renderer registry with its error boundary. Parsing is light and lives in the main bundle; the source-level renderers are lazy chunks (see `./renderers/README.md`).

## Interface
- `parseArtifact(p: ArtifactPayload): ParseResult` — contract payload (`content_type` + `content` string) → typed `ArtifactModel` (`diff | table | chart | graph | trace | timeline | json | markdown | text`); a payload that does not parse becomes a bounded error, never a throw past this module [model.ts:parseArtifact, model.ts:ArtifactModel]
- `parseDiff(text)` / `parseMarkdown(text)` — content parsers for unified diff and the markdown document convention [model.ts:parseDiff, model.ts:parseMarkdown]
- `RENDERER_LABEL` — display label per renderer kind [model.ts:RENDERER_LABEL]
- `createArtifactService(port)` — artifact resolution: payloads come only from `port.resolveArtifact`, are cached by ref and parsed once; failures stay attached to the ref so the rest of the UI keeps working [service.ts:createArtifactService]
- `ArtifactState` — `loading | ready | error` per ref [service.ts:ArtifactState]
- `useArtifact(service, ref)` — React binding via `useSyncExternalStore` [service.ts:useArtifact]
- `artifactErrorText(e)` — bounded error rendering for artifact failures [service.ts:artifactErrorText]
- `SourceRenderer`, `RendererBoundary`, `RENDERER_KINDS`, `loadedRenderers` (read by the E2E ladder through `__gs`) [registry.tsx:SourceRenderer, registry.tsx:RendererBoundary, registry.tsx:RENDERER_KINDS, registry.tsx:loadedRenderers]
- Child renderers: [./renderers/README.md]

## Invariants
- Payloads come only from the port (`resolveArtifact`); nothing synthesizes artifact bytes [service.ts:createArtifactService]
- Failures stay attached to the artifact ref; the rest of the UI keeps working and the source reference stays visible [service.ts:createArtifactService]
- Content conventions (`text/x-diff` unified diff, `text/markdown` document, `application/json` with a `schema` discriminator: table | chart | graph | realitytrace | timeline; anything else renders as a JSON tree) [model.ts:parseArtifact]
- Each renderer kind is a separate lazy chunk; the build fails if the nine renderers are not nine distinct chunks (VAR-006) [registry.tsx:SourceRenderer]

## Side Effects
- Cached artifact resolution triggers port reads (`resolveArtifact`) on first access per ref [service.ts:createArtifactService]
- `loadedRenderers` records which renderer chunks a session requested (E2E observability) [registry.tsx:loadedRenderers]

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; service/model tests: `src/artifacts/service.test.ts`)
