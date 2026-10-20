# Current status

**Status revision:** 131

**Stage:** security-round4-workload-self-inspection

**Summary:** The connect workload now reports its own seccomp mode, filter count and a NEW_LISTENER self-probe (EBUSY proves a listener is in its own chain) alongside the connect attempt, making the next run self-describing about whether the user-notify filter is present and why its notification is not delivered.

**Verified:**

- just lint clean; sandbox unit tests 11 pass; see revision 113 for the round-4 verification record

**Limits:**

- See revision 113 for the round-4 open limits; this entry records a per-commit increment only.

**Next move:** Kernel-security run for this commit; see revision 113.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
