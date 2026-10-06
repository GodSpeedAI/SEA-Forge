# Run observation selector — Phase 2 production assignment

Read the original `run-observation-selection-original-assignment-oct06.md`,
final independent Phase 1 source review and focused RED runtime verdict in this
directory. Root accepts the compiled assertion RED (session66279, exit1), with
successful-selection tests failing on the typed unavailable stub. That releases
only this private production prerequisite, not manager or public wiring.

## Builder boundary

Read applicable repository/scoped instructions, Graft/Neatcode, the existing
RunSummary/error definitions and the complete frozen fixtures. Change ONLY
`apps/godspeed-casework-go/internal/server/run_observation_selection.go` using
native apply_patch. Frozen fixture SHA-256:
`1e2295a37aec4383f2473226680129f26394253e0ed43d6f05120ef6b30bada4`.
Keep signature `selectObservationRuns([]ports.RunSummary) ([]ports.RunSummary, error)`.

One pass over every input row, retaining at most eight summary values: O(n*8)
time and O(8) retained storage. No map/full-input copy, no mutation/sorting of
caller input. Output must have separate bounded backing storage. Preserve every
summary field exactly, including settlement and timestamp flags.

Rank active first; pending/enabled next; completed/failed/terminated together
last. Within a rank use FinishedAt iff HasFinished, otherwise StartedAt iff
HasStarted. Known timestamps precede missing; newer instants precede older
(time equality is instant equality across offsets). Final ties use the complete
unmodified RunID bytes ascending, not suffixes. Settlement never ranks a row.
An unknown execution standing anywhere, including after eight retained rows,
must return typed KindUnavailable and no partial output. Empty nil/non-nil
inputs both succeed with zero output length; nil versus empty is unspecified.
Authority already owns case/ID/duplicate validation; do not add another validator.

Private helpers/constants in this file are allowed when necessary. No source
outside it, fixture edits, dependencies, counters, manager/auth/HTTP/SSE/public
interfaces or identity changes. No compiling/tests/scanners/Graft build/Git/
status/debt edits. Read-only gofmt -d allowed; repair whitespace with native
patch. Report full actual implementation/diff, source and fixture hashes,
deviations, and definitive freeze. Stop at any conflicting instruction.

## Independent critic

Receive BOTH original assignments and the full actual implementation. Read all
code and fixtures; verify ordering, bounded allocation/time, all-row rejection,
field preservation and input/output independence. Explain every material
deviation, cite exact source/diff evidence and independently recompute hashes.
No evidence means no approval. Source approval is separate from runtime approval;
no compiler is released by this assignment. Root separately grants serialized
focused race GREEN, full-server race, full-module race and canonical Go gates
with fresh resource captures, actual joined exits, frozen identities and archive
byte comparisons. Rejection requires a different fresh builder.

T09 remains partial. This assignment does not approve live manager behavior,
public correction, current UI evidence, CI or publication.
