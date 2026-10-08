# Run observation manager failure-fixture assignment review — 2026-10-07

## Verdict

**Do not release the fixture edit yet.** The bounded assignment preserves the original lifecycle contract and correctly calls out one unresolved point: its deterministic Stop-before-first-read case has no currently available synchronization seam. The fixture can be released only after root chooses a genuine production transition that the test can order, or revises that case's scope. Do not add a test-only field, callback, or timing race to bridge the gap.

## Scope and source evidence

The reviewed assignment is `run-observation-manager-failure-fixture-testfirst-assignment-oct07.md`. Its cited governing sources include the Unit 1 original assignment, lifecycle preregistration, revision 6 and addenda/corrections, the initial-failure clarification, the signal/ref-ownership decision, and the independently approved per-Prepare lease-cardinality supplement. The assignment keeps lifecycle implementation unreleased and proposes only a new test file.

Current [run_observation_manager.go](/home/sprime01/projects/sea-rs/apps/godspeed-casework-go/internal/server/run_observation_manager.go:67) declares `ready`, manager-owned worker context/cancel, and `workerDone`, but these are scaffold fields only. Its `prepare`, `stopAndDrain`, and `detachAndDrain` methods remain unwired stubs at lines 123–153. There is no worker-start claim, pre-read stop check, or completion boundary in the current scaffold for a fixture to synchronize with.

The frozen tests prove neighboring but different behavior. [run_observation_manager_test.go](/home/sprime01/projects/sea-rs/apps/godspeed-casework-go/internal/server/run_observation_manager_test.go:1049) blocks after the fake `ReadRunTrace` has started, then cancels and proves read return/JOIN before retry. Lines 1136–1216 prove a shared initializer survives initiating-Prepare cancellation after its read starts. Lines 1218–1270 prove timed-out Stop leaves admission closed while an actual port call is held and requires that call to return before Prepare rollback completes. These cases do not establish Stop winning before the first port call.

The preregistration requires no lock-held I/O/cancel/wait/JOIN and says worker completion follows actual trace-call and retirement return (sections “Shared initializer and poller worker” and “Required ownership and behavior invariants”). The revision 6 stop correction also requires the safe stop decision before joining. Those requirements support the assignment's need to distinguish a read whose start was claimed before Stop from one that Stop prevented.

## Case-by-case assessment

- Case 1 correctly treats shared initial read failure as successful Prepare with nonnil per-caller leases and distinguishes owner versus waiter read counts. Its cleanup remains exact-ref and JOIN ordered.
- Case 2 correctly separates Stop's per-lease ref removals from one-Prepare rollback and preserves the survivor's ref. The root's sole-ref-release ownership decisions are reflected.
- Case 3's in-flight/cancellation-resistant read and capacity-through-JOIN portion is supported by the existing channel-controlled fake and frozen drain test. The pre-first-read portion is **not yet testable deterministically** against the current scaffold. A signal emitted by the fake `ReadRunTrace` can only prove entry after the call has started; then Stop is no longer the winner before first read. Starting Stop without an ordering signal leaves a scheduler race. Directly manipulating `mu` cannot prove a worker reached the gate, and polling state does not prove that it has not already claimed or started the call.
- Case 4 correctly requires manager-level admission and A/C accounting evidence; the pure serializer alone cannot prove it.
- The assignment preserves the frozen all-eight-start-before-waiting test and its explicit prohibition on serializing away the behavior.

## Required resolution before release

Root should choose one of these source-grounded resolutions before authorizing the fixture file:

1. Define an actual private production transition that atomically claims a read start under the manager mutex, and make that transition directly exercisable by the test. The test can then perform Stop before the claim, assert refusal/no port call/one ready resolution, and separately prove that a claim made first counts as an in-flight read that must return and JOIN. This is a protocol operation, not an injected callback or test-only behavior.
2. If no such production transition fits the approved lifecycle architecture, narrow or defer the pre-read race case and record the remaining proof gap explicitly; do not call the full case-3 matrix ready.

Either choice must preserve reserve-all-selected-initializers-before-waiting, the exact fixed stopped initializer code/phase, ready resolution outside the mutex, one worker join owner, and capacity retention until actual port return plus `workerDone` JOIN. This review approves no implementation, fixture file, compiler, test, or runtime claim.
