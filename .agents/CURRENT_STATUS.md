# Current status

**Status revision:** 121

**Stage:** security-round4-adversarial-hardening

**Summary:** Independent adversarial review verdicts implemented: the supervisor thread is created before any restriction (it inherited the filters and would deadlock on its own brokered connect); there is no continuation of any trapped syscall - the CONTINUE family-flip race (AF_INET rewritten to AF_UNIX between the supervisor's read and the kernel's re-fetch) is eliminated by brokering INET/INET6 with an explicit granted-port check and refusing abstract AF_UNIX outright; sendmmsg(307, +x32) is trapped and refused as a race-free datagram exfil path; getpeername peer names are canonicalized before the grant comparison; connect(fd,NULL,0) dissolves are brokered as written; over-cap iovec arrays refuse. ADR-014 amendment 2 records the supersession of the CONTINUE delegation model.

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
