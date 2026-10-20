# Current status

**Status revision:** 128

**Stage:** security-round4-install-audit

**Summary:** The supervisor audit still printed zero lines because the gates script's VAR=value function prefix is not exported to children; it is now exported explicitly. The helper logs the applied containment mechanism, listener fd and the /proc/self/status seccomp filter count before and after installation, so the next run splits install failure from inheritance failure for the workload-side non-trapping that the battery evidence indicates (every observed refusal is reproducible by DAC alone; every bypass is an untrapped syscall).

**Verified:**

- just lint clean; sandbox unit tests 11 pass; see revision 113 for the round-4 verification record

**Limits:**

- See revision 113 for the round-4 open limits; this entry records a per-commit increment only.

**Next move:** Kernel-security run for this commit; see revision 113.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
