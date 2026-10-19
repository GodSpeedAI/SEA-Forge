# Current status

**Status revision:** 118

**Stage:** security-round4-kernel-ci

**Summary:** Dedicated GitHub Actions kernel-security job (ubuntu-24.04 VM, no container) records a capability preflight - kernel, inherited seccomp mode, an actual disposable-child user-notify install probe, Landlock ABI, user-namespace availability (relaxing the Ubuntu 24.04 AppArmor restriction via sysctl when refused, recorded before/after), subuid delegation and uidmap prerequisites - then runs scripts/security/round4-socket-gates.sh against the exact tested commit including the M-S2/M-S4 mutation controls. The gates script now distinguishes PASS/FAIL from NOT-YET-EXECUTABLE: without mapped user namespaces the suites self-report unavailable and mutations are recorded NOT-EXECUTABLE rather than false-KILLED. The job is intentionally not a required gate and writes a mechanism-status summary separating native Landlock (NOT-YET-EXECUTABLE on runner ABI < 9) from the secured fallback.

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
