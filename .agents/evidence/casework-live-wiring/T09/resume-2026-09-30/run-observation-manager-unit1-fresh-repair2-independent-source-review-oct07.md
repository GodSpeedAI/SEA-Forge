# Run observation manager Unit1 repair 2: independent source review

Date: 2026-10-07  
Disposition: **REJECT focused-RED readiness pending cleanup proof; no implementation or runtime approval**

## Scope and provenance

Reviewed the original Unit1 assignment, repair 2 instructions and record, repair 1 independent rejection, this reviewer's repair 1 findings, revision 6 proposal/addenda/corrections and cap erratum, and root's private ownership decision. Current candidate identities:

- `run_observation_manager.go`: SHA-256 `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`
- `run_observation_manager_test.go`: SHA-256 `6bc6666ee176a56e233636bca0d72fcda439e430a03c0642e4fd2ad695af826a`
- repair 2 record: SHA-256 `2cb8d19a1ee3597744d1e4effbc3b44f5e504d7343135620e50022a8ba1f285b`

No tests, compiler, formatter, scanner, build, Git, or runtime command was run.

**Preservation deviation:** The required immediate pre-edit repair 1 source/test payloads (`aad810...` / `64dc...`) were not preserved. The recovery note explicitly records that the combined earlier read was truncated and that exact bytes could not be reconstructed. The available `ee7...` / `2b2...` backups are the original baseline, not repair 1's immediate preimage. I therefore did not claim or perform an exact repair-1-to-repair-2 diff. The current full source and tests, their exact hashes, the historical repair 1 review, and the repair 2 recovery note were reviewed. This provenance gap remains a recorded deviation, not a fabricated backup.

## Findings

The four prior findings are materially repaired:

1. Retry now emits `retryReadStarted` before blocking on `allowRetry`; the test waits on that distinct signal before releasing it (`run_observation_manager_test.go:929-947, 979-987`).
2. Shared initialization waits for two refs in the manager's mutex-protected `pollers[key].refs`, with `runtime.Gosched` and a bounded timer rather than a launch-order assumption (`:208-240, 1043-1055`). Those are real private ownership fields (`run_observation_manager.go:39-67`), not duplicate counters or callback hooks. The second caller is canceled only after this observed membership.
3. Poller capacity waits for all sixteen unique exact `case/run` keys from A and B, then checks A's eight cancellations, B remains active, and each entry still has one ref and an unclosed `workerDone` before attempting C (`run_observation_manager_test.go:769-864`).
4. The affected fixtures register cleanup that cancels supplied caller contexts, starts bounded manager stop/drain, releases fake reads, and waits for the stop result (`:177-205`, with registrations at `:546`, `:644`, `:755`, `:959`, `:1034`, and `:1106`). This is a substantial correction to release-only cleanup.

The source remains an unwired stub. The mutex, exact-key map, reciprocal lease/poller references, readiness channel, worker context/cancel, and worker completion channel match the authorized minimum real ownership scaffold. No public seam, callback, test-only counter, or manager algorithm is apparent in the current complete source. This does not approve production implementation.

## Remaining blocker: cleanup can accept an unjoined live manager

`cleanupRunObservationManager` accepts every `KindUnavailable` result from `stopAndDrain` without checking whether the manager still owns poller entries or worker goroutines (`run_observation_manager_test.go:190-201`). That exception is needed for the intentionally unwired stub's expected semantic RED, but it also treats an unavailable stop that returns with live ownership as successful cleanup. It neither checks that `manager.pollers` is empty after an unavailable result nor verifies any captured `workerDone` channels are closed. A failed assertion after real workers have started could therefore leave goroutines running while cleanup reports no error. Keep the stub path clean by allowing the typed unavailable result only when there was no owned work to join; when entries exist, require a successful bounded stop/join and verify ownership has drained. This can inspect the actual map/channels under the existing mutex and needs no extra production hook.

The shared-ref wait currently accepts `refs >= want` rather than exactly two (`:222-226`). Only two callers are launched in this fixture, so this is not a current blocker; exact membership count would make the barrier contract tighter and better detect accidental duplicate attachment.

## Disposition

Repair 2 fixes the four rejected fixture defects and uses the approved private ownership representation. I still reject focused-RED readiness until cleanup distinguishes the no-work stub from a failed drain with live ownership. The missing immediate-preimage archive remains explicitly disclosed; this review is based on the current full source and historical review evidence, not an exact prior-candidate diff. No runtime, compiler, test, or implementation authorization is granted.
