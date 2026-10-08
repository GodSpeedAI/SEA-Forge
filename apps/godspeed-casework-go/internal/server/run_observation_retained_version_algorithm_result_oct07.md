# Retained candidate algorithm phase — immutable result

Date: 2026-10-07. Source-only implementation; no compiler, test, scanner,
formatter, typecheck, runtime, or Git command was run.

## Assignment and source baseline

The full release assignment is preserved verbatim in
`run_observation_retained_version_algorithm_release_assignment_oct07.md`.
It authorized only `run_observation_retained_version.go`, after an accepted
actual semantic RED, and required an independent source critic before any
compiler GREEN. Its governing original assignment, preregistration, root
algorithm decisions/decomposition, and current RED review are identified in
that assignment. Root retains architecture and orchestration authority.

Before editing, I decoded the archived JSON content preimage
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/retained-helper-d8df-before-algorithm-oct07.json`.
Its exact 5,836 source bytes matched the live source and SHA-256
`d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331`.
The source archive is a lossless JSON content preimage, not a raw filesystem
copy.

## Change and resulting identity

Only `run_observation_retained_version.go` changed. It replaces the rejected
candidate stub with the pure candidate builder and private helpers for terminal
standing, frame pointer copying, full safe-state copying, and marker copies.
Current source is 10,389 bytes, SHA-256
`38ca7fdaf45b016fb8a55fdb72a32b15cad100fb5b31585410af943dddf7447a`.
The complete source diff is the exact old-to-new difference from the archived
preimage above; it changes only the builder comment/body and adds those four
helpers. The canonical image declarations and marshaler are byte-for-byte
unchanged.

The implementation checks snapshot and previous-state identity, count/window
bounds, and duplicate IDs. It trusts the RunTracePort safe-projection contract
for other validation. It builds a new generation and exact lifetime ledger in
source order, reuses existing ordinals, checks overflow before adding an ID,
clones optional pointers, stamps only candidate versions with the supplied
RFC3339Nano time, and accepts only a full canonical image at or below 1 MiB.
It handles nonterminal/terminal budget and ordinal refusal, generation stop,
final-marker refusal, and recovery from read/retention markers according to
the ratified decisions. No rejected candidate values are put in marker copies.

The frozen base test, policy test, manager source, and manager test remain at
their verified pre-edit identities:

* base test: `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`
* policy test: `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`
* manager source: `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`
* manager test: `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`

## Deviations and limits

No assignment deviations. The implementation leaves the canonical image
shape, JSON field order, and encoding unchanged. The evidence records are
package-local because `.agents/evidence` is read-only to this worker; root may
archive them natively. No compiler/test result or implementation correctness
claim is made. A different manager critic must review the full source together
with the original assignment and this result before root authorizes a GREEN
run.
