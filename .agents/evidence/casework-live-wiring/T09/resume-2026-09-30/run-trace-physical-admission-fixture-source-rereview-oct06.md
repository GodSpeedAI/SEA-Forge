# Physical run.get admission fixture — independent source re-review

Date: 2026-10-06  
Verdict: **APPROVE bounded declarations/fixtures for separately owned focused assertion-RED compilation only.**  
Scope: source-only review. This does not approve production limiter behavior, runtime integration, or T09 settlement. No compiler, test, runtime, Git, status, or debt operation was performed.

## Review basis

Read the original `run-trace-physical-admission-fixture-root-assignment-oct06.md`, the original limiter proposal and its rejection, repair proposal, root adjudication, round-two rejection, wait clarification and approving supplement, my prior immutable rejection `run-trace-physical-admission-fixture-source-review-oct06.md`, and root's binding `run-trace-physical-admission-fixture-root-review-supplement-oct06.md`. Graft was refreshed before inspecting source. I inspected both complete fixture files, the unchanged stub and Config field, and the existing `Client.discard`, `Client.release`, and cancellation callback ordering.

## Actual changed paths and identities

Only the two fixture files changed in this repair. Root's previously verified declaration and Config hashes remain unchanged:

| Path | SHA-256 | Review |
|---|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `d2f993d7dda3a5d63a414ef5a64d6f995c04de15f94537374940dc6c031027c6` | unchanged; Config field only |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go` | `f661bd8f8d9046d6eac29738d8521a48ca964361db2e107151cdedce9a0d62a7` | unchanged typed-unavailable stub/declarations |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission_test.go` | `d9334d3e1ea1ad61e0c89e68bb18de29ae77dfac33fd83db87dfed3a58e35ac8` | repaired limiter fixtures |
| `apps/godspeed-casework-go/internal/adapters/sfwp/client_run_get_admission_test.go` | `3466befd626c75785c03ce90c659351ce470f5499ecf69ee5fb8bd18aadcf6cb` | repaired client transport fixtures |

No production hook, limiter algorithm, constructor wiring, manager, public contract, or additional changed source path is present in the reviewed result. The earlier concern about function return type covariance remains resolved by the unchanged explicit timer adapter; the only production constructor still fixes two slots and a one-second cooldown and the test constructor normalizes nil clock/timer seams.

## Prior findings closed

- **Real no-spin release/reacquire:** `run_get_admission_test.go:163–207` obtains `permitA` and `permitB`, starts/finishes their write attempts, releases B, advances the controlled clock so both cooldowns are expired while A remains busy, then runs and joins a canceled same-run waiter. A second same-run waiter is joined after calling the actual `permitA.Release()`. There is no direct mutation of `busy` or `changed` in this fixture. The `timerCalls` total is checked after both waiter lifetimes (`:205–207`).
- **No-spin observation limitation is honest:** `assertNoAdmissionReevaluation` samples controlled clock calls during a bounded 20 ms no-change window (`run_get_admission_test.go:320–335`) and explicitly says it cannot prove absence of every possible CPU-spin implementation (`:322–324`). This supplements the timer-factory count; it does not overclaim a formal liveness proof. The production algorithm still requires its own source review for a blocking change/context wait without a default retry loop.
- **Queued waiter cleanup:** shared `queuedRunGetWaiter` cleanup cancels and bounded-joins each goroutine (`run_get_admission_test.go:211–285`); fatal/timeout paths reach `t.Cleanup`. Explicit cancel/deadline paths join their results. Result channels are buffered. This closes the previous unjoined-waiter gap.
- **Post-write cancellation barrier:** the cancellation/pool fixture waits for `requestReceived`, signaled only after the peer reads the full newline-terminated request (`client_run_get_admission_test.go:456–493`), before canceling. It therefore deterministically tests cancellation after a physical write, not cancellation racing the goroutine start.
- **Pool retirement at permit release:** the mock calls `onRelease` after recording release and outside its mutex (`client_run_get_admission_test.go:22–36`). The canceled case observes `Client.mu.TryLock`, `live == 0`, and no idle connection (`:515–544`); the healthy case observes lock acquisition, `live == 1`, `idle == 1`, and the expected connection present (`:202–241,532–544`). The callback reports through buffered channels and does not call fatal assertions. `TryLock` makes a lock-order defect a bounded failed observation instead of hanging the test. This distinguishes pool cleanup from merely observing socket `Close`.

## Fixture coverage and remaining boundaries

The fixtures cover production constructor dimensions; same-run exclusion and release wakeup; matching-slot reuse/no fallthrough; two simultaneous owned records and blocking a third; retention of unexpired idle/busy records; queued cancellation/deadline typed-unavailable results; prewrite release without cooldown; controlled finish-time cooldown; the mandatory expired-deadline/busy same-run case; run_get-only discrimination; the four physical retry attempts; Ask refusal retry and mutation refusal/no-resend behavior; payload/LF boundary ordering; partial/error payload and LF writes; blocked payload/LF writes; and callback join, connection retirement, and pool state before permit release.

Directly seeded slot state in matching/reuse/eviction/no-spin setup is arrangement for testing edge conditions, not a replacement implementation of acquisition. The no-spin transition itself now goes through the real permit API. No fixture asserts that the typed-unavailable stub is correct; the test comments accurately expect assertion failures until a separately assigned implementation exists.

The no-spin timer/clock probe is bounded sampling and cannot establish absence of arbitrary CPU spinning; this is disclosed in source and is acceptable for this fixture stage under the root supplement. The source has not been compiled, formatted, or run in this review. No actual RED is claimed. The focused assertion-RED compile may proceed only under the separately assigned compiler ownership and capture protocol. No production behavior, logical manager budget, hydration, SSE/UI, or T09 completion is approved.

Graft retrieval saved approximately 31,433 tokens (~$0.03) for this review turn.
