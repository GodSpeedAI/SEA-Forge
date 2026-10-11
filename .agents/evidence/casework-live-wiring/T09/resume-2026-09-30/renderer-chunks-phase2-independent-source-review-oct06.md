# Independent source review: emitted renderer chunk checker, Phase 2

Date: 2026-10-06. Verdict: **APPROVE the source implementation and build-hook
scope.** This does not verify an actual production bundle or approve runtime,
browser, ladder, or T09 behavior; the canonical UI gate remains required.

## Review envelope and identities

Read the full Phase 2 production assignment, Phase 1 original assignment and
fixture-repair assignment, independent Phase 1 source rereview, accepted
assertion RED record, and complete current helper, Vite config, and frozen
fixture. Current hashes match the assignment:

| File | SHA-256 |
|---|---|
| `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.ts` | `52cdf00b9650f0413c91e4375c0ce5b6527b63ad9f09219f2102383dca4dec64` |
| `apps/godspeed-cognitive-ui/vite.config.ts` | `95c287317aa4028a1976c145124f918d0d856a3bb0ca60802f31abd909e901ea` |
| `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.test.ts` | `25b74d25318ce97659c91f3d734ca9b2cd794d0b4d90b1ec54c045b7ce022d08` |

## Contract implementation

The checker declares only the readonly structural Rollup subset at
`rendererChunkContract.ts:1-16`. The installed declaration confirms
`OutputBundle` is `Record<string, OutputAsset | OutputChunk`
(`node_modules/rollup/dist/rollup.d.ts:451`); assets have the `asset`
discriminator (`:954-957`), while chunks supply `type`, `isDynamicEntry`,
`dynamicImports`, `fileName`, and `modules` (`:967-993`). The local shape
therefore accepts Rollup's mutable bundle without importing or adding a
Rollup type dependency. The assertion only reads the input; its local arrays
and set do not mutate entries, arrays, or module maps.

The literal nine renderer paths and registry path are declared at
`rendererChunkContract.ts:18-30`. `normalizedModuleId` strips the earliest
query/hash suffix and converts backslashes to forward slashes (`:32-36`).
`containsModulePath` accepts only the exact relative path or a slash-delimited
full suffix (`:38-43`), so both `not-src/...` boundary cases and unrelated
same-basename modules are excluded. `chunksContainingModule` filters to chunk
entries, excluding assets (`:45-50`).

The assertion rejects missing and multiply placed renderer paths with
path-specific diagnostics before testing chunk edges (`:53-63`); it then
requires distinct output filenames and dynamic-entry status (`:65-74`). It
requires exactly one containing registry chunk and checks each renderer
filename in that chunk's `dynamicImports` (`:76-89`). This also diagnoses
coalesced renderers with the required `distinct` text. No `facadeModuleId`,
registry filename, or renderer filename convention is used. Additional
imports and unrelated shared chunks are permitted by these exact membership
checks.

The frozen fixture hash is unchanged from the accepted Phase 1 RED. Its
independent nine-path expectation, valid renderer/registry graph, harmless
asset/shared chunks, Windows/query/hash case, nonmutation comparisons, all
original single-condition negatives, and both renderer/registry `not-src`
boundary negatives remain in `rendererChunkContract.test.ts:5-289`. The
accepted RED record reports all 13 tests reached their assertions and failed
on the explicit stub with joined exit 1. The Phase 2 source removes that stub
and implements the behavior those fixtures specify; the RED evidence itself
does not prove this implementation passes.

## Build hook and scope

`vite.config.ts:10-19` adds one contextually typed Vite `Plugin` with
`apply: 'build'`; `generateBundle` runs the checker against the emitted
Rollup bundle and logs the success marker only after it returns. The plugin
is registered alongside the existing React plugin (`:21-22`). The existing
`/api` proxy target and `changeOrigin` values remain at `:6-8`, and both dev
and preview host, port, strict-port and proxy settings remain at `:23-24`.
The helper is imported by build configuration, not application runtime code.
No runtime renderer/registry or port code is changed in this assignment.

No material deviation from the Phase 2 assignment was found in the reviewed
files. The build hook's marker is informational and does not claim actual
browser downloads. Source review does not substitute for the required
production build that demonstrates the marker and distinct emitted artifacts.

## Verification boundary

No tests, typecheck, build, browser test, compiler, scanner, Graft build, Git,
status, or debt operation was run. Root retains the separate compiler token
and production-build acceptance decision.
