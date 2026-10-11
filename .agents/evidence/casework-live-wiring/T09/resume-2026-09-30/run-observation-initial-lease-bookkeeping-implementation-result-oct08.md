# Initial lease bookkeeping implementation result

## Result and identities

Implemented the granted initial lease bookkeeping in the manager only. The
focused RED03 was accepted by root with all six capture artifacts independently
compared losslessly (root disposition: `initial-lease-red-root-disposition-oct08.md`).
The root retains compiler ownership; this builder did not run a compiler, test,
formatter, gate, or Git command.

| File | Before SHA-256 | After SHA-256 | Result |
|---|---|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `e2b7709410ff4766503a52580e0cdec8b9eaf189f695bbb75ae87b2dbbdf3285` | `2d01953469e3d7eedc5688ceb68c9e2d64aca86590251dcd11f3a59afbb3a6cd` | implementation |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go` | `08852d598e91264a81a745ac9ba08b31e7eeb5cdb60dd01324e578967f260f50` | same | frozen |
| `apps/godspeed-casework-go/internal/server/run_observation_delta.go` | `5241e656d65f86a315e37df44b48d5eaae865c92cf80cc306c9bf4fc0fd7e128` | same | frozen |
| `apps/godspeed-casework-go/internal/server/run_observation_delta_test.go` | `48c6d28e658b8bd4e8fd3c6650ff26c73b0c791f35feff83d056ff8f501c89ed` | same | frozen |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` | same | frozen |

## Changed behavior and source anchors

- `run_observation_manager.go:123-128`: every lease initializes its own empty
  `map[runObservationPollerKey]runObservationWatermark`.
- `:397-436`: the actual list result is recorded as `unavailable` or `complete`
  under `m.mu`; the same outcome is passed into initial DTO construction.
- `:537-553`: before capturing a retained state for initial projection, the
  manager mutex protects checks for active cohort, nonstopping/nondraining
  lease, exact manager entry, exact lease forward membership, reverse ref,
  running phase, exact state key, and `retainedCurrent`. The accepted immutable
  state is converted and seeded in that same critical section, before initial
  DTO construction or aggregate hydration/pruning.
- `:678-694`: the O(1) conversion copies only scalar highest ordinal, standing
  enums, and the complete generation/total/retained/omitted/truncated window.
  It copies no frames, pointers, event IDs, or identity ledger.
- Existing final authorization and disclosure gates remain after hydration;
  authorization and guard calls remain outside the manager mutex. No recapture
  after hydration, public contract, dependency, budget, notifier, wake,
  operation-wait-group, or drain behavior was added.

Material deviations: none. Tests after the implementation grant remain
unverified by this builder; root owns the serialized focused GREEN and required
follow-up gates. This result makes no GREEN or completion claim.

## Fresh implementation grant

```text
Fresh implementation builder, tightly bounded unit in /home/sprime01/projects/sea-rs. Read repository/scoped AGENTS.md, global routing, updated .agents/AGENTS.md and ONLY latest CURRENT_STATUS.yaml line. Original FULL task BASE/run-observation-initial-lease-bookkeeping-assignment-oct08.md must be read and reproduced in result; BASE=.agents/evidence/casework-live-wiring/T09/resume-2026-09-30. Read original TDD result/correction/two-compile-fix-result and independent-source-review. Root actual focused RED03 compiled three tests and all fail on absent listState; hydration test passed real pruning assertions before failure. ALL6 captures independently root lossless compared, runtime /tmp/initial-lease-red03-26e1v_tb and BASE/initial-lease-red03-{command,preflight,preflight-exit,stdout,stderr,exit}-oct08.raw.json. Prior01 cache setup failure and02 unrelated compiler preflighthold NOT RED. Implement ONLY initial lease bookkeeping described fullassignment: initialize private perlease watermark map; capture actual complete/unavailable list outcome under manager mutex; seed all scalar fields from actual accepted immutable capture before DTO/hydration under SAME capture lock with exact manager-entry+forward+reverse memberships, activecohort/nonstop/nondrain/current-key-phase validity. Reuse scalar conversion if natural. Keep authorization outside mutex and final authorization/disclosure gates. No recapture afterhydration, no frames/pointers/eventID/extraledger copied. Allowed source ONLY manager.go; freeze manager_test.go08852d59, delta/source5241e656/tests48c6d28e, workerb0fde3fe and all others. No tests modifications unless root explicitfreshgrant. No Next/notifier/wake/operations/drain redesign/public/deps/budgets. Native apply_patch ONLY for source/result; read source first and Graft scope. No compiler/test/gate/Git/status. Produce immutable new result with FULL original instructions + this grant and actual final identities/materialdeviations and anchors. Go formatting derive nonmutating output then nativepatch, never gofmt-w. Escalate ambiguity/incidental defects to root; do not broaden. Independent critic will review and root serialize actual GREEN. User wants minimum cost so don't wrestle extensively; ask root semantic question promptly.
```

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
