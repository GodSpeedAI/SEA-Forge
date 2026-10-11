# Run observation hydration cap — independent Phase 2 production source review

Date: 2026-10-06  
Verdict: **APPROVE the private production helper for separately authorized GREEN runtime verification.** This is source approval only; no implementation pass or wider gate is claimed.

## Frozen identities and assignments

- Production source `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go`: SHA-256 `3dfa2993004676a13b1dd9c449965c3b531a199885db325a6fd14941a90f501a`.
- Frozen fixture `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap_test.go`: SHA-256 `0e3b1774303fa6b155269e45adb0703262b789611a81993d4c2b63bd9e8140f9`.
- Original Phase 1 assignment `run-observation-hydration-cap-original-assignment-oct06.md`: SHA-256 `ed68ba884fac5673d6b8f6093612a3f3e2c342176beaf03a8cf88cd8dccce37d`.
- Production Phase 2 assignment `run-observation-hydration-cap-production-original-assignment-oct06.md`: SHA-256 `f2edeabd1a8144bb82dce28ccf87446285afe483365b0a8b62dd8654c2d93b26`.
- Fixture repair assignment `run-observation-hydration-cap-fixture-repair-original-assignment-oct06.md`: SHA-256 `3ce3a79b6d38c0695ed85482d577460fb0bf0842a13703e6f7de145ae4afe827`.
- Zero-frame metadata repair assignment `run-observation-hydration-cap-zero-frame-repair-original-assignment-oct06.md`: SHA-256 `a1c70a25d6f7bbaa4221461a1b389ca8f2e74c1dac1568d72445dbd3572f9953`.

The complete current implementation and frozen complete fixture were read and compared with all four assignments and the Go DTO/canonical validator. Graft refreshed the source context before direct inspection. No source/test edit or compiler/build/test command was run in this source review.

## Implementation evidence against required behavior

### Bounds, counts, timestamps, and typed failures

`hydrationFrameCoordinates` rejects more than eight runs and more than 1,024 frames per run (`run_observation_hydration_cap.go:80-88`). It rejects negative counts, retained count unequal to slice length, total below retained, omitted unequal to `total-retained`, and truncation inconsistent with omitted count (`:89-94`). It parses every retained frame timestamp as RFC3339Nano before accepting the DTO (`:95-104`). Every invalid branch returns a zero `RunTraceObservation` with `apperr.KindUnavailable` (`:22-26,192-194`); JSON marshal and irreducible metadata overflow also return zero output and typed unavailable (`:28-37,51-57,145-149`). No partial result escapes.

This is the requested scope rather than a broad wire validator: the assigned trusted caller remains responsible for ownership, identity grammar and safe-kind validation. The helper makes no execution/settlement inference or policy change.

### Exact payload cap and oldest-prefix selection

The helper marshals a deep clone with `encoding/json` and compares its exact bytes against `1<<20` (`:12,28-35`). When oversized, coordinates are sorted by parsed instant, then full Go string byte ordering of `runID`, then `eventID` (`:40-49`); Go string comparison is lexicographic over the underlying bytes, matching the assignments’ complete-ID ordering. It removes a prefix of this global ordering, but rebuilds each run’s frame slice by walking its original indexes, so run order and surviving within-run frame order remain unchanged (`:109-143`). Runs remain in the output even when no frame survives (`:124-143`).

The binary search first verifies that removing all frames leaves metadata that fits (`:51-57`), then finds the smallest fitting prefix (`:62-77`). The documented monotonicity argument is sound: each removed frame has a nonempty JSON object with fixed required field names, while one removal changes only that run’s retained/omitted decimal counts; their combined digit-length change is at most +1 byte, less than the removed object and array separator contribution (`:59-61`). Thus serialized length strictly decreases with each prefix step. Search uses bounded sorting and `O(log n)` full JSON candidate serializations for at most 8,192 coordinates, rather than marshaling once per frame.

### Count and field preservation

For each nonnil frame slice, the helper keeps original `TotalFrameCount`, sets retained count to the output slice length, omitted count to `total-retained`, and sets truncation exactly from omitted count (`:133-143`). Previously omitted ring frames remain counted because total is not changed. Other run fields and all cohort fields remain copied from the input. The clone function independently copies the cohort’s six optional integer pointers, run slice, per-run frame slices, and optional execution-status/exit-code pointers while preserving nonnil empty slices (`:152-190`). The trimmed candidate starts from this clone; input data is never mutated.

The frozen fixture directly asserts exact size and no-change behavior at the cap, explicit oldest-frame removal at cap-plus-one, timestamps with different offsets, full RunID/EventID ties, unsorted per-run order, ring plus hydration omission accounting, zero-frame run identity/standing/counts, unaffected-run equality, and trim-branch input/output independence (`run_observation_hydration_cap_test.go:143-204,208-326,328-393`). Invalid bounds/count/time and metadata overflow cases return typed unavailable with zero result and unchanged input (`:395-428`). The fixture has already had its source review and expected stub RED accepted; that RED was against the old stub and does not establish this implementation’s behavior.

## Caller precondition and remaining boundary

This is a private sizing/trimming helper over an already well-formed DTO, as the production assignment specifies. It preserves slice shape: nonnil empty `Runs`/`Frames` stay nonnil empty, and nil slices stay nil (`:152-168`; trimmed empty frame results use `make` at `:133-140`). A nil slice marshals as `null`, which the canonical DTO validator rejects (`internal/contract/golden_test.go:217-219,253-257`). The current helper does not itself reject or normalize nil slices. This is acceptable for the assigned private precondition, not a guarantee for arbitrary zero-value DTOs; the future cohort caller must construct nonnil arrays and otherwise supply a well-formed DTO. There is no production callsite yet (`rg` found only the function definition and fixture calls), so this review does not approve any future wiring or claim this precondition has been met in production.

No other material deviation or source defect was identified. The 1 MiB check covers only the serialized `RunTraceObservation`, not transport or heap allocation; no broader resource-bound claim is made.

## Runtime boundary and conclusion

The separate focused stub RED is recorded in `run-observation-hydration-cap-focused-red-01-runtime-review-oct06.md`. This source review authorizes no command itself and does not make that RED a GREEN result. Root separately owns release and serialization of implementation verification. The implementation may proceed to the assigned fresh-preflight GREEN gates only after root’s acceptance of this source verdict. No full server/module/canonical gate, Graft build, manager wiring, SSE/public route, or T09 completion claim is included.
