# Current status

**Status revision:** 129

**Stage:** security-round4-audit-surfacing

**Summary:** The launch audit lines were being swallowed: the tests capture the whole launch chain's stderr with .output() and printed only the workload stdout. The socket tests now surface any supervisor or helper audit lines into the suite log, so the next run shows the applied mechanism, the seccomp filter count across installation, and every notification decision for the failing pathname and symlink connects.

**Verified:**

- just lint clean; sandbox unit tests 11 pass; see revision 113 for the round-4 verification record

**Limits:**

- See revision 113 for the round-4 open limits; this entry records a per-commit increment only.

**Next move:** Kernel-security run for this commit; see revision 113.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
