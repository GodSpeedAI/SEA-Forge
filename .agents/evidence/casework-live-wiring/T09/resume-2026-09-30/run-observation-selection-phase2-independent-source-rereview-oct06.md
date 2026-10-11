# Run observation selection — independent Phase 1 source rereview

Date: 2026-10-06  
Verdict: **REJECT Phase 1 fixture pending one bounded expected-order correction.** The frozen helper remains an acceptable typed-unavailable stub. No compilation or tests were run; actual assertion RED remains unverified.

## Frozen identities

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_selection.go` | `1e5cdf404b96bd206f2fc1ad9b6f96365740e8e529079f65aaac5673c1176c50` |
| `apps/godspeed-casework-go/internal/server/run_observation_selection_test.go` | `e1c01eccd63a8be6b2c00372c215f03a4e6df9b3b7cb5a98b08d60acb8553ab1` |

The stub hash is unchanged from Phase 1. The complete test file is the sole repaired file. Read-only `gofmt -d` on both files exited 0 with empty output. The stub at `run_observation_selection.go:8-11` remains only the documented `KindUnavailable` body; no algorithm, manager, HTTP/SSE, identity, counters, or polling lifecycle code was added.

## Blocking finding

### Terminal exact-ID tie is expected in descending order

S2 · confirmed · introduced  
**Location:** `apps/godspeed-casework-go/internal/server/run_observation_selection_test.go:71-80`  
**Evidence:** `run_terminal_completed_tie_z` and `run_terminal_failed_tie_a` both have `FinishedAt = 16:00 UTC` (`:71,73`). Their priority and timestamp tie, so exact run-ID bytes decide the order. The literal expected output puts `run_terminal_completed_tie_z` first (`:78`) and `run_terminal_failed_tie_a` second (`:79`). Ascending byte order requires `run_terminal_failed_tie_a` before `run_terminal_completed_tie_z`.  
**Consequence:** A correct deterministic selector will fail this fixture; changing the implementation to satisfy it would violate the assigned ID tie-break.  
**Correction:** Swap those two expected rows only. Preserve the same fields and the explicit terminal timestamp/status cases.

## Repaired requirements verified

- The primary fixture now correctly orders the active 14:00 UTC start before the two 13:00 UTC active rows, retaining ascending ID order for their equal-instant offset tie (`:35-48`).
- All six terminal candidates are selected in a separate short cohort (`:66-82`), including completed, failed, and terminated, with timestamps that make recency independent of execution-state names, start fallback, a missing timestamp, and the exact-ID tie described above.
- Empty input now asserts only zero length (`:96-106`); it no longer prescribes nil-versus-empty representation.
- Short successful output checks values and verifies that mutating a returned summary does not mutate caller input (`:108-123`).
- Main and terminal successful inputs are compared with copies; unsupported-standing input is also compared with a copy after typed unavailable/no-partial-output assertions (`:51-63,83-94,127-150`).
- Existing explicit cutoff, selected full `RunSummary` values, permutation, finish-versus-start fallback, equal instants with different offsets, missing timestamp, and unknown-standing-after-eight cases remain present (`:25-64,127-150`).

No test was executed, so these are source-level checks only. The actual focused assertion RED is still unproven and must be separately authorized by root after a corrected freeze. A different builder should make the one-row expected-order repair; then repeat independent source review before compiler release.

## Review basis and scope

Compared the complete frozen files against the original selection assignment, the fresh fixture-repair original assignment, and the prior Phase 1 rejection. Graft refreshed the named selector/test symbols; the actual complete source and fixture were read. No source or test edits, compiler/scanner/Graft build, Git/status/debt changes, or runtime claims were made. This record is a new immutable rereview and does not overwrite the earlier rejection.
