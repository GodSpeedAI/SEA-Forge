# Independent source review: emitted renderer chunk fixtures, Phase 1

Date: 2026-10-06. Verdict: **REJECT fixture source readiness pending a
path-segment-boundary regression fixture.** This is a source-only review. No
test, compiler, build, browser, scanner, Graft build, Git, status, or debt
operation was run; no source or test file was changed.

## Review envelope and identities

Read the complete original bounded assignment
`renderer-chunks-original-assignment-oct06.md`, both complete frozen files,
and installed Rollup output declarations. The frozen source identities match
the review envelope:

| File | SHA-256 |
|---|---|
| `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.ts` | `c5d9e16187ede70fd3a77e5ec078b99ef0e872b99e7a644aa8be55e204e0fd77` |
| `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.test.ts` | `b14719a9df31f769744d19e0dd29bc9d231cd300dc46138ea41b2b98b2263bda` |

The project instruction boundaries have no `AGENTS.md` under
`apps/godspeed-cognitive-ui` other than a vendored Cytoscape file in
`node_modules`; root and `.agents/AGENTS.md` were read. Graft retrieval was
used before opening source; the relevant target files were not indexed. The
Rollup declarations show `OutputBundle = Record<string, OutputAsset |
OutputChunk>` (`node_modules/rollup/dist/rollup.d.ts:451`), `OutputAsset` has
`type: 'asset'` (`:954-957`), and `OutputChunk` inherits `type`,
`isDynamicEntry`, `dynamicImports`, `fileName`, and `modules` (`:967-993`).
These are structurally assignable to the local narrow readonly shape.

## Source findings

The contract file defines only a readonly structural subset of bundle fields
at lines 1-16 and exports the required `assertRendererChunks` signature with
an explicit throwing stub at lines 18-21. There is no checking algorithm,
runtime import, Rollup import, Vite edit, or added dependency. The type accepts
the installed Rollup `OutputBundle` without requiring extra properties, and
its nested readonly arrays and module map communicate the nonmutation
contract.

The fixture independently lists all nine required full suffixes at test lines
5-15. `makeValidBundle` gives each its own dynamic output chunk, links all nine
from the registry-bearing main chunk, and includes shared dynamic/static
chunks, an additional dynamic dependency, and an asset (lines 25-70). The
registry module itself has query/hash suffixes. The main positive test checks
acceptance and deep equality against a pre-call clone (78-84). The Windows
fixture rewrites each renderer module ID to backslash separators plus query
and hash suffixes and also checks bundle immutability (86-110). The shared
dependency test checks the same valid bundle (112-118); it is redundant with
the first positive case, but the fixture does exercise harmless shared chunks
and an additional import edge.

Every requested corruption has a distinct fixture and a targeted message
expectation: missing renderer plus same-named asset (120-131); duplicate
renderer placement (133-151); coalesced renderer chunks while preserving
registry links (153-185); non-dynamic renderer entry (187-198); removed
registry edge (200-216); missing registry placement (218-225); duplicate
registry placement (227-238); and unrelated same-basename impostor
(240-257). The checks do not weaken the expected successful contract. The
negative tests do not assert post-call equality, but the assigned requirement
specifically asks that each synthetic *valid* bundle be unchanged; all three
valid-test calls compare before/after state. No material scope, type, or stub
deviation was found.

## Blocking fixture gap

The assignment requires matching a **full path segment suffix** and says an
unrelated same-basename module must not satisfy a required renderer. The
existing impostor at test lines 248-254 is only `/workspace/test-fixtures/
DiffRenderer.tsx`. This correctly catches a basename-only matcher, but it
does not distinguish a path-segment-aware matcher from a raw string
`endsWith(requiredSuffix)` matcher. For example,
`/workspace/not-src/artifacts/renderers/DiffRenderer.tsx` ends in the literal
`src/artifacts/renderers/DiffRenderer.tsx` while `src` is not a path segment.
All existing positive IDs have a real slash-delimited `src` component, and
the existing unrelated-basename negative has no overlapping full suffix, so
a boundary-blind full-string suffix check can satisfy these fixtures while
violating the original path-segment rule.

Add a separate negative fixture whose impostor uses a preceding non-segment
prefix such as `not-src/artifacts/renderers/DiffRenderer.tsx`, with no real
Diff renderer module, and assert that this exact required suffix is still
reported missing. Keep the existing same-basename fixture. Until that
distinguishing case is present, the fixture set does not fully pin the
specified matching rule, so source readiness is rejected. This review does
not approve runtime implementation or release the production Vite hook.

## Verification boundary

No fixture execution or assertion RED is claimed. No source/test edits were
made; only this new immutable review record was added. Root owns any fresh
repair assignment and the later compiler token.

Graft retrieval saved approximately 9,348 tokens (<$0.01) during this review
turn.
