# Round 2 — enforcement gaps closed after the adversarial review

Builder evidence. Not independent verification. The reviewer saw `0b25e08`,
and none of this round has been re-attacked.

Environment: WSL2 kernel 6.18.54 (Landlock ABI 7), uidmap 4.17.4,
delegation `sprime01:100000:65536`, PostgreSQL 18.1 (pg0). Runs used
`SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION=1` and `SEA_FORGE_REQUIRE_POSTGRES=1`,
so missing prerequisites fail instead of skipping.

## Suites (all exit 0 on the restored sources)

| Log | Contents |
| --- | --- |
| `sandbox-boundary.log` | `workload_boundary` 4/4, `workload_isolation` 3/3 |
| `server-suites.log` | server lib 137, `conformance_m16` 19 (+2 pre-existing ignored), `exposure_flow` 2, `mcp_workload_broker` 3, `postgres_broker` 4 |
| `postgres-broker.log` | `postgres_broker` 5/5, including the ADV-B07 reconciliation test |
| `realitytrace.log` | `broker_attempt_trace` 1, `sec12_trace` 5 |

Also run, but the log was not kept: the runtime `isolated_execute_command`
test passed 2/2, and `postgres::tests` and `mcp::tests` passed 2/2.

## Code-level mutation controls (`mutations.log`)

All six mutations were applied together and each targeted test failed with
its intended assertion. The sources were then restored, verified by SHA-256
against pre-mutation copies.

| Mutation | Detecting test | Failure |
| --- | --- | --- |
| M1: no reaping before slot reissue | `workload_dies_with_launcher_and_orphans_are_reaped_before_reissue` | "slot reissued while its previous workload still runs"; the orphan was left alive and then killed by hand |
| M2: isolated read scope widened to `/` | `isolated_reads_are_limited_to_system_and_declared_trees`, `isolated_execute_command…` | host file, home, `/tmp`, `/run`, `/var` and `/sys` readable; isolated command read the host file (exit 0) |
| M3: exposure recorded only on accepted responses | `classified_error_responses_expose_the_context` | no label after a classified `isError` payload |
| M4: issued credentials not redacted | `t16_4d…` | "credential persisted in …/ledgers/case-…/entries.jsonl". This confirms the leak existed before the fix |
| M5: no expired-session reconciliation | `expired_sessions_without_a_reaper_end_at_the_next_issuance` | expired backend still active after the next issuance |
| M6: closed approval window accepted | `postgres::tests::approval_window…` | a session was minted at T+10 for a window ending at T+10 |

The in-test mutation controls (widened read path; launch without a
lifeline; network posture with a TCP grant) run on every test run.

## Diagnosed during the round

- **pdeathsig does not work here.** A `setpriv --pdeathsig` wrapper did not
  kill the workload. The parent-death signal is cleared by the credential
  change in `unshare(CLONE_NEWUSER)`. It was replaced by the lifeline pipe.
- **Read-only reopen.** The helper could not reopen a 0600 launcher-owned
  pipe by path. Mode 0444 permits reopening for read only; reopening for
  write was verified denied.
- **Probe flaw fixed.** `ls` on a file only stats it, and Landlock does not
  govern stat, so files are now probed by reading their content.
- **Build deadlock.** The sccache daemon inherited the cargo serialization
  lock fd; runs now use `flock -o`.

## Gates (`gates/`, final tree before commit)

| Gate | Result |
| --- | --- |
| `just check` | exit 0, "all gates green" (fmt, context, clippy `-D warnings`, typecheck, deny, gitleaks) |
| `just test` (isolation and PostgreSQL required) | 1498 passed, 0 failed, 23 ignored (the same pre-existing 23) |
| `just proof` | exit 0 |
| `just no-async-kernel` | exit 0 |
| `just security` | exit 0, no leaks |

Changes made after the full test run: a test-only `wait()` added after the
fixture's sleep, a clippy `allow` on the helper's init-reaped spawn, and an
unused import removed. The sandbox suites were re-run afterwards (4/4, 3/3).
The first full run failed one round-1 test, `slots_are_exclusive_while_held`,
which assumed it held the slots exclusively. Host-wide slot sharing broke
that assumption; the test now waits for the released slot.
