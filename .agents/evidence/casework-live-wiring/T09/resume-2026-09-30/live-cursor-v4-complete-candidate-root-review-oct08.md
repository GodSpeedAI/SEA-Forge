# Root architecture review of the complete V4 candidate

Date: 2026-10-08

Candidateffdfbdb3 is not ready for independent approval or public release.
Preserve it. A fresh builder should produce a bounded revision addressing
these concrete conflicts, then a different critic receives the full original
assignment, root decisions and actual revision.

1. Section 2 proposes uint64 producer ordinals but safe JavaScript integers.
   Specify exact lossless decimal-string wire ordinals backed by u64 in Rust/Go
   and BigInt comparisons in TypeScript, canonical decimal validation and
   explicit overflow errors. Do not silently round or reduce the ledger domain.
   This is an exact proposed public schema amendment requiring approval.
2. Section 5 puts up to 64 replay revisions into a live queue capped at two.
   Return a separately bounded immutable replay snapshot from the same atomic
   readiness/registration lock, followed by the two-frame live queue. Specify
   replay overflow, concurrent invalidation and total retained subscription
   byte accounting; serialized byte limits do not establish heap/RSS limits.
3. Locked subscription admission proves readiness at that instant. It cannot
   prove no invalidation occurs before later headers or writes outside the lock.
   Specify admission/dequeue/write checks, generation invalidation and actual
   cancellation ownership without network I/O under the mutex. State the real
   as-of linearization guarantee and residual in-flight write limit explicitly;
   do not promise instantaneous revocation or atomic kernel Facts.
4. A restarted process may capture new current Facts at the same latest real
   cursor as an earlier process. Cursor/ordinal alone therefore does not uniquely
   identify a capture across restart. Specify detection/reset of incompatible
   client retained captures without fabricating event IDs or historical facts.
   A derived digest of canonical public snapshot bytes is a possible concrete
   proposal; distinguish it from authority cursor identity and enumerate its
   additional public contract changes. Escalate a remaining contradiction.

Derived index and honest as-of inventory choices from root remain candidate
decisions only. Exact caps are proposed availability limits pending approval,
not evidence of measured runtime budgets. All-writer coverage requires direct
verification; unproven writers remain explicit HOLDs. No public implementation
or operator approval is granted by this review.
