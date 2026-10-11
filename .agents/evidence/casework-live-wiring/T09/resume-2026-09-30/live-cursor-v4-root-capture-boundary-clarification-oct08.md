# C2 root clarification: coherent capture across existing read calls

Date: 2026-10-08. DOCONLY proposal, no implementation release.

The root decisions' shared writer lock cannot by itself make Go LiveSource.Facts
atomic: it makes several existing kernel RPCs. Do not invent a client-owned
kernel lock, hold a file lock across network I/O, or imply that individually
locked view reads form a transaction.

Concretize the proposed journal with a durable checked-u64 cell state epoch.
Under the common writer lock, publish the pending marker and advance its epoch
durably BEFORE case-visible writes. Complete/recover operations under that lock,
durably advance the epoch on the finished transition, then expose clean state.
Overflow is unavailable, never wrap or reset. Pending/complete crash ordering
must make uncertain state dirty, never accidentally clean. This epoch is a
derived mutation detector, not an event ID, revision, authority or source-loss
count. Its schema and wire additions require exact approval.

Extend existing version2 range/status responses with a bounded current
case_state_stamp containing canonical decimal-string state_epoch and the
requested case's dirty boolean. A stamp read acquires the same writer lock,
reads actual durable epoch/pending state and real case event head, then unlocks
before returning. No new kernel verb or credential. Stamps do not reconstruct
history or increment acknowledgement by themselves.

Go captures a clean stamp before its existing authorized Facts reads, then a
clean stamp after them. Accept only equal epochs and exact same real case
head/cursor/ordinal; dirty/change/unknown/cap failure is unavailable with no
partial publication. Retry at most3 bounded attempts, then typed unavailable.
An in-tree mutation during the interval necessarily changes epoch or leaves
dirty state. Reads therefore capture a coherent supported-writer interval;
they are not atomic with later disclosure/mutation. A mutation after the final
stamp may make the capture stale; existing event delivery and intent/domain
checks still apply. Unsupported direct filesystem writers remain outside that
guarantee, explicitly stated in the exact approval request.

The candidate must include stamped-capture tests: mutation between any two
view reads; crash before/after pending durability; dirty marker recovery;
epoch overflow; clean same-epoch success; post-final-stamp residual; no new
kernel/client lock or network I/O while a filesystem/Store mutex is held.

This additive clarification supersedes only the ambiguous capture-lock phrase
in live-cursor-v4-root-decisions-oct08.md. All other choices and C2 HOLD remain.
