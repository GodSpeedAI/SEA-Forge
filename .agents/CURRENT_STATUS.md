# Current status

**Status revision:** 138

**Stage:** security-round4-broker-refusal-audit

**Summary:** With the arch constant fixed the 24.04 supervisor now traps: outside and declared sockets are refused and only the granted one connects (helper self-probe errno 13). Two socket tests still failed only because the diagnostic prelude in the connect workload broke their exact-match assertions, so the original connect script is restored. postgres-lifecycle now fails, since its workloads' connects to the granted Postgres socket are refused (EACCES) where the no-op filter used to let them through; the brokered path now logs which check refuses.

**Verified:**

- run 39482520444 on 24.04 (kernel 7.0, Landlock ABI 8): outside and declared refused, granted connected, self-probe 13
- same run: postgres-lifecycle fails with 'timed out waiting for three backends' while the supervisor logs Err(13) for the granted /tmp/.tmp*/sock/.s.PGSQL.* path

**Limits:**

- The reason the granted Postgres socket is refused is not yet established; the 24.04 gate stays red until it is. See revision 113 for the other round-4 open limits.

**Next move:** Read the pathname/dup/pin_and_verify debug lines from the next 24.04 postgres-lifecycle log.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
