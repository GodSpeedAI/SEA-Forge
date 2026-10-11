# Run observation selection — independent Phase 1 source review

Date: 2026-10-06  
Verdict: **REJECT Phase 1 fixture pending bounded test repair.** The declaration file is an acceptable stub, but the fixture has an incorrect expected order and does not prove terminal-state ranking. No compilation or test command was run; actual assertion RED remains unverified.

## Frozen source identities

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_selection.go` | `1e5cdf404b96bd206f2fc1ad9b6f96365740e8e529079f65aaac5673c1176c50` |
| `apps/godspeed-casework-go/internal/server/run_observation_selection_test.go` | `53bb91b66c89b193d342d0a613e1c1e0b7d442bf8f51537e1dbf747bf4534b15` |

The complete change is exactly these two new files. `run_observation_selection.go:8-11` contains only the documented `KindUnavailable` stub with the exact private signature and imports; it adds no selection algorithm or wiring. Read-only `gofmt -d` on both frozen files exited 0 and emitted no output. No gate or runtime result is claimed.

## Blocking fixture findings

### Expected active-run order contradicts newest-instant ordering

S2 · confirmed · introduced  
**Location** `apps/godspeed-casework-go/internal/server/run_observation_selection_test.go:26-38,40-49,51-58`  
**Evidence** `run_active_started` has `StartedAt = 14:00 UTC` and no finish (`:38,43`); `run_active_a` and `run_active_z` finish at `09:00` in UTC−04:00 and `13:00 UTC`, respectively, so both instants are 13:00 UTC (`:26-27,37,41-42`). The literal expected list puts `run_active_started` after both 13:00 runs (`:41-44`).  
**Consequence** A correct newest-instant-first implementation will fail this fixture, or an implementation could be distorted to satisfy a false order. The focused fixture therefore cannot be accepted as the selection oracle.  
**Correction** Put `run_active_started` before the two 13:00 UTC runs; retain `run_active_a` before `run_active_z` for the exact-ID ascending tie. Keep `run_active_finished` after them because its selected recency is 12:00 UTC, despite its 20:00 UTC start.

### Cutoff prevents the fixture from proving all terminal statuses share the terminal group

S2 · confirmed · introduced  
**Location** `apps/godspeed-casework-go/internal/server/run_observation_selection_test.go:28-39,40-49`  
**Evidence** The input contains one `completed`, one `failed`, and one `terminated` run (`:29,32,36`), but five active and two other nonterminal runs precede terminal priority. With the eight-item cutoff, only `run_completed` appears in the expected output (`:40-49`); `run_failed` and `run_terminal_missing` are both omitted. No expected pair compares failed/terminated against another terminal or exercises their terminal recency order.  
**Consequence** The required terminal priority group is not established for two of its three supported execution states. A comparator that ranked `failed` or `terminated` differently could still satisfy every current successful-selection assertion.  
**Correction** Add a literal expected case where completed, failed, and terminated candidates all fit inside the eight selected slots and demonstrate newest timestamp ordering within that shared group, including a missing timestamp if useful. Retain a separate over-eight cutoff case.

### Empty-input assertion over-specifies nil-versus-empty representation

S4 · confirmed · introduced  
**Location** `apps/godspeed-casework-go/internal/server/run_observation_selection_test.go:66-77`  
**Evidence** The same `reflect.DeepEqual(got, runs)` assertion is used for a nonnil zero-length input and a one-element input (`:67-76`). A successful nil empty result has length zero but is not deeply equal to the nonnil empty input. The assignment requires successful empty input, but does not define nil-versus-empty representation.  
**Consequence** This adds an incidental representation constraint to the Phase 1 fixture, separate from selection semantics.  
**Correction** Assert `len(got) == 0` for the empty case; retain full value equality for the one-item case.

## Requirements met by this source snapshot

- The helper is private and stub-only; there are no manager/API/poller changes.
- The primary fixture uses explicit expected summary values rather than deriving expected order from a comparator. It checks original and reversed input against the same oracle and compares the full input with a copy afterward (`:40-63`), covering identity, parent, both standings, timestamps/flags, and evidence count for selected rows.
- The input includes the six supported execution values; the unknown execution row occurs after eight otherwise valid rows and asserts typed unavailable with no partial selection (`:81-100`).
- The timestamp fixture includes an equal instant in different offsets, exact run-ID tie ordering, finish-versus-start fallback, and missing timestamps. The incorrect order above still blocks approval.
- The short and empty cases are both present, subject to the over-specified empty representation finding.

The terminal-group gap is not cured by the fact that failed/terminated values occur in the input: their selected ranking behavior is hidden by the cutoff. No fixture was executed, so this record is source review only and does not approve an actual assertion-RED result. A fresh different fixture builder is required for the bounded repair; after a frozen repaired fixture, independent source rereview must precede the separately assigned focused RED gate.

## Retrieval and scope

Graft refreshed the two changed files and returned their API outlines. The complete files and final hashes were then inspected. The governing assignment, approved cohort selection order, RunSummary fields, valid execution standings, and existing authority validation were read as context. No production code, tests, Git state, handoff status, or ledger outside this new immutable review record was changed.
