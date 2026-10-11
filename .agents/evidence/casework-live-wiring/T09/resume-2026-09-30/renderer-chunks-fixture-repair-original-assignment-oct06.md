# Renderer chunks fixture repair — original bounded assignment

Read renderer-chunks-original-assignment-oct06.md and the independent Phase1
source rejection. Root confirms the missing path-segment-boundary attack:
not-src/artifacts/renderers/DiffRenderer.tsx ends with the required src/... text
but is not that path segment. The existing basename impostor catches a different
defect. Retain it and every other fixture.

A DIFFERENT fresh builder may change ONLY
apps/godspeed-cognitive-ui/src/build/rendererChunkContract.test.ts with native
apply_patch. Add a distinguishing negative fixture which replaces the legitimate
Diff module ID inside its existing otherwise-valid dynamic chunk with
/workspace/not-src/artifacts/renderers/DiffRenderer.tsx. Keep the chunk filename,
registry edge and other eight renderers intact, so the only defect is the module
path boundary. Assert an error identifying the missing required Diff path.
Also cover the registry path boundary similarly if needed: replace its legitimate
module ID with /workspace/not-src/artifacts/registry.tsx without removing edges.

Helper c5d9e16187ede70fd3a77e5ec078b99ef0e872b99e7a644aa8be55e204e0fd77
must remain frozen. Existing fixture hash before repair is
b14719a9df31f769744d19e0dd29bc9d231cd300dc46138ea41b2b98b2263bda.
No other source/Vite/helper/fixture/dependency edits, no compiler/test/build/
browser/scanner/Graft build/Git/status/debt/evidence commands or writes. Read-only
format diagnostics allowed. Return exact diff/two hashes/deviations/freeze.
Independent critic receives both originals and full result; approval requires
direct evidence and explicit material deviations. No runtime release here.
