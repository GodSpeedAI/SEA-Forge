# Current status

**Status revision:** 124

**Stage:** security-round4-pg-provisioning

**Summary:** The gates script now provisions the lifecycle battery from the distro PostgreSQL layout (/usr/lib/postgresql/<v>) when SEA_FORGE_SECURITY_PG_INSTALL is unset, so CI runs the full qualified battery instead of failing on the test's own honest gate; it skips with a recorded reason only when no install root exists. Also recorded: 22.04's util-linux 2.37 predates unshare --map-users, so its identity pool honestly reports unavailable there (portability follow-up: a newuidmap-direct fallback in the pool).

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
