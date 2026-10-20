# Current status

**Status revision:** 135

**Stage:** security-round4-trap-probe-preflight

**Summary:** The 24.04 run now exposes the real failure: connect(2) to an outside socket succeeds and the supervisor logs no notification (probe errno 2 = untrapped), while 22.04 passes with identical code. The preflight now runs a standalone USER_NOTIF trap probe, bare and inside the workload namespaces, to say whether kernel 6.17 traps connect at all before any SEA-Forge code is touched.

**Verified:**

- 24.04 sandbox-suite log: 2 of 7 unix_socket_reach tests fail on real connects; datagram test passes only because file permissions refuse it (not evidence the supervisor traps)

**Limits:**

- Root cause on kernel 6.17 is not yet established; the containment claim does not hold on 24.04 until it is. See revision 113 for the round-4 open limits.

**Next move:** Read the preflight trap-probe verdict from the next kernel-security run and act on it.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
