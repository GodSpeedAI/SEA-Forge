# Run observation hydration cap — independent final Phase 1 source review

Date: 2026-10-06  
Verdict: **APPROVE fixture source for separately authorized actual assertion RED only.** This does not approve an implementation or any runtime result. The production source remains the required typed-unavailable stub.

## Frozen identities and review basis

- Production stub `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go`: SHA-256 `1ebfa2e2a002b3c86ca4fd8b599e8b572929feb8ebaa192326658b3c3b8227d7`.
- Fixture `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap_test.go`: SHA-256 `0e3b1774303fa6b155269e45adb0703262b789611a81993d4c2b63bd9e8140f9`.
- Original assignment SHA-256 `ed68ba884fac5673d6b8f6093612a3f3e2c342176beaf03a8cf88cd8dccce37d`.
- Fixture repair assignment SHA-256 `3ce3a79b6d38c0695ed85482d577460fb0bf0842a13703e6f7de145ae4afe827`.

Reviewed both assignments, the prior Phase 1 rejection and rereview, the complete frozen stub and fixture, and DTO/golden validation. Graft refreshed the changed fixture context before inspection. No compiler, test, scanner, runtime gate, Git/status/debt operation, or source edit was performed.

## All previously identified fixture defects are repaired

- `hydrationRun` now uses allowed values `active`, `unsettled`, and `validated` (`run_observation_hydration_cap_test.go:26-34`; authoritative vocabularies `internal/contract/contract.go:434-442`).
- Constructors allocate nonnil empty run/frame arrays, while `cloneHydrationObservation` preserves nil versus nonnil-empty shape and copies nested slices, cohort count pointers, and optional frame pointers (`:26-91`). This matches the required JSON array fields (`internal/contract/golden_test.go:214-219,250-257`).
- A cap-plus-one trim test snapshots the original, confirms input immutability, checks run/frame storage independence, mutates returned run/frame identities, execution-status and exit-code pointers, and all six count pointers, then confirms the original remains unchanged (`:168-204`).
- The nine-run fixture keeps the read budget within its limit and sets the other cohort counts consistently for eight selected/validated, one omitted (`:376-386`).

## Final-frame removal count coverage is now explicit

The natural multi-run oversize test spans exactly eight runs and removes the globally oldest frame from `run-old`. It now asserts that the emptied run retains its identity, observation time, plan parent, execution/settlement standings and nested state; preserves total count; reports zero retained, increments omitted exactly once, sets `truncated=true`, and has an empty frame array (`:264-324`). It deep-compares every unaffected run, including its frames and all frame-count fields (`:310-324`). This resolves R1 from the immediately preceding immutable rereview.

**Erratum to the immediately preceding rereview:** the same-run EventID tie case at `:229-235` removes `evt-a` but retains `evt-z`; it does not empty that run. Only the complete RunID tie case at `:222-228` and the natural multi-run case at `:264-324` remove a run’s final frame. The former rereview’s mention of the EventID tie among zero-frame cases was inaccurate. This record corrects that description without modifying prior immutable records.

## Expected-value and contract audit

The exact-cap case measures `encoding/json` bytes independently and requires no change; cap-plus-one names the older timestamp and checks the surviving frame and exact count change (`:155-204`). Deterministic ordering cases use explicit expected IDs: preserve the unsorted within-run order after global timestamp removal; treat offset-different RFC3339 values as equal instants; remove the older complete RunID byte string at the run tie (`run-1` before its extension `run-10`); and remove `evt-a` at the event tie while preserving `evt-z` (`:208-262`). Run order is separately required to remain unchanged.

The naturally oversized multi-run case removes the 01:00 `run-old` frame while retaining seven 02:00–08:00 frames, and now verifies the removed run’s full metadata plus full unaffected-run equality (`:264-324`). The existing ring-omission fixture starts with total 5, retained 2, omitted 3, then requires total 5, retained 1, omitted 4, truncated true while preserving the other run and cohort fields (`:326-384`).

The empty/short preservation, metadata-only over-cap, nine runs, 1,025 frames, inconsistent count metadata, invalid timestamp, no-partial typed-unavailable, and failed-call input-immutability cases remain in the fixture (`:143-153,370-410`). The production file is still exactly the required signature plus typed `KindUnavailable` stub; no algorithm or wiring has been added.

## Scope and release boundary

No material deviation from either original assignment remains in the reviewed fixture. This is source/fixture approval only. No actual RED is claimed. Root must separately authorize the focused test command and its evidence protocol; the resulting failure must be attributable to the throwing stub and all required assertions must be reached before it serves as the intended RED proof. Root retains approval for that runtime boundary and all later implementation/gate claims.
