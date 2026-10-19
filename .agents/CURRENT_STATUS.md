# Current status

**Status revision:** 117

**Stage:** security-round4-unotify-brokered-authority

**Summary:** ADR-014 amendment: the seccomp supervisor no longer continues workload syscalls under a userspace verdict. Kernel-authoritative targets (non-UNIX families under Landlock net rights, abstract AF_UNIX under the scope) use CONTINUE; pathname targets and address-less sends are brokered - pidfd_getfd duplicate of the workload descriptor, single-fetch process_vm_readv payload, connect-pin with getpeername kernel-recorded peer verification under the granted trees, and kill-before-reply on any mismatch, so a rename raced into the check-to-connect window costs the workload its life and the outside peer an empty connection, never data. Fail closed: 1 MiB payload cap, 5s connect poll, launch refusal when no listener is installable. The draft's fake-success allow reply and planned check-then-continue are removed.

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
