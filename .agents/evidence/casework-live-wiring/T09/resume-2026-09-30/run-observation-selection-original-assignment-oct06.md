# Run observation selection — original bounded assignment

Root semantic decomposition: implement only deterministic selection from the
already validated, case-scoped readable summaries returned by RunsListForCase.
This is a private manager prerequisite; it does not release manager, HTTP/SSE,
public cursor/bootstrap, identity, counters or polling lifecycle changes.

Existing authority owns row/ownership/duplicate/standing validation
(`authority.go:256-307`); do not duplicate that whole validator. Existing proposal
requires at most eight candidates, active first, other nonterminal next, terminal
last, newest timestamp and run-ID ordering. Root resolves the unspecified ties:
active priority0; pending/enabled priority1; completed/failed/terminated priority2.
Within each priority, use FinishedAt if HasFinished, otherwise StartedAt if
HasStarted; known timestamps precede missing ones; newest instant first, then
exact run-ID bytes lexicographically ascending. Do not infer settlement from
execution and never normalize/convert IDs or mutate input summaries.

## Phase1 fixture and declaration builder

Read root/scoped instructions, governing spec/approved proposal, Graft/Neatcode,
ports.RunSummary, current contract execution standings, authority validation and
one nearby server test pattern. Write ONLY two NEW files:
`apps/godspeed-casework-go/internal/server/run_observation_selection.go` and
`run_observation_selection_test.go`. Private signature:
`selectObservationRuns(runs []ports.RunSummary) ([]ports.RunSummary, error)`.
Phase1 source is ONLY a typed KindUnavailable stub with a clear test-first comment;
no algorithm or wiring. Native apply_patch only, no compiling/tests/scanners/Graft
build/Git/status/debt/evidence edits. Read-only gofmt -d allowed, native whitespace
repair only. Return hashes, complete bounded diff, deviations and definitive freeze.

Fixtures distinguish the ranking groups, finish-vs-start fallback, absent timestamp,
equal-instants with different offsets, exact-ID ascending ties, and >8 cutoff.
Exercise reordered input with identical expected output, short/empty input, preserve
execution/settlement and every summary field, and prove caller slice unchanged.
Expected results must be explicit identities/values, not derived with the production
comparator. Include unsupported execution standing as typed-unavailable/no partial
selection, even if it occurs after the first eight otherwise valid rows. No source
calls/network/time sleeps are needed. Do not invent ID validation or cohort counts.

## Independent Phase1 verification

Independent critic gets this ORIGINAL assignment and full result. Verify fixture
correctness/completeness, exact preconditions/ties/cutoff/immutability, all material
deviations and source stub-only scope. No evidence means no approval. Source approval
then separately root-assigned narrow actual race test must reach expected assertion
RED, not type/setup errors. Fresh resource preflight/captures/joins required. Only
one compiler at a time. On rejection use DIFFERENT fresh builder.

## Phase2 held until Phase1 acceptance

After independently accepted actual assertion RED, root separately releases the
same private helper's algorithm with fixtures frozen: single pass retaining at most
eight summary values (O(n*8) time, O(8) additional retained storage), fresh bounded
output, no sorting/mutating the input or allocating a map/full-input copy. Inspect
all input standings; unknown standing returns typed unavailable and no partial data.
Every other property is preserved exactly. Independent code/runtime verification
and server/full-module/canonical gates required before root accepts the prerequisite.
No manager reads, authentication, publication, public interface or T09 completion
claim is authorized by this selector unit.
