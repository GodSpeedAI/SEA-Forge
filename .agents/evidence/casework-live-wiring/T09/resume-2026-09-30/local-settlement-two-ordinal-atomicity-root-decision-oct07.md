# Root decision: atomic two-event local settlement

The private local settlement operation produces a snapshot and a distinct
settlement_recorded ordinary event. Both require consecutive ordinals from the
same per-case allocator. Revision4 describes sequential numbering; the builder
identified a partial-commit risk when only one ordinal remains. The assignment
requires exhaustion without partial snapshot/side-event/history/frontier change.

Reserve and validate BOTH candidate ordinals, and prepare all fallible snapshot,
history and side-event material, before committing any state. If fewer than two
ordinals remain, reject the entire settlement operation and report one safe
error per affected subscriber; no ordinary event or state/history/frontier
change. Otherwise synchronously commit both consecutive allocations and their
associated state, then enqueue subscriber notifications. User callbacks must
not run inside the commit. A later delivery failure is handled per watcher and
does not roll back a committed settlement.

This clarifies one logical operation's atomicity while retaining distinct
event ordinals and the same ordinary allocator. It changes no public live API,
schema, dependency, security model or kernel boundary. Original revision4 stays
immutable; implementation review must explicitly account for this clarification.

Builder must add NEW narrowly scoped supplemental tests without changing frozen
134ba/373a/edf. After a real public dispatch/start, seed F=9999999998 and prove
atomic rejection with only one slot remaining. Seed F=9999999997 and prove
snapshot9999999998 plus settlement event9999999999, including correct historical
getAt retention. Controlled actual settlement timers required. Write the tests
before implementing this branch. No new baseline RED is claimed, and all new
source/fixtures still require independent critic/root review before runtime.
