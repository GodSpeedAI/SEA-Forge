# Independent source review: Unit 1 authorization retry fixture correction

Date: 2026-10-07  
Disposition: **APPROVE as future-compatible Unit 1 test-first fixture only; no runtime or lifecycle approval**

## Scope and identities

Reviewed the full Unit 1 original assignment, revision 6 proposal and root private-design decision, the list-refusal assignment/result and independent rejection, the fresh authorization-retry assignment/result, current manager source/test, and exact pre-edit test archive.

Current source identities:

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `run_observation_manager.go` | 5,488 | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `run_observation_manager_test.go` | 55,815 | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |

The root's exact UTF-8 JSON preimage for the test decodes to 54,907 bytes and
SHA-256 `87be7ae28010132c3e5ddbaa9ad0469ed5891061b88a005f86d8d70b977124c0`,
matching the recorded immediate pre-edit identity. A direct unified diff
against that decoded preimage is confined to removing the now-unused `strings`
import and the named authorization-retry test. The manager source remains
byte-identical. The earlier fixture correction and this correction preserve
their separate records; no immutable review or result file was edited.

## Previous finding addressed

The previous review rejected the corrected-claim retry because it expected the
unwired stub's literal error, zero wrapper, nil lease, and no list call. The
current test keeps the stale-claim refusal assertions: typed unavailable, zero
wrapper, nil lease, zero list calls, and zero perspective checks
(`run_observation_manager_test.go:950-960`). It then supplies the caller's
correct claim and now expects the future successful empty-list result instead
of the stub placeholder (`:961-995`):

- exact `execution_observation` event, current cursor, and fixed timestamp;
- `run_list_state=complete`, `observation_state=no_runs`;
- all six optional counts present and zero;
- read budget limit 8, zero reads, and `Exhausted=false`;
- a nonnil empty `Runs` slice and nonnil lease;
- one perspective verification, one list call, and zero trace reads;
- explicit detach of the empty lease.

The valid list fake returns nonnil empty `Runs` and `UnreadableIDs`, and the
fixture supplies the matching current present-context cursor. Those values
support the expected exact `no_runs` event under revision 6. This is a
future-success test-first assertion against the intentionally unavailable
stub, not a claim that it has passed or that a run occurred.

## Refused-list case and bounded Unit 1 scope

The separate `TestRunObservationManagerRefusedListReturnsUnavailableDTOAndEmptyLease`
and all lifecycle/cleanup fixtures are unchanged from `87be`. The refused-list
case remains aligned with revision 6: unavailable DTO, nonnil empty `Runs`,
all optional counts absent, no candidate trace reads, and a nonnil empty
lease. The private lease declaration still has no `Next` method, so the
fixture does not invent one; `Next() == ErrRunListUnavailable` remains a
future implementation obligation. No manager algorithm, authorization or
guard implementation, production source, public interface, dependency, policy,
or caller wiring changed.

The prior selected RED was captured against test hash `b1b585...` before the
list-refusal and retry-fixture repairs. Its qualified result remains a
historical observation of that prior candidate only; it is not a runtime
result for current test hash `af2df...`. No new RED/GREEN is claimed.

## Verdict and limits

The prior placeholder-behavior defect is fixed. Approve this candidate for
Unit 1 test-first fixture use and the next separately authorized source/runtime
review. This is not approval of the manager implementation, `Next`, rollback,
capacity, lifecycle behavior, runtime GREEN, integration, or T09 settlement.
No compiler, test, scanner, formatter, typecheck, Git, or runtime command was
run.

Graft first-pass retrieval saved approximately 16,515 tokens (~$0.01) this
turn.
