# Run observation selection — independent Phase 2 production source review

Date: 2026-10-06  
Verdict: **APPROVE the frozen private selector implementation for root's separately assigned runtime gates.** This is source approval only; no runtime GREEN or manager/live-wiring claim is made here.

## Frozen identities

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_selection.go` | `b9de3dae19ea9f473eaa0bbf680d06bcab3975195d90eea3bc0e40866c1ca63d` |
| `apps/godspeed-casework-go/internal/server/run_observation_selection_test.go` | `1e2295a37aec4383f2473226680129f26394253e0ed43d6f05120ef6b30bada4` |

Both hashes were independently recomputed from the complete current files and match the freeze. Read-only `gofmt -d` on both exited 0 with empty output. The full implementation and the full unchanged fixture were read.

## Contract and implementation assessment

Compared the source against both originals—`run-observation-selection-original-assignment-oct06.md` and `run-observation-selection-production-original-assignment-oct06.md`—plus the accepted Phase 1 review and focused actual-RED runtime verdict.

- `selectObservationRuns` retains the requested signature and scans every input row in order (`run_observation_selection.go:14-21`). It validates the execution category before considering whether a row will fit in the top eight; an unsupported category returns typed `KindUnavailable` and a nil result, even after eight candidates have already been retained (`:18-21`). No partial selection can escape.
- Retained state is a fixed `[8]ports.RunSummary` (`:10,15-16`). Each valid row is inserted into the bounded ordered prefix with at most eight comparisons and shifts (`:23-44`). Once full, a candidate after all retained rows is skipped; a candidate inside the prefix shifts rows and evicts the old eighth. This is one pass, O(n*8) time and O(8) retained storage, with no sort, map, or full-input copy.
- The result is a new, bounded slice allocated only after the scan and populated by value-copying the selected prefix (`:46-48`). It does not share the caller's input backing array, and every `RunSummary` field is preserved. The actual struct has only value fields—IDs/standings, timestamps/flags, and evidence count (`ports/ports.go:276-289`)—so struct assignment preserves the complete summary.
- Group ranking is exactly active 0, pending/enabled 1, and completed/failed/terminated 2; unknown values rank invalid (`:51-62`). Settlement is not read for ranking.
- Recency uses FinishedAt iff `HasFinished`, otherwise StartedAt iff `HasStarted`, otherwise missing (`:82-90`). Known times precede missing, unequal instants sort newest first, and `time.Time.Equal` treats equal instants across offsets as a tie (`:71-79`). The final comparison is Go string `<` on the complete unmodified `RunID`, which compares bytes lexicographically; no ID normalization or conversion occurs (`:79`).
- The original focused fixture remains hash-frozen and explicitly covers groups, finish/start fallback, offset-equivalent times, exact full-ID ordering, missing timestamps, cutoff, permutation, full-field preservation, caller-input immutability, both nil/non-nil empty inputs, short-result non-aliasing, and an unsupported standing after eight valid rows (`run_observation_selection_test.go:25-161`). Its prior source review and accepted RED runtime record establish the pre-implementation assertion boundary only.

## Deviations and scope

No material deviation from either original assignment was found. The implementation adds only the named private rank, comparison, and recency helpers plus the private limit constant, all in the authorized selector source file. It adds no duplicate scope/ID/duplicate validator, no manager or public wiring, no dependency, and no change to fixtures or `RunSummary`.

## Evidence boundary

Graft refreshed the selector and `RunSummary` symbols; complete current source and fixture were then inspected. The focused Phase 1 RED record is read as prior evidence, not repeated here. No compiler, test, scanner, Graft build, Git/status/debt edit, or source/test edit was performed for this review. Root owns and must separately authorize sequential focused GREEN, full-server race, full-module race, and canonical gates. This approval does not establish any of those results, production invocation, live manager behavior, public correction, T09 completion, CI, or publication.
