# Creator/drain wording erratum — 2026-10-07

Applies to `run-observation-creator-drain-final-correction-supplement-oct07.md`
(SHA-256 `6526c249f800f23b966063b69ef97f00fa3531b69c0057ccedd52241b2160758`);
all prior records remain unchanged.

1. “Marks its now-ineligible batch for stop” applies only to entries made
   watcherless by removing this lease's exact eligible membership:
   `!lease.draining && lease.pollers[key] == entry`. Another eligible lease
   keeps a shared entry running; creator cancellation alone never forces phase
   2/3, code 4, or no-read. Launch every immutable reserved entry exactly once.
   A failed creator returns zero DTO/nil lease; a shared worker may still make
   its actual initial read, while a surviving Prepare's ownerReads is zero.
2. Sixteen bounds logical waiting lease-drain owners for counted cohorts, plus
   one logical global Stop owner; it is not an instantaneous goroutine, heap,
   RSS, or process-memory bound. After required list/bridge/creator/worker JOINs
   and registry removal, the owner closes stable completion outside the mutex
   and returns; this tail may overlap a newly admitted cohort. Capacity release
   and completion closure still follow required actual JOIN. Existing
   draining/stopping ownership is unchanged; no additional state is introduced.

Wording only: no source, fixture, test, compiler, formatter, scanner, Graft,
Git, or runtime work is authorized. Independent review remains required before
TDD source release; no implementation or runtime claim is made.
