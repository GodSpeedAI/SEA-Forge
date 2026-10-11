# Manager read-start testability proposal — source-only

Date: 2026-10-07. This proposal follows root's architecture decision but grants no source or test release.

## Governing decision and source facts

Root's `manager-read-start-testability-root-decision-oct07.md` (SHA-256 `e1b1db0e7155ff162869c54845d0251783c72e4ed2837f3961ac91e58df602ac`) selects a real private read-eligibility transition protected by the manager mutex and called by the sole production poller worker immediately before every trace read. Its direct independent review `run-observation-manager-failure-fixture-independent-review-oct07.md` (SHA-256 `dc37ae4c4170c7be0ea50a2235c55e8b09d21c230dbacda2ebf5d760e3cc4a23`) rejected the earlier one-new-test-file plan because the current scaffold has no worker or pre-read state to synchronize against.

The existing manager (`run_observation_manager.go`, SHA-256 `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`) declares only `mu`, `pollers`, list/trace/session/guard dependencies, the exact key, and the poller's `refs`, `ready`, manager-owned `ctx/cancel`, and `workerDone` (`:48-87`). `prepare`, `stopAndDrain`, and `detachAndDrain` remain unwired stubs (`:125-153`). There is no stop bit, lifecycle phase/result/current pointer, worker method, read eligibility transition, or worker completion path. The current manager fixture (`run_observation_manager_test.go`, SHA-256 `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`) has only post-call entry barriers: canceled partial Prepare (`:1049`), shared initializer survives caller cancellation (`:1136`), drain-timeout waiting for an already-entered read (`:1218`), and all 16 initialization reads before read release (`:555-639`).

## Minimum production boundary

Propose a genuine private method, implemented in the one worker path and invoked immediately before every initial and recurring `ReadRunTrace` call:

```go
func (m *runObservationManager) claimPollerReadStart(entry *runObservationPoller) bool
```

It acquires/releases `m.mu` itself and allows the single worker to proceed only if the manager is not stopping, `m.pollers[entry.key]` is exactly `entry`, the entry phase permits reads, and at least one eligible lease ref remains. It performs no I/O, cancellation, channel signal, or wait. It does not increment `ReadsAttempted`, reserve another worker, add an initializer token, or mutate a read-in-flight counter. The sole-worker invariant plus `workerDone` owns the claimed work. The actual worker checks its manager-owned context outside the lock immediately before the port call, invokes the port outside the lock, and records an attempt only at the actual call expression. It reuses the existing `RunTracePort` and physical admission/retirement path.

The minimum private state needed in the existing manager source is:

- `runObservationManager.stopping bool`, read/written under `mu`.
- `runObservationPoller.phase runObservationPollerPhase`, `.initResult runObservationInitializerResult`, and `.current *runObservationRetainedState`, each transitioned under `mu`. Reuse the fixed enums and image seam already defined in `run_observation_poller_image.go`; do not add lifecycle fields to its JSON wrapper.
- Existing `ready`, `ctx/cancel`, `workerDone`, `key`, and per-lease `refs` continue to own signal, cancellation, exact membership, and the sole worker completion. The worker alone resolves initial `ready` once outside the mutex and closes `workerDone` after actual port return/retirement. No extra `sync.Once`, read-start counter, callback, generic gate, or scheduler field is needed if ownership remains sole-worker.

Add one real worker method in new `run_observation_poller_worker.go`, e.g. `func (m *runObservationManager) runPoller(entry *runObservationPoller)`, that calls `claimPollerReadStart` and performs its actual `ReadRunTrace`/result path. This method is the production boundary a same-package fixture can call directly. It must not be a test-only trampoline or fake callback. If root's lifecycle design chooses a different name or places the worker in the manager file, review that complete worker path against this contract before release.

If `claimPollerReadStart` refuses before the first read because Stop/final eligible-ref release already won `mu`, the actual worker stores no current value, sets existing initializer code 4 and phase 2 (stopping) or 3 (draining), resolves `ready` exactly once outside the lock, and completes its owned worker path. This is a failed Prepare, not successful A. A real first `ReadRunTrace` error is code 2/read-unavailable: the selected row is A, each otherwise-successful Prepare returns nil error and its own nonnil cohort lease, owner `ReadsAttempted=1`, shared waiter `ReadsAttempted=0`, with no duplicate read. Capture time and whole DTO may differ per Prepare.

## Minimal revised future file boundary

The earlier failure-fixture assignment only named a new test file and cannot compile or prove this boundary. Before any edit, a new root release and independent review must explicitly revise the source paths to:

1. Modify existing `apps/godspeed-casework-go/internal/server/run_observation_manager.go` only for the required private manager stop/lifecycle state and its real mutex-protected transition/stop coordination.
2. Add `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` for the genuine sole worker, actual read path, pre-read transition call, initializer result/readiness, and actual `workerDone` completion.
3. Add `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go` for deterministic tests below.

