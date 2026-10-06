# Run observation selection — final independent Phase 1 source review

Date: 2026-10-06  
Verdict: **APPROVE Phase 1 source and fixture for the separately assigned actual-RED gate only.** This is source review, not evidence that the fixture has been compiled or that an assertion has gone RED.

## Frozen identities and checks

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_selection.go` | `1e5cdf404b96bd206f2fc1ad9b6f96365740e8e529079f65aaac5673c1176c50` |
| `apps/godspeed-casework-go/internal/server/run_observation_selection_test.go` | `1e2295a37aec4383f2473226680129f26394253e0ed43d6f05120ef6b30bada4` |

Read-only `gofmt -d` on both files exited 0 with no output. The production file remains the same typed-unavailable stub at lines 8-11. The new fixture diff only adds a nonnil-empty success subtest beside the existing nil-empty subtest; algorithm, callers, manager, HTTP/SSE, identity, counters, and polling lifecycle remain out of scope.

## Correction to the prior rereview

The immediately prior independent rereview at `run-observation-selection-phase2-independent-source-rereview-oct06.md` incorrectly rejected the terminal exact-ID tie. That finding is **retracted as a false positive**. The tie-break compares complete IDs, not suffix letters. The full IDs are `run_terminal_completed_tie_z` and `run_terminal_failed_tie_a`; their first differing byte is `c` (99) versus `f` (102). A deterministic Python standard-library bytes comparison printed:

```text
first differing bytes: (13, 99, 102)
bytewise compare: -1
```

Therefore the current expected order at test lines 78-79 is correct: `run_terminal_completed_tie_z` precedes `run_terminal_failed_tie_a`. This matches Go string byte ordering for these ASCII IDs. Root's adjudication is recorded at `run-observation-selection-root-lexical-adjudication-oct06.md`. The earlier immutable record remains unchanged; this record corrects it.

## Complete fixture assessment

- The main case has explicit newest-first group order, including the 14:00 UTC active start before the equal 13:00 UTC active finish timestamps, then ascending full run-ID bytes for the equal instant (`run-observation-selection_test.go:25-64`). It preserves the >8 cutoff, finish-versus-start fallback, equal instants in different offsets, missing timestamps, and exact explicit full-summary expected values.
- A separate short cohort selects completed, failed, and terminated together and orders by recency, including a started-at fallback, missing timestamp, and an exact full-ID tie (`:66-94`). The bytewise check above confirms its expected tie order.
- Both nil and nonnil empty inputs succeed and assert only output length, without constraining nil-versus-empty output representation (`:96-117`).
- The short success checks values and mutates an output summary to establish that the returned values do not alias the caller's input slice (`:119-135`). Main and terminal successes also compare input against saved copies (`:51-63,83-94`).
- The unsupported execution after eight valid rows asserts typed `KindUnavailable`, no partial output, and unchanged input (`:138-161`).
- Expected identities/values are literal and not derived from a comparator. All current execution categories required by the original assignment remain represented, and the helper remains a stub, so no selector behavior has been assumed by this source review.

## Evidence boundary and review basis

Reviewed complete current source and fixture against the original selection assignment, the fixture-repair assignment, both prior independent review records, and root's lexical adjudication. Graft refreshed the test symbol, then the full file was read. The recorded hash and read-only formatting check match the final freeze. No compilation, tests, scanner, Graft build, or algorithm review was performed. Root may separately authorize the focused actual-assertion RED gate; source approval here does not prove RED and does not approve Phase 2 implementation or wider wiring.
