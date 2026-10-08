# Independent architecture review: poller read-start testability

Date: 2026-10-07  
Verdict: **Approve the mutex-linearized read-eligibility design, subject to the
two explicit test/protocol clarifications below.** This is a source-only
architecture review. It authorizes no source or fixture edits, compiler,
tests, or runtime claim.

## Reviewed records and source identities

- Root proposal: `manager-read-start-testability-root-decision-oct07.md`, SHA-256 `e1b1db0e7155ff162869c54845d0251783c72e4ed2837f3961ac91e58df602ac`.
- Unit 1 assignment: `run-observation-manager-unit1-original-assignment-oct06.md`, SHA-256 `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`.
- Lifecycle preregistration: `apps/godspeed-casework-go/internal/server/run_observation_manager_unit1_lifecycle_implementation_preregistration_oct07.md`, SHA-256 `f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5`.
- Revision 6: SHA-256 `60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`.
- Revision 6 addendum: SHA-256 `9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`.
- Retention/initializer corrections: correction 1 SHA-256 `449c73301de85794e9aec3093e82a88fdba8c60e46f207cd30454a15f1ce0524`, correction 2 SHA-256 `3494e3de99b2f5d3fe725a85971dccd0d70cef226da257ce237cce7fba2d72e1`, correction 3 SHA-256 `b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`.
- Failure-fixture assignment: SHA-256 `d6f5f4c922c78a6c7f40215609b6199b4ecbad32c28f0b2b38002779bed6ad3a`.
- Failure-fixture independent review: SHA-256 `dc37ae4c4170c7be0ea50a2235c55e8b09d21c230dbacda2ebf5d760e3cc4a23`.
- Initial-failure clarification: SHA-256 `34750ceb5e7dc99b0a7c3755c514f6a7fbdb6fd31ac60d99833fad181ed117c2`.
- Ref ownership decision: SHA-256 `5b9888e5a7212b8d0696f790822f08e718dd03a71f51b28c2bec2ea0098f8df4`.
- Current scaffold [run_observation_manager.go](/home/sprime01/projects/sea-rs/apps/godspeed-casework-go/internal/server/run_observation_manager.go): SHA-256 `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`.
- Frozen fixture [run_observation_manager_test.go](/home/sprime01/projects/sea-rs/apps/godspeed-casework-go/internal/server/run_observation_manager_test.go): SHA-256 `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.
- Port contract `internal/ports/run_trace.go`: SHA-256 `88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5`.

## Assessment against the contract and scaffold

The proposal resolves the review's missing synchronization point with a real
manager transition, not a test callback. The worker checks admission, exact
entry identity, phase and eligible refs under the manager mutex before each
initial or recurring read. Stop and last-ref release use the same mutex. This
gives a clear linearization order: if teardown transitions first, the read
claim is refused; if the worker's claim wins first, the worker owns the
resulting work through its completion and JOIN. The actual `RunTracePort`
callback remains outside the mutex.

That model is consistent with Unit 1 and revision 6. The original assignment
counts actual logical `ReadRunTrace` starts; the preregistration repeats that
counting actual call starts and requires all adapter calls, cancellation,
waits and JOIN outside the lock. The port contract is an ordinary method call
(`internal/ports/run_trace.go:6-8`), so the manager cannot hold its state mutex
through physical callback entry without violating the no-I/O-under-lock rule.
No reviewed normative text requires a wall-clock guarantee that the callback
body has entered before Stop's wall-clock invocation. A shared mutex therefore
provides the appropriate concurrent-operation ordering.

The new transition is also required by the source: the current poller scaffold
has only `ready`, context/cancel and `workerDone` fields
(`run_observation_manager.go:73-80`), and `prepare`, `stopAndDrain` and
`detachAndDrain` are unwired stubs (`:125-153`). The frozen tests prove an
already-started read is cancelled and joined before retry (`run_observation_manager_test.go:1049-1134`),
shared initializer survival after the read has started (`:1136-1216`), and
capacity retention while a cancellation-resistant read is held (`:1218-1270`).
They do not prove a pre-read stop decision. The requested private eligibility
method and worker call sites are thus necessary new lifecycle behavior, not a
test-only expansion.

## Required semantic clarification: a claim is not an invocation count

The proposal correctly says `ReadsAttempted` increments only when the port
method is actually invoked and that cancellation can prevent a claimed call.
These facts must remain distinct:

1. The mutex-protected eligibility claim is the **read-start linearization
   point**, not evidence that `ReadRunTrace` was invoked.
2. If Stop wins before that point, the worker rejects the read, retains nil
   current, resolves ready once with existing invalid/unavailable code 4 and
   stopping/draining phase 2/3, and returns; `ReadsAttempted` remains zero.
3. If the claim wins but the manager-owned context is canceled before actual
   port invocation, the port is not called and the actual invocation count
   remains zero. Since this is still a no-first-read stop outcome, resolve the
   same code-4/phase-2-or-3 initializer and ready signal, then let the actual
   worker close `workerDone`; capacity remains owned until JOIN.
4. If the callback is entered, it counts as an actual invocation even if Stop
   wins in wall-clock time before or during its callback body. The claim
   linearized first, so teardown must retain ownership through the callback's
   actual return/client retirement and worker JOIN. No subsequent claim may
   pass once Stop's state transition wins.

This is a clarification of the count and cancellation rules already in the
assignment/preregistration, not permission to count eligibility checks as
reads. It also makes the previously unspecified claimed-but-canceled-before-
invocation case deterministic and preserves the root's stopped-before-first-
port-call code/phase contract.

## Required testability clarification

The proposed tests should use the mutex-linearized transition as the ordering
point and make their claims precise:

- Stop-before-claim: establish the real Stop/admission transition first, then
  exercise the same production eligibility method and the real worker's
  stopped-initializer path. Assert denial, no port invocation, code 4/phase
  2-or-3, exactly one ready resolution, actual worker completion, and zero
  ReadsAttempted.
- Claim-before-Stop: establish a permitted claim on a registered referenced
  running entry, then Stop. Do not assert that the callback must have entered
  before Stop's wall-clock call. If it enters, hold it through actual return
  and JOIN; if cancellation prevents entry, assert zero invocations and the
  stopped-before-first-read result. In either branch, prove no later claim is
  accepted and capacity is not released before the worker's actual completion
  and JOIN.
- Positive eligibility must use a valid registered entry/ref/phase so an
  always-deny method cannot satisfy the negative case. Keep the existing
  all-eight-start-before-waiting fixture unchanged.

The root proposal says tests may exercise the production transition directly
and separately says to exercise the real worker path. Before a lifecycle
fixture is called deterministic, its design must show exactly how the Stop
state transition can be ordered before a real worker reaches its first gate,
without a test hook, artificial port barrier, scheduler sleep or racy poll.
This review approves the architecture, but does not assume that the current
unwired scaffold already provides that scheduling path. A synchronous
production stop-state transition reused by `stopAndDrain`, or an equivalent
source-grounded path, is acceptable; test-only field/callback injection is not.

## Material differences and limits

- The previously ambiguous phrase “stop before first read” is refined to
  “Stop wins before the worker's manager-mutex read-start eligibility claim.”
  It is not a promise about wall-clock callback entry after a prior claim.
- Actual invocation counts remain tied to `ReadRunTrace` entry, while
  eligibility claims carry ownership but do not themselves increment counts.
- A claimed call canceled before entry is specified as a no-first-read
  stopped initializer with code 4, phase 2 or 3, nil current, one ready
  resolution and zero invocation count; worker completion/JOIN still governs
  capacity.
- Root's new transition/method and worker checks are substantive production
  lifecycle additions beyond the current scaffold. They are justified by the
  original assignment and prior independent fixture review; this document
  approves their design only, not their source implementation.
- All current bounds, code ranges, helper semantics, exact per-Prepare lease
  ownership, per-target notifications, public/private boundaries, and frozen
  fixtures remain unchanged.

No source, test, compiler, formatter, scanner, Git, or runtime operation was
performed. The result is architecture review only and grants no fixture or
implementation release.
