# Watcher membership during external validation

Date: 2026-10-08
Architecture note for the later manager/worker repair, not a source grant.

The pre-read survivor regression attaches a new valid lease while the
worker's initial watcher validation is held in an external dependency. The
initial local watcher snapshot may contain only the initiating lease.
Rejecting that initiator must not stop a worker whose exact current forward
membership now includes an independently valid survivor.

After external checks, recheck exact entry/prior pointer/phase/membership under
the existing mutex. If a currently eligible watcher was admitted after the
snapshot, include and independently validate it before concluding no watcher
can authorize the read. Never treat an unvalidated new member as authority.
Use local snapshots and existing state; no persisted generation, counter,
optional policy, or extra field is required. All Current/perspective/guard
callbacks remain outside the mutex. Invalid watchers drain independently.

The new test must genuinely attach that survivor before releasing the held
authorization dependency. Later semantic review and runtime proof must cover
this membership race as well as post-read publication and final handoff.
