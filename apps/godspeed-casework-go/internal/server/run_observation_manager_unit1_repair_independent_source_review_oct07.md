# Run observation manager Unit1 repair: independent source review

Date: 2026-10-07  
Disposition: **REJECT for focused-RED readiness; no implementation or runtime approval**

## Scope and evidence

This is an independent source-only review of the repair against the original Unit1 assignment, revision 6 proposal and its addenda/corrections, response-line-cap erratum, root private-design authority, and the original independent source rejection. The repaired candidate is:

- `run_observation_manager.go`: SHA-256 `aad810b928590acc712eb5d5eeac15a5358a85e75d2a28d7acce0bd3c9fdce0f`
- `run_observation_manager_test.go`: SHA-256 `64dc5a074b926aed6da85f6070ad568dd61fec9b7c88d1f4e3d7265437af3f6a`
- builder repair record: SHA-256 `0c06a7de8d55a6399c53cc2d255b1a4039eef8cf5fa63fa33717653a850c6d23`

The original source/test pre-edit bytes were SHA-256 `ee7afab1b22c011cf79b602f415336d459fda346aad1bf0c942a4464326ff878` and `2b2d1fbcd3bfb7e7c27254673ca46b2deb30d9edd4ac862fa464e2a9b18334b6`. Root archived matching `.raw` copies under `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/`. The builder record's account that its first backup attempt used package-local `.txt` files because `.agents/` was read-only is historically accurate; the later root archive preserves the same original bytes.

No compiler, test, build, or Git command was run. This review does not grant permission to run them.

## What the repair got right

- The source diff removes the `afterAttach` test callback and adds no exported API, public configuration, dependency, or test-only production hook. The manager remains explicitly unwired; no implementation algorithm was smuggled into this test-first repair.
- The new Prepare-result-vs-read-start helper makes the unwired candidate fail semantically instead of hanging while waiting for a fake port callback.
- The cohort-capacity case exercises a different case key while a held poller is draining, then checks admission after the read is released and joined.
- The test-first boundary and no-runtime claim in the repair record are consistent with the source reviewed.

## Blocking findings

1. **Retry test self-deadlocks before releasing its fake read.** In `TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry`, the retry's second `run-second` trace call blocks on `allowRetry` (`run_observation_manager_test.go:817-818`). The test first waits for `secondReadStarted` (`:854`) and closes `allowRetry` only afterward (`:855`). That start channel is sent only by the first trace call (`:811-815`), so the retry cannot satisfy the wait. A future implementation that correctly starts the retry read will still hit the helper timeout; a fail-fast candidate may instead report the intended semantic RED. The retry needs a distinct signal emitted before its own wait, or should wait on its Prepare result with bounded time.

2. **Shared-initializer test does not establish that the second watcher attached before cancellation.** At `:910-917`, the second Prepare goroutine is launched and the first context is canceled immediately. There is no attachment handshake or observation of real registry/ref state. Scheduling may cancel/rollback the first initializer before the second Prepare attaches; the second caller can then perform a fresh read after release. The test's `readCalls == 1` assertion (`:941` onward) would fail for that valid ordering. The test must synchronize on the actual mutex-protected production ownership/ref representation before canceling, not infer attachment from goroutine launch or add a duplicate test-only counter. This requirement is explicit in the accepted shared-initializer design.

3. **Poller-capacity case does not keep all 16 entries occupied through the admission check.** `TestRunObservationManagerPollerCapacityCountsDrainingEntries` waits for only one later read, and only requires that it belongs to case A (`:699-708`). It never waits for the other 15 later reads. After A detaches, the one held read can keep at most one A poller draining; the other seven A pollers can retire, leaving room for case C despite the intended full-capacity assertion (`:735-738`). Keep all 8 A and 8 B poller reads held and accounted for, then verify A's eight cancellations and that B remains active before attempting C.

4. **New controlled-read tests can leak fake workers on assertion failure.** The cohort case's fake waits for `ctx.Done()` before `readRelease` (`run_observation_manager_test.go:563-568`); its deferred cleanup only closes `readRelease` (`:552-554`). The poller-capacity fake has the same ordering (`:674-679`) and its defer likewise only releases the second wait (`:659-662`). If setup/assertion fails before detach cancels the manager-owned context, the goroutines remain blocked. Cleanup must cancel/detach owned work as well as release the fake, with bounded joins; cleanup should remain safe on the intentionally unwired RED path.

## Disposition and next repair

The candidate is appropriately a private, unwired TESTONLY scaffold, but these defects mean its tests are not yet a reliable focused RED suite: one future-success path is structurally deadlocked, the shared-initializer assertion is nondeterministic, and the poller-capacity assertion does not establish its premise. Repair only the tests and any minimal real private ownership representation needed to observe actual attachment; preserve the no-hook/no-public-seam boundary. After a fresh independent source review, the operator may separately decide whether to authorize runtime verification. No behavior or implementation is approved here.
