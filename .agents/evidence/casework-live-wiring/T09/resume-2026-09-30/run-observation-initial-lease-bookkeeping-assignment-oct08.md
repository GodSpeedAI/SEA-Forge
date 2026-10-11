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
