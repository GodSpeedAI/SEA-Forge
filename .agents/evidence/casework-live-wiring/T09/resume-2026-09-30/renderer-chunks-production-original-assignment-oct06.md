# Emitted renderer chunk checker — Phase 2 original production assignment

Read renderer-chunks-original-assignment-oct06.md, fixture repair original,
independent source rereview and independently accepted assertion RED. Root
independently matched both frozen source hashes and all three actual RED
captures, and verified all13 intended semantic failures. This releases ONLY
the private build assertion and production build hook, not UI runtime or ladder.

## Builder scope

Native apply_patch writes ONLY these existing files:
apps/godspeed-cognitive-ui/src/build/rendererChunkContract.ts and
apps/godspeed-cognitive-ui/vite.config.ts. Read both first and a nearby pattern.
Fixture25b74d25318ce97659c91f3d734ca9b2cd794d0b4d90b1ec54c045b7ce022d08
is frozen; no edits. Original helperc5d9e161 is the accepted throw-only stub.

Implement assertRendererChunks over the existing narrow readonly structural
bundle type. Each of the nine literal full renderer module paths must occur
in exactly one containing output chunk; those nine chunks must have distinct
fileNames and isDynamicEntry true. Find exactly one chunk containing the full
registry module path, including when bundled into the main entry. Its
dynamicImports must contain every renderer filename. Ignore assets and permit
shared dependency chunks/additional imports. Do not rely on facadeModuleId or
registry chunk naming. Normalize backslash separators and strip query/hash
suffixes; a module must equal the required path or end with slash + that path.
not-src/... and basename-only impostors must fail. Treat duplicate matched IDs
in different chunks as duplicate placement. Do not mutate input or add broad
configuration, dependencies, unrelated helper APIs or silent fallbacks.
Throw descriptive Error identifying the violated condition/path; coalesced
renderer filenames must produce a diagnostic containing distinct. Validate
module placement before checking import edges so a missing module is correctly
diagnosed even if its old edge also exists in a corrupt fixture.

Add one small Vite Plugin, contextually typed from Vite if helpful, with
apply:'build' and generateBundle calling assertRendererChunks(bundle). After
successful assertion, emit a build-only informational marker confirming the
nine distinct renderer chunks. Register it alongside the existing React plugin.
Keep dev/preview host/port/proxy and all runtime code unchanged. No generated
manifest/package script/CI change. The checker validates actual emitted build
metadata; it does not claim runtime downloads, J0/J2 behavior or ladder success.

No tests/compiler/build/browser/scanner/Graft build/Git/status/debt/evidence
commands or writes. Formatting diagnostics allowed, native repair only. Return
full actual implementation/diff, three hashes, every deviation and freeze.

## Independent verification

Independent critic receives BOTH originals plus repair assignment, full actual
files and frozen fixture. Verify all required matching, exact module/chunk/edge
conditions, readonly behavior, Rollup structural compatibility, build-only hook
and narrow scope. Explicitly explain all material deviations with direct
source/diff citations; no evidence means no approval. Source approval is separate
from runtime. Root separately releases serialized focused GREEN, strict UI
typecheck and canonical just casework-ui-check with fresh resource captures,
actual joined exits, full source identities and immutable exact output archives.
Actual production build must show the assertion marker and separate emitted
renderer artifacts. Rejection requires a different fresh builder.

T09 remains partial. No publication or live/CI/ladder acceptance is included.
