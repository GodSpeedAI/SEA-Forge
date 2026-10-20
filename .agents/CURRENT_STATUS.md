# Current status

**Status revision:** 122

**Stage:** security-round4-ci-diagnostics

**Summary:** The gates script probed a hardcoded subordinate range belonging to a different account on GitHub runners (the pool reads the real delegation and worked, the probe did not), so legs that could execute the battery recorded NOT-EXECUTABLE; the probe now derives base+1 from the current user's subuid line. The preflight matrix adds runner-owned-binary rows under the real delegation mapping and a full namei traversal record for the helper path, isolating the remaining 24.04 exec denial between mapping content, file ownership, and path traversal.

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
