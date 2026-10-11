# T09 emitted renderer chunks — original bounded assignment

The plan requires a bundle assertion for nine separate lazy renderer chunks.
Registry lazy imports already exist; a source registration test does not prove
emitted chunks. This private build assertion adds no dependency, public API,
package script, CI recipe or runtime UI behavior. Root will separately release
the production Vite hook after independently accepted fixture RED.

## Phase 1: declarations and fixtures only

Read applicable instructions, Graft/Neatcode, registry.tsx, vite.config.ts,
tsconfig.json, installed Rollup output types and a nearby Bun test. Native
apply_patch writes ONLY two NEW files in apps/godspeed-cognitive-ui/src/build/:
rendererChunkContract.ts and rendererChunkContract.test.ts.

Export assertRendererChunks(bundle: RendererChunkBundle): void for the later
private Vite build hook. Define a local structural readonly bundle type accepting
Rollup OutputBundle without a new Rollup dependency: asset entries need only their
type discriminator; chunk entries need type, fileName, isDynamicEntry,
dynamicImports and modules. Do not import the helper into application runtime.
Phase 1 function body is ONLY an explicit Error throwing test-first stub. No
validation algorithm and no Vite edit yet.

The eventual checker must find each of these full module suffixes exactly once
among chunk modules: src/artifacts/renderers/{Diff,Text,Markdown,Table,Chart,
Json,Graph,Trace,Timeline}Renderer.tsx. All nine containing chunks must be dynamic
entries with distinct fileNames. Find the chunk containing
src/artifacts/registry.tsx (it can be the main entry chunk); it must contain
dynamicImports edges to every renderer chunk. Assets do not count. Match a full
path segment suffix, normalize backslashes, and strip query/hash suffixes.
Same basename elsewhere must not satisfy a required module. No reliance on
facadeModuleId or a standalone registry filename. Missing/duplicate registry
module placement is a contract failure as well.

Use explicit synthetic fixtures with distinct renderer filenames and a registry
module in the main entry. Positive tests: valid nine renderer graph plus harmless
asset/shared chunk; Windows paths and query/hash suffixes; permitted shared
dependency imports. Negative tests independently corrupt one condition: missing
renderer, duplicate renderer module placement, coalesced renderer chunks,
non-dynamic renderer chunk, missing registry edge, missing/duplicate registry
placement, and unrelated same-basename renderer impostor. Assert descriptive
condition-specific errors so one blanket throw cannot satisfy all negatives.
Construct expected requirements explicitly, independently of helper internals.
Each synthetic valid bundle must remain unchanged after checking; do not mutate
bundle entries, arrays or module maps. These tests prove the graph assertion,
not J0/J2 browser downloads or current ladder correctness.

No test/compiler/build/browser/scanner/Graft build/Git/status/debt/evidence
commands or writes. Read-only formatting diagnostics allowed; native patch
repairs only. Return full actual result, two SHA-256 hashes, deviations and freeze.

## Independent critic and later releases

Critic receives this ORIGINAL assignment plus both complete files. Verify fixture
validity, each distinguishing corruption, complete requested coverage, minimal
structural type compatibility and stub-only scope. Cite source/diff evidence and
every material deviation; no sufficient evidence means no approval. Root will
separately grant focused Bun assertion RED with fresh RAM/swap preflight and
actual joined capture; only one compiler/heavy owner at a time. Rejection requires
a different builder. No production helper/hook release until actual RED is
independently accepted. Later production review must verify the actual emitted
production bundle via the canonical UI gate. T09 remains partial.