The manager source is the explicit exception to its earlier frozen `fa1601...` identity for this separately reviewed unit. Keep the existing manager fixture `run_observation_manager_test.go` byte-identical at `af2df...`; add no fields or hooks there. Keep the pure retained helper `run_observation_retained_version.go` (`215758...`), helper fixture (`34df...`), policy fixture (`4c70...`), encoder source (`3dba...`), and encoder fixture (`cf501...`) frozen. Their current test result remains with root's serialized renderer; this proposal makes no verification claim.

The fixture should exercise the production path without callbacks or scheduler tricks:

1. **Stop wins before eligibility:** create/register one initializing poller with its actual worker not yet launched; start the actual `stopAndDrain` path and synchronize only on its real manager-stopping transition under `mu`; then invoke the actual `runPoller(entry)` method. Assert `claimPollerReadStart` refusal, no `ReadRunTrace` invocation, code 4, phase 2/3, nil current, waiter readiness, and worker completion. Ensure the stop owner joins the worker outside `mu`. Separately directly call the eligibility transition for a registered, referenced, running entry and assert it permits; a universal false return cannot satisfy both tests.
2. **Claim wins before Stop:** order `claimPollerReadStart` success first, then Stop; the read is owned until its actual return and worker join. Use the existing channel-controlled port fake after entry to hold the real invocation, and prove reservation/capacity remains occupied until return/retirement plus `workerDone` JOIN. This is separate from the stop-wins/no-call case.
3. Keep global Stop across distinct pending Prepare leases and one-Prepare rollback while a distinct authorized lease survives as separate cases; preserve one exact ref-release owner per lease. Keep shared failed-first-read successful-A/per-Prepare count semantics. Keep the actual oversized-key manager A/no-entry/no-ref/no-read test. Retain existing all-eight-selected-start-before-any-wait fixture unchanged.

The first case's setup is deterministic because the manager's actual stop transition is ordered before the actual worker's eligibility method, with no read port barrier pretending an unstarted call. If stopAndDrain cannot leave the reserved entry available for its actual worker to complete/join, the production transition/setup contract must be revised before fixture work.

## Stop/read wording and linearization limit

Root's decision makes `claimPollerReadStart` the read-start eligibility linearization point, not the wall-clock timestamp at which `ReadRunTrace` begins. If Stop/final-ref-release wins `mu` first, the transition rejects and the worker makes no port call. If the worker wins the transition first, that claimed operation is owned through actual return/JOIN; cancellation checked after unlock may prevent the port call from being invoked. Because the contract forbids I/O and cancellation under `mu`, a worker paused after winning eligibility can physically enter `ReadRunTrace` after Stop is requested but before cancellation takes effect. In that ordering Stop did not win the eligibility transition.

Several records say “starts no trace read” or bar “every future `ReadRunTrace` start.” Read literally as “no port method entry at any wall-clock time after Stop begins,” that cannot be guaranteed by a pre-call lock transition followed by unlocked I/O/cancellation. The new root decision resolves the race only if these phrases mean: Stop winning the eligibility mutex transition prevents the port call; a prior eligibility winner remains owned and may either be canceled before invocation or complete under normal join rules. This interpretation must be explicitly accepted in the source-review record before implementation; do not silently claim a stronger timestamp guarantee.

## Separate primitive extraction dependency

Graft's type trace found `runObservationPollerKey` at `run_observation_manager.go:61-65` and no indexed callers; direct `rg` over the pure helper/encoder and their three fixtures confirms the sole dependency is its exact three-string private struct. `run_observation_retained_version.go` uses it in retained state/candidate/build/marshal; `run_observation_poller_image.go` uses it in the wrapper encoder; `run_observation_retained_version_test.go`, `run_observation_retained_policy_test.go`, and `run_observation_manager_retained_image_test.go` use it in fixtures/cases. No references to manager construction, authorization, lease, poller, or stop methods occur in those five pure files/tests. The helper test supplies `retainedTestKey`, `retainedTestState`, and clone helpers to policy and encoder tests.

After the serialized encoder GREEN source is frozen, a separately explicit, independently reviewed primitive checkpoint can extract exactly the existing declaration (fields remain lowercase `caseID`, `runID`, `planItemID`, all `string`) into new `run_observation_key.go` and remove only that duplicate declaration from `run_observation_manager.go`. Those are the two files in that distinct extraction unit. Leave helper/encoder implementations and all three fixtures byte-identical. This permits the pure primitive source/test closure to no longer depend on an unpublished manager scaffold; it does not authorize a lifecycle edit or make the manager fixture pass. No compile, formatter, vet, race, or Git action was run for this source-only dependency audit.

## Evidence limits and deviations

This is a concrete testability/source-boundary proposal, not implementation approval. It adds no public API, dependency, fake callback, test-only field, generic token, phase code, or retained-image field. It narrows “no read after Stop” to the explicit mutex-ordered eligibility semantics chosen by root and separately preserves actual-return/JOIN for a prior winner; if that is not the intended meaning, the normative conflict remains unresolved. No source/test/compiler/scanner/Git operation was performed.
