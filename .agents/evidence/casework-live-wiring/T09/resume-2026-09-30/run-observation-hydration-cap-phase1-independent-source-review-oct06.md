# Run observation hydration cap — independent Phase 1 source review

Date: 2026-10-06  
Verdict: **REJECT the Phase 1 fixture pending bounded repair.** The production file remains a typed-unavailable stub and is within its source-only scope. No compiler, test, scanner, or runtime gate was run; actual assertion RED is unproven.

## Frozen identities

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go` | `1ebfa2e2a002b3c86ca4fd8b599e8b572929feb8ebaa192326658b3c3b8227d7` |
| `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap_test.go` | `b315c20bd0bdc25ad8a594552270b64348b3ac402edf40217f6ef0c2c57741de` |

The production source has only the 1 MiB constant and the exact private signature returning typed `KindUnavailable` (`run_observation_hydration_cap.go:8-12`); it contains no algorithm or wiring. The full source and fixture were read. No source or test edits were made.

## Blocking fixture findings

### F1 · Run execution and settlement values are outside the DTO vocabularies

S2 · confirmed · introduced  
**Location:** `run_observation_hydration_cap_test.go:26-32`  
**Evidence:** `hydrationRun` creates every run with execution `running` and settlement `pending` (`:29`). The normative Go contract permits execution only from `pending`, `enabled`, `active`, `completed`, `failed`, `terminated`, and settlement only from `unsettled`, `accepted`, `rejected`, `escalated` (`internal/contract/contract.go:434-436`). The canonical Go payload checker rejects values outside these separate vocabularies (`internal/contract/golden_test.go:244-245`); the UI mirror has the same allowlists (`wireContract.test.ts:217-218,436-437,538-539`).  
**Consequence:** Positive and negative fixtures built with this helper do not describe well-formed wire DTOs. In particular, a successful trim can still produce a payload rejected by the actual contract checker.  
**Correction:** Use valid independent execution and settlement standings throughout. `validated` at test line 30 is valid: the contract's run-observation state vocabulary explicitly contains `validated` and `unavailable` (`contract.go:438`, mirrored by `wireContract.test.ts:221`).

### F2 · Nil slices serialize as `null` where the contract requires arrays

S2 · confirmed · introduced  
**Location:** `run_observation_hydration_cap_test.go:26-32,35-45,48-76,129-138,335-347`  
**Evidence:** `hydrationObservation()` assigns the zero-argument variadic `runs` directly (`:35-45`), so the empty-payload fixture has `Runs == nil`; `hydrationRun(runID)` assigns zero-argument `frames` directly (`:26-32`), so no-frame runs have `Frames == nil`. `encoding/json` serializes nil slices as `null`. Both canonical contract validators require arrays: Go rejects nil cohort runs (`golden_test.go:214-219`) and nil nested frames (`:250-257`); the UI mirror checks `Array.isArray` for both (`wireContract.test.ts:517,545`). The clone helper also uses `append([]T(nil), input...)` for both slices (`:63-66`), which converts an empty-but-nonnil slice back to nil and cannot serve as a shape-preserving copy oracle.  
**Consequence:** The required empty and no-frame fixtures are malformed, and changing their constructors to nonnil empty slices without fixing the clone helper will make preservation comparisons report the wrong representation. The “metadata alone exceeds cap” fixture (`:336-340`) also has nil `runs`, creating a second invalidity besides size.  
**Correction:** Construct required zero-length `Runs` and `Frames` as nonnil empty slices; update the clone helper to preserve nil-versus-nonnil empty shape; keep empty success expectations representation-neutral only if the contract permits it (here the wire contract requires arrays).

### F3 · Caller immutability and output independence are only tested on the under-cap path

S2 · confirmed coverage gap · introduced  
**Location:** `run_observation_hydration_cap_test.go:154-170,173-266,268-307,309-333`  
**Evidence:** The only test that snapshots the input and mutates returned run/frame arrays, cohort pointers, and optional frame metadata is `TestBoundRunObservationHydrationDoesNotAliasCallerData` (`:309-333`); its input is only one run with one frame and fits well below the cap (`:310`). The actual trimming paths are exercised in the cap-plus-one, oldest-selection, multi-run, and ring-omission tests (`:154-307`), but none snapshots and compares the caller input after success or mutates the returned arrays/pointers to prove independence there. The ring-omission case builds an expected clone and compares `got` to it (`:275-296`) but never compares post-call `input` to a pre-call copy; an implementation could mutate caller storage while returning those same modified values and pass that equality.  
**Consequence:** The required no-mutation and independent arrays/count/optional-frame pointers are unproven for the distinct trimming branch.  
**Correction:** Add an oversized successful case that snapshots the original DTO, checks caller input remains unchanged after trimming, then mutates returned run/frame arrays and cohort/optional-frame pointers and proves the original remains unchanged.

### F4 · The over-eight-runs fixture also exceeds the independent hydration-read budget

S3 · confirmed test isolation gap · introduced  
**Location:** `run_observation_hydration_cap_test.go:35-45,341-347`  
**Evidence:** `hydrationObservation` sets `HydrationReadBudget.ReadsAttempted` to `len(runs)` (`:43`). The “more than eight runs” case constructs nine runs and calls that helper unchanged (`:341-347`), so the case has both nine `Runs` and `ReadsAttempted == 9` with `Limit == 8`. The DTO checker independently rejects a budget with more than eight reads (`golden_test.go:211-213`, mirrored by `wireContract.test.ts:528-530`).  
**Consequence:** The expected unavailable result does not isolate the assigned run-count bound; it can be satisfied because the read-budget field is invalid.  
**Correction:** Keep the budget valid and within its limit in the nine-run case so the tested over-bound condition is the run array itself; preserve other count values within their allowed ranges.

## Remaining fixture coverage assessment

- Correctly computes fixture byte sizes with `encoding/json`, distinguishes exact-cap and cap-plus-one, names the explicitly oldest frame, and checks serialized output does not exceed the cap (`:80-100,141-171`). Padding with repeated ASCII `x` increases JSON size by exactly one byte per character.
- Deterministic cases correctly express global timestamp order, unsorted per-run frame preservation, offset-equivalent instants, complete run-ID and event-ID tie-breaking (`:173-227`); the earlier incorrect zero-frame survivor expectation has been corrected. The naturally oversized multi-run case has eight runs and removes the globally oldest frame (`:229-266`).
- The ring-omission fixture has internally consistent starting counts and expected post-removal counts while preserving `TotalFrameCount` (`:268-307`). Metadata-only over-cap, 1025 frames, inconsistent counts, and invalid timestamp cases are present (`:335-363`); only the nine-run case has the additional invalid-budget confound described above.
- Cohort pointer values are checked on a trim case and pointer independence plus optional `execution_status`/`exit_code` mutation are checked on a separate short success (`:268-333`). The missing coverage is specifically that the latter properties and input immutability are not checked on the trim path.

## Review basis and scope

Compared the complete frozen files with `run-observation-hydration-cap-original-assignment-oct06.md`, the approved 1 MiB/oldest-frame proposal at `run-observation-manager-concrete-proposal-oct06.md:199-200`, Go DTO declarations (`internal/contract/contract.go:196-238`), canonical Go validation (`internal/contract/golden_test.go:196-265`), and the UI wire validator (`apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts:489-569`). Graft refreshed the production/contract context before source inspection. Root's earlier claim that `validated` is invalid is not adopted; the allowed state list proves it is valid. No runtime output or RED claim is made, and the fixture must be repaired by a different fresh builder before independent rereview.
