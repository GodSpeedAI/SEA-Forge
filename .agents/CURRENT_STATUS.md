# Current status

**Status revision:** 132

**Stage:** security-round4-trap-probe

**Summary:** The failing connect workload now attempts a nonexistent-path connect first and prints its errno: ENOENT means the user-notify filter never traps the syscall (untrapped, kernel ENOENT); EACCES means the filter traps and the broker refuses - splitting filter-matching from reply delivery for the bypass.

**Verified:**

- just lint clean; sandbox unit tests 11 pass; see revision 113 for the round-4 verification record

**Limits:**

- See revision 113 for the round-4 open limits; this entry records a per-commit increment only.

**Next move:** Kernel-security run for this commit; see revision 113.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
