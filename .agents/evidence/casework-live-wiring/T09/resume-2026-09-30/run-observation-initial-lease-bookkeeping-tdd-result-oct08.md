# Initial lease bookkeeping TDD source result

## Scope and result

This is a source-only TDD preparation result. It does not implement lease
bookkeeping and does not claim any test, compiler, or runtime outcome.

Only the authorized manager and manager test files changed:

| File | Before SHA-256 | After SHA-256 |
| --- | --- | --- |
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57` | `e2b7709410ff4766503a52580e0cdec8b9eaf189f695bbb75ae87b2dbbdf3285` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go` | `cf7188c23e5bad5f7b0f093561bf85e1e273b48c721a677084429e8038f0ba86` | `5e8f2728278590d6364d51b464146068baebc92b41f602d79c84eb7410c481eb` |

Manager change: `runObservationLease` now declares `listState contract.RunTraceListState`
and `watermarks map[runObservationPollerKey]runObservationWatermark` at lines 84–85.
No constructors or lifecycle paths initialize or populate either field.

Test changes:

- `TestRunObservationManagerPrepareReturnsExactEmptyInitialEventAndLease` checks,
  under `manager.mu`, for complete list state and a nonnil empty watermark map.
- `TestRunObservationManagerRefusedListReturnsUnavailableDTOAndEmptyLease` checks,
  under `manager.mu`, for unavailable list state and a nonnil empty watermark map.
- `TestRunObservationManagerPrepareSeedsInitialWatermarksBeforeHydrationAndKeepsLeaseMapsSeparate`
  uses actual `Prepare` calls with eight accepted terminal captures of 1,024 valid
  frames each. The event assertion requires aggregate frame pruning; expected
  scalar watermarks are independently calculated from generation 1, ordinal
  1,024, complete retained window counts, and the accepted standings. A second
  real Prepare checks that adding a distinct key does not alter the first lease's
  map. Both leases use normal detach/drain cleanup.
- The manager test import adds `strings` for deterministic long opaque event IDs.

The six frozen implementation/test inputs remain byte-identical:

| File | SHA-256 |
| --- | --- |
| `apps/godspeed-casework-go/internal/server/run_observation_delta.go` | `5241e656d65f86a315e37df44b48d5eaae865c92cf80cc306c9bf4fc0fd7e128` |
| `apps/godspeed-casework-go/internal/server/run_observation_delta_test.go` | `48c6d28e658b8bd4e8fd3c6650ff26c73b0c791f35feff83d056ff8f501c89ed` |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` |

No other source was edited by this bounded task. Graft was attempted first but
reported no graph/manifest; source anchors were verified directly. No tests,
compiler, formatter, scanner, build, or Git command was run. The test changes
remain unverified and are intended for independent source review, followed by
the separately authorized actual focused RED. No source change after these
hashes is included here.

## Full original assignment

```text
# Initial lease bookkeeping bounded builder assignment

Authority: full revision6 manager proposal/addendum/retention correction3;
root Next integration decisions and independent reviewf261af24; separate private
policy operator receipt; accepted pure projection24c7f2c5; decompositionf36f6271.
Read all these artifacts and the nearest instructions before changing source.

## Scope and behavior

Only change internal/server/run_observation_manager.go and its existing
run_observation_manager_test.go in the Go app. Add private lease listState of
contract.RunTraceListState and a private per-lease map from exact poller keys to
runObservationWatermark. No public interface/schema, dependency or retention
budget change. Complete-empty list is complete, failed list is unavailable;
empty attachment maps cannot distinguish those outcomes.

For each actually accepted immutable retainedCurrent capture, seed a scalar
watermark with captured HighestOrdinal, execution/settlement/observation
standings and full generation/total/retained/omitted/truncated window. Seed
inside the same manager mutex critical section as capture, before any initial
DTO construction or aggregate hydration/pruning. Do not copy frames, pointers,
event IDs or another identity ledger into lease bookkeeping. Do not recapture
after hydration. Distinct leases own distinct maps.

Root correction to the decomposition's source description: the current initial
capture loop does not yet check all manager/forward/reverse memberships there;
the final disclosure gate does. Before seeding, require exact current manager
entry, exact lease forward reference and existing reverse reference, active
cohort, nonstopping/nondraining state and the existing current/key/phase checks.
Keep the final disclosure and authorization gates intact. Authorize/guard work
stays outside the mutex. No field becomes an authority token.

## TDD phase first, no implementation release yet

Read both source files and nearby tests first. Add the two field declarations
only as compile scaffolding, with no initialization/seeding/list-state logic.
Extend the existing complete-empty and refused-list tests with locked lease
bookkeeping assertions. Add a deterministic test through actual Prepare that
proves accepted captured scalar baselines survive real initial hydration pruning:
assert actual DTO frames are pruned and compare every scalar watermark field
with the accepted pre-hydration capture. Use existing fake/read/manager patterns,
no sleep-based scheduling or manual fake post-Prepare map population. Assert
separate lease maps where feasible without expanding the test unit. Preserve
all prior assertions and perform ordinary detach/actual drain cleanup.

Use fitting per-poller synthetic captures that cross the aggregate hydration
boundary; do not rely on a malformed over-limit retained state or impossible
enum. If an additional deterministic hook is needed, stop and propose it to
root rather than broadening production code. Keep all unrelated files frozen,
including delta source/tests and worker.

Apply native patches only, including formatter-derived text if needed. No
compiler/test/gate/Git mutation. Produce a source-only result containing this
full assignment, exact pre/post identities and changed lines. An independent
critic reviews the original assignment plus result before a serialized actual
focused RED. Implementation initialization/seeding requires a fresh grant
after expected runtime failures are captured and root-compared.

## Subsequent implementation acceptance

After RED, implement only the described fields/map/list outcomes/initial scalar
seeding; a small O(1) private conversion helper is allowed. Independent critic
must review all original requirements/material differences and independently
run focused race, canonical Go and required integration regression checks with
direct child status capture. One compiler owner, RAM and swap guards; immediately
archive all actual six capture files, root decode/compare before next gate.
No Next method, notifier, wake, operation wait group or drain redesign belongs
to this unit. Those remain explicit later units. No T09 settlement follows.
```
