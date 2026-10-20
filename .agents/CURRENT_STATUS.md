# Current status

**Status revision:** 123

**Stage:** security-round4-helper-traversal-repair

**Summary:** ensure_helper_reachable now walks the entire helper path: inside the mapped namespace every host identity is unmapped, so directory components lacking other-traverse block execve before the file is reached (the working unfiltered probe execs a system path, which is why only the isolated launch failed). The launcher repairs launcher-owned directories along the path by granting other-read/execute - never write bits - and fails closed naming the first component it cannot repair; the regression test covers the 0750-parent shape, and the CI preflight records and repairs a non-traversable checkout path on the disposable VM.

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
