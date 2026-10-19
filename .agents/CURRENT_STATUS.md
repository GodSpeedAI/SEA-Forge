# Current status

**Status revision:** 120

**Stage:** security-round4-helper-reachability

**Summary:** Kernel-security run 1 diagnosed: the preflight proved the user-notify mechanism installable on GitHub runners (fd 3 in a disposable child) and the sysctl userns relaxation effective, but the helper binary failed execve inside the mapped namespace because a restrictive-umask build leaves only owner bits and the namespace maps no owner for the workload process. New ensure_helper_reachable: verify other-rx before launch, repair when launcher-owned, fail closed otherwise; regression test included (11 sandbox unit tests pass).

**Verified:**

- just check all gates green on this tree; sandbox unit tests 10 pass; see revision 113 for the round-4 verification record

**Limits:**

- See revision 113 for the round-4 open limits; this entry records a per-commit increment only.

**Next move:** Kernel-security CI job, independent adversarial review, then operator-directed push; see revision 113.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md
- docs/decisions/ADR-014-unix-socket-containment.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
