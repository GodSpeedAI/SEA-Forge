# Independent source review: renderer chunk contract and build hook

Date: 2026-10-07. Verdict: **READY for the separately authorized focused
assertion, strict typecheck, and production-build verification.** Source review
does not itself prove those gates, actual emitted artifacts, runtime downloads,
browser behavior, ladder success, or T09 settlement.

## Review envelope and identities

I read both original assignments in full:

* `renderer-chunks-original-assignment-oct06.md`
* `renderer-chunks-fixture-repair-original-assignment-oct06.md`
* `renderer-chunks-production-original-assignment-oct06.md`

I also read both Phase 1 reviews, the accepted Phase 1 RED record, the Phase 2
source review, and `renderer-chunks-checkpoint-scope-reconciliation-oct07.md`.
The current complete files were inspected directly. Their SHA-256 identities
match the exact identities assigned for this independent review:

| File | SHA-256 |
|---|---|
| `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.ts` | `52cdf00b9650f0413c91e4375c0ce5b6527b63ad9f09219f2102383dca4dec64` |
| `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.test.ts` | `25b74d25318ce97659c91f3d734ca9b2cd794d0b4d90b1ec54c045b7ce022d08` |
| `apps/godspeed-cognitive-ui/vite.config.ts` | `95c287317aa4028a1976c145124f918d0d856a3bb0ca60802f31abd909e901ea` |

## Checker correctness and fixture coverage

`rendererChunkContract.ts:1-16` uses a readonly structural subset of Rollup's
output shape and has no Rollup dependency. The literal contract enumerates all
nine required full renderer paths at `:18-28`, plus the registry at `:30`.
`normalizedModuleId` removes query/hash suffixes and normalizes Windows
separators (`:32-36`). `containsModulePath` accepts an exact relative path or a
slash-delimited suffix (`:38-43`), so basename-only and `not-src` impostors do
not count. `chunksContainingModule` excludes assets (`:45-50`). The function
only reads the bundle and its module keys; it does not alter entries, arrays,
or module maps.

The assertion checks placement before import edges: each renderer must occur
in exactly one chunk (`:53-63`), all nine output filenames must be distinct
and each chunk must be a dynamic entry (`:65-74`), then the registry must occur
in exactly one chunk and that chunk must import each renderer filename
(`:76-89`). Diagnostics identify missing/duplicate paths and non-dynamic
entries; coalescing emits the required `distinct` wording. It does not rely on
facade IDs or naming conventions, and allows unrelated/shared chunks and
additional registry imports as assigned.

The fixture has three valid positives and ten isolated negative cases, for
13 focused tests total (`rendererChunkContract.test.ts:77-289`): valid graph
and nonmutation; Windows separators plus query/hash suffixes and nonmutation;
harmless shared chunks/additional dependencies; missing renderer despite a
same-named asset; duplicate renderer placement; coalesced filename; a
non-dynamic renderer; a missing registry edge; missing registry placement;
duplicate registry placement; unrelated same-basename impostor; renderer
`not-src` boundary impostor; and registry `not-src` boundary impostor. The
last two preserve their chunk/edge context while changing only the relevant
module ID, closing the precise gap identified by the earlier immutable Phase
1 rejection. The expected renderer paths are independently listed in the
fixture (`:5-15`). The accepted RED record reports all 13 tests and 32
expectations reached their intended assertions against the throw-only stub;
that RED establishes fixture reachability only, not GREEN behavior.

## Build hook and material deviations

`vite.config.ts:10-19` registers one `apply: 'build'` plugin whose
`generateBundle` calls the checker and logs the nine-distinct-chunks marker
only after success. It is registered with the existing React plugin (`:21-22`).
The `/api` proxy and dev/preview host, port, strict-port and proxy settings
remain present (`:6-8`, `:23-24`). The helper is imported only by build
configuration. This matches the production assignment's narrow boundary:
private emitted-bundle assertion and build-only hook, without runtime UI,
package, dependency, CI, or public-interface changes.

No material deviation from either original assignment or the repair
assignment was found. One fixture case for shared chunks is redundant with
the first valid-graph case, but both exercise the assigned harmless shared
chunks and additional dynamic import. The existing thirteen-case set still
covers every assigned condition independently.

## Verification boundary and checkpoint distinction

No tests, compiler, typecheck, build, browser, scanner, Graft build, or Git
operation was run for this review. The accepted RED record is historical
fixture evidence. Per the reconciliation record, the three renderer files
are a separate renderer unit and were expressly excluded from cursor
checkpoint `f549bf0`; this review does not call them newly checkpointed. The
reconciliation also records the earlier renderer build evidence and its
source-identity limits for later cursor verification. Therefore this source
approval releases only the already specified serialized focused assertion,
strict typecheck, and production-build checks with fresh preflight, joined
captures, and exact current source identities. A production build must still
show the marker and nine distinct emitted renderer artifacts. No runtime or
checkpoint claim is made here.

Graft retrieval: `graft ask "renderer chunk contract tests and Vite build hook validating chunks and registry edges" --source` (one call; ~24,746 tokens saved, $0.02).
