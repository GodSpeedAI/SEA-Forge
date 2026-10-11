# Initial failure clarification for independent design repair

Root read the independent rejection and accepts the missing shared-initial
failure and stop-versus-ready fixture/protocol findings. This clarifies two
phrases in that review before a fresh builder prepares its supplement.

A decoded-list selected run's first unavailable/invalid/unretainable read is
A, not a failed Prepare. Each otherwise successful Prepare returns nil error,
the normal unavailable initial DTO with successful-list counts, and a nonnil
cohort lease. The failed poller attachment is released internally; no caller
cleanup responsibility for that attachment is invented. If no observed rows
remain, the valid returned cohort lease has no poller attachments and continues
to count until normal detach. This follows the reviewed R/V/A/C/O rules and
is separate from auth/guard/cancellation/stop/irreducible assembly failure,
which returns zero wrapper and nil lease plus internal owned rollback.

Shared waiters receive the same fixed poller initialization outcome, not
necessarily byte-identical whole DTOs. ReadsAttempted counts actual logical
starts owned by each Prepare: the new initializer owner counts one, a shared
initializer waiter counts zero. Each DTO may also have its own capture time.
For a single failed selected readable row, both have R=1,V=0,A=1,C=0,O=0,U=0,
no Runs row, and unavailable observation state; only the caller-owned start
count may differ. No second logical/physical initialization or retry is allowed.

The review's phrase 'cleanup has an owner even though no lease is returned'
applies to failed Prepare paths, not this successful A-result path. The fresh
fixture must distinguish both. Its 'same unavailable DTO/result' phrase must
mean the shared poller result and matching count semantics, without weakening
the independently approved per-Prepare actual-start accounting.

This clarification releases no source work. Full combined-image supplement,
exact JSON branches, waiter signaling ownership and test-first coverage still
require independent evidence-based approval. Existing frozen fixtures stay exact.
