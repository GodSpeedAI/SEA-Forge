# Current status

**Status revision:** 125

**Stage:** security-round4-deny-versus-kill

**Summary:** End-to-end execution of the charter battery on the 24.04 runner isolated the last defects, all fixed: policy denials (outside grants, abstract sockets, ungranted ports) now reply EACCES to the frozen workload instead of killing it, so refusals are observable and the workload continues preventively; kill-before-reply is reserved for race-class and uninterpretable cases (getpeername mismatch after the pin, unreadable or oversize payloads), threaded with the workload pid. The inherited-descriptor test deadlocked on its own listener thread reading a stream the test held (dropped before the join); the availability guard exited the whole test binary silently on an unavailable host and now returns per test. Abstract refusal expectation follows the EACCES spelling. Gates gained 20-minute suite and 10-minute mutation watchdogs. First live results: rename-swap race, datagram sendto and own-tree IPC PASSED on a real kernel.

**Verified:**

- just lint clean; sandbox unit tests 11 pass; see revision 113 for the round-4 verification record

**Limits:**

- See revision 113 for the round-4 open limits; this entry records a per-commit increment only.

**Next move:** Kernel-security run for this commit; see revision 113.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
