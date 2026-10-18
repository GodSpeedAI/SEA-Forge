# Stage B: credential lifecycle, JIT elevation, active revocation — 2026-10-17

Captured regression (authority + server, isolation and PostgreSQL required):
689 passed, 0 failed, 9 ignored. Full `just test` on the preceding commit is
in `../continuation-20261017/full-test` (1483 passed, 0 failed, 23 ignored).

| Invariant | Seam | Observation | Status |
| --- | --- | --- | --- |
| Governed issuance, scoped, unexpired | `PostgresBroker::issue` + `database_session` authority | unauthorized scope creates no role; native 42501 outside scope | PASS (supplementary) |
| Renewal re-decides; failed renewal has no fallback | `PostgresBroker::renew` | VALID UNTIL moves on Allow; withdrawn policy → renewal refused, live backend terminated, role dropped; revoked session cannot be renewed | PASS (supplementary) |
| Expiry ends live access | `PostgresBroker::expiry_reaper` (also wired per isolated ACP session) | held `pg_sleep` backend terminated at expiry; post-expiry INSERT refused | PASS (supplementary) |
| JIT elevation: independent, SoD, exact, one use, time-bounded | `PostgresBroker::issue_with_approval` over existing `grant_after_approval` | self-approval, wrong-role approver, other scope, other workload, reused approval all refused with specific reasons; approved INSERT within window succeeds; session ends at approval window; rows `1,2,3,4` only | PASS (supplementary) |
| Active MCP revocation barrier | `McpListenerGuard::revoke` | upstream killed before acknowledgement; no call forwarded after the barrier; `mcp_session_revoked` recorded | PASS (supplementary) |
| Active PostgreSQL revocation | `PostgresBroker::revoke` | terminate → verify none survive → drop; survivors fail closed | PASS (supplementary) |

Mutation controls (manual, restored): reaper without revocation fails the JIT
test (live session survives expiry).

Limits: no injectable clock — expiry tests use real seconds-scale windows,
not T+9/T+10/T+11 exactly. Approver *authentication* is the existing
approval-resolution path (`run_authenticated` + SoD); these tests commit the
approval record directly and exercise the grant and resource sides. JIT
elevation granularity is a PostgreSQL scope role, not a single statement.
Frozen v1 rows SEC-02/07/08 remain NOT-YET-EXECUTABLE (their frozen
interfaces are unchanged).
