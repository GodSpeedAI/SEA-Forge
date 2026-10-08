# Independent review: initial poller failure clarification

Date: 2026-10-07  
Verdict: **AGREE, with the branch distinction below carried into the supplement.**  
Scope: review of the root clarification against revision 6 and the lifecycle preregistration. Source-only; no code, fixture, compiler, test, build, scanner, or Git work.

## Finding

The clarification correctly distinguishes an unavailable selected run after a successfully decoded list from a failed `Prepare`.

Revision 6 defines `A` for selected rows that are unavailable, malformed, denied, mismatched, orphaned, or unretainable; only validated `V` rows appear in `Runs`. It requires all optional counts on a successfully decoded list and gives `unavailable` precedence when `U>0 || A>0` (revision 6 §§“Cohort selection, counts, and exact DTO” and “Outcome table”). Consequently an initial selected trace read that returns unavailable cannot become a whole-Prepare error or erase successful-list counts. The root clarification’s `A`, no-Runs-row, nonnil cohort lease, and normal unavailable DTO are consistent with that contract.

The distinction from failed-Prepare rollback also matches the original assignment and preregistration: authorization, guard, cancellation, stop, or irreducible assembly failures return an error/zero wrapper/nil lease and require internal rollback. Revision 6 separately identifies irreducible initial helper metadata failure as `ErrInitialAssemblyUnavailable` with nil lease and rollback (§“Outcome table”); do not accidentally classify that branch as successful `A` merely because it occurred during initialization. A decoded-list run that is unretainable because its candidate cannot fit the approved combined image remains `A` as root decided, while irreducible DTO/helper assembly is still the typed failed-Prepare branch.

The per-Prepare read count is also consistent with revision 6: `ReadsAttempted` measures actual logical starts, while cache/shared-initializer reuse is zero (§“Cohort selection, counts, and exact DTO”). Thus an initializer owner can report one start and a waiter that shares it can report zero. Both receive the same fixed poller initialization outcome, but their entire DTOs need not be byte-identical; capture time and caller-owned read accounting are not properties of the shared poller result. The clarification’s concrete one-row example correctly has the same `R=1,V=0,A=1,C=0,O=0,U=0` and unavailable observation state for both callers.

## Wording to preserve in the fresh supplement

Use “same fixed poller initialization outcome,” not “same DTO.” Each Prepare independently reports its actual `ReadsAttempted`; its derived `Exhausted` value follows the approved per-Prepare formula `ReadsAttempted==8 && R>8`. The one-row example has `Exhausted=false` for both, but a multi-row, eight-start owner and a shared waiter may differ on that derived field as well as `ReadsAttempted`. The clarification’s phrase “only the caller-owned start count may differ” should be understood as the per-Prepare accounting difference for the stated one-row outcome, not a blanket byte-for-byte DTO guarantee.

For a selected initial read failure, the returned cohort lease may have no poller attachments after the failed attachment is released internally; it remains a valid lease until ordinary detach. This is distinct from the nil lease returned for failed Prepare. Shared waiters must each receive the one poller result, release only their own failed reference, and not cause an additional read or retry. Worker return/JOIN and capacity retention remain independently tested as required by the prior review.

This review accepts only the clarification’s semantics. It does not approve the pending combined-image supplement, fixture edits, lifecycle implementation, or compiler work. Those remain subject to the fresh supplement and separate independent review/root release.

## Source anchors

- `run-observation-manager-concrete-proposal-revision6-oct06.md`: “Cohort selection, counts, and exact DTO”; outcome rows for selected invalid/denied/mismatch/orphan, initial irreducible metadata error, and failed later trace read.
- `run-observation-manager-unit1-original-assignment-oct06.md`: original bounded Prepare/rollback ownership and successful-watcher behavior.
- `run_observation_manager_unit1_lifecycle_implementation_preregistration_oct07.md`: §2 Prepare outcomes/counts; §3 shared initializer ownership and cleanup; required invariant that failed Prepare has internal rollback ownership.
- `manager-combined-image-root-decisions-oct07.md`: decision 2 initial failed/unretainable selected read outcome and decision 7 fixture requirements.
- `manager-initial-failure-root-clarification-oct07.md`: clarification reviewed here.
