# Root decisions for narrow protocol repair

These resolve the two findings in the fresh supplement's independent rejection.
They release no source or fixture work. Existing controls, bounds, scopes and
frozen fixture identities remain unchanged.

## One ref-release owner

The first lease detach/drain transition under the actual manager mutex owns
removing that lease's remaining exact refs. It clears the lease's ref membership
atomically and records any watcherless workers to cancel after unlocking.
Awakened Prepare waiters do not remove those refs again: they recheck draining
state and use/join that same lease drain owner. No independent decrement is
allowed for already-absent membership.

A selected A-result cleanup while the lease is still preparing is separate:
that Prepare removes only its own failed selected ref under the same mutex if
membership still exists. A concurrent detach either runs first and owns that
release, or runs later and sees it absent. Exactly one transition wins removal;
no path decrements twice or cancels a worker with another eligible lease ref.
Cancellation, closes, callbacks and JOIN remain outside the mutex. Add explicit
race coverage for detach versus selected-ref cleanup and for awakened waiters.
Failed-Prepare rollback must not wait on its own unfinished Prepare operation;
operation ownership must finish before waiting on its drain completion.

## Stopped before first read

A worker whose reserved entry is stopped before its first port call uses the
existing generic invalid/unavailable initializer code4 with phase stopping2
(or draining3 if teardown already owns the entry). It retains no current value,
starts no trace read, and claims/resolves ready exactly once outside the mutex.
No seventh initializer code or candidate metadata is introduced.

Prepare always rechecks manager/lease stop state after a wake; this path is a
failed Prepare with typed stop/cancellation error, zero wrapper, nil lease and
owned rollback. Code4 does not turn a stopped Prepare into a successful A-result.
WorkerDone closes only when that real worker returns; reservation and capacity
remain counted until the required actual JOIN/drain. Test both allowed phases,
existing code range, no port start, ready resolution and stop-winning recheck.
