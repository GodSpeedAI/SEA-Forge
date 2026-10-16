# Observed security v1 baseline

Baseline production commit: `5664912f085f5abbc82895570de8dca19b598a6f`.
Recorded on Linux, 2026-10-16. Production source and dependencies were unchanged.
Pre-existing `godspeedai-stack.code-workspace` edits and `.agents/Explore.md` were
preserved and are excluded from this change.

The canonical acceptance capture is
`.agents/evidence/security-v1/freeze-baseline/results.json`, observer run
`c1ccbc33-6bc8-4c06-aa6d-56f715712145`, recorded at 15:46:01 UTC. Both RED
cases reached actual resource boundaries and their positive controls passed.

| Gate | Case | Observed classification |
|---|---|---|
| A | SEC-01.same_uid_approval | SECURITY-RED |
| A | SEC-01.workload_binding | NOT-YET-EXECUTABLE |
| A | SEC-03.mcp | INFRASTRUCTURE-BLOCKED |
| A | SEC-04.mcp_canonical | INFRASTRUCTURE-BLOCKED |
| A | SEC-05.postgres_native | INFRASTRUCTURE-BLOCKED |
| A | SEC-05.workload_access | NOT-YET-EXECUTABLE |
| A | SEC-12.false_settlement | SECURITY-RED |
| A | SEC-12.end_to_end_attribution | NOT-YET-EXECUTABLE |
| B | SEC-02.credential_lifecycle | NOT-YET-EXECUTABLE |
| B | SEC-06.child_environment | PASS (environment subcase only) |
| B | SEC-06.alternate_paths | NOT-YET-EXECUTABLE |
| B | SEC-07.scoped_elevation | NOT-YET-EXECUTABLE |
| B | SEC-08.active_revocation | NOT-YET-EXECUTABLE |
| C | SEC-09.exposure | NOT-YET-EXECUTABLE |
| C | SEC-10.injection | NOT-YET-EXECUTABLE |
| C | SEC-11.delegation | NOT-YET-EXECUTABLE |

Totals: 2 SECURITY-RED, 1 narrow PASS, 3 INFRASTRUCTURE-BLOCKED, 10
NOT-YET-EXECUTABLE. **No gate passed.** The full SEC-06 property did not pass.

## Observed unmet contracts

SEC-01: the original connection submitted as A, then changed only its claimed
actor to B. The real kernel returned `ok=true`, `status=Approved` and
`resolved_by=operator_b`; the durable journal changed. Native run attribution is
`run_20261016T154557Z_492f8d`, case `case_20261016T154557Z_a9de95`, approval
`apr_0001`, decision `auth_01`. Journal SHA-256 changed from
`ea61d7a92942013936e11210ff176b580978876bfacee837bae486bdbd89a07b` to
`d2face1869e2492954dc386101141be69e8db9c13a07681e263ae3ed827393b5`.
The positive submit and missing/unbound/role/self-approval controls executed.
The configured bindings still lack independent per-workload authentication;
this proves the connection-substitution gap, not a complete identity matrix.

SEC-12: a real governed helper attempted to overwrite a read-only synthetic
file. The OS returned EACCES (13). The before/after SHA-256 remained
`06ebb27e5df9512149363c4be7accb7b424d746db456dbe9987601129712e146`.
The current public operational settlement API nevertheless accepted the
attacker's claimed successful effect. The same pipeline accepted a separately
observed successful write as the positive control. Real authority, committed
decision, admitted invocation, native observation, submitted observation and
returned settlement records were persisted and the ledger hash chain verified.
Observer authentication through every production ingress remains uncovered.

SEC-06: the actual runtime child could not see the uniquely named synthetic
credential present in the parent environment, and the bounded shell produced
`bounded-shell-ok`. This does not prove host-file or network isolation.

## Prerequisites and existing regressions

No PostgreSQL client/server binaries were installed, and no disposable external
URI was supplied. The native probe ran and returned BLOCKED. Its full assertions
exist independently of provider selection; no database result was skipped green.
No product MCP ingress exists/provisioner command was supplied. The full stdio
attacks returned BLOCKED rather than running a fake policy gateway. The harness
self-check ran the real unrestricted upstream and detected both violations as RED.

* `RUSTC_WRAPPER= just crate-test sea-forge-authority`: PASS.
* Existing runtime, sandbox, ledger and settlement crate regressions: PASS,
  captured in `.agents/evidence/security-v1/regression-baseline`.
* The first server baseline was blocked by the existing M16 fixture's missing
  built CLI. `just build` subsequently passed. The resumed original server
  unit/integration tests recorded 483 passed, 0 failed, 2 existing ignored;
  its doctest stage was initially INFRASTRUCTURE-BLOCKED by a missing agent `.rlib` artifact.
  See `.agents/evidence/security-v1/regression-server-after-build`. The ignored
  tests are not new acceptance coverage and are not counted as passes.
* `RUSTC_WRAPPER= just check`: PASS (status, format, clippy, typecheck,
  dependency/advisory checks and repository secret scan). Initial status and
  new-test lint findings were corrected without changing production code or
  weakening assertions.
* `python3 tests/security-v1/selftest.py`: 7 harness checks PASS, including
  missing-prerequisite classification, external PostgreSQL provider selection,
  real insecure-upstream detection and frozen-pin tamper/replacement-ref checks.
  These are harness results, not product security successes.
* Worktree secret scan of `tests/security-v1`: PASS.
* `RUSTC_WRAPPER= just test`: 1,460 passed, 2 failed (exactly the new SEC-01
  and SEC-12 assertions), 23 existing ignored. The canonical full workspace
  run had no other failures, including no remaining doctest artifact failure.
  The ignored tests are 13 real-OPA tests requiring OPA 1.21.1, 6 live OpenShell
  MicroVM tests requiring its live gateway, 2 self-invocation helper tests and
  2 ACP/SWE_SEED release gates requiring configured real hosts. None is counted
  as passing native enforcement evidence.
* `RUSTC_WRAPPER= just proof`: PASS (P1–P4b); captured with workspace tests in
  `.agents/evidence/security-v1/required-gates`.
* `gitleaks protect --staged --no-banner --redact`: PASS. Evidence scanning
  initially reported 62 public SHA-256 `idempotency_key` identifiers as generic
  API keys. `.gitleaks.toml` now recognizes only the exact 18 recorded digests,
  exact seven evidence paths and that single rule, with all conditions required.
  Analysis is retained in `idempotency-scan-analysis.json`; arbitrary new values,
  source paths and credentials retain the existing scan behavior. No assertion,
  authority path, sandbox or conformance gate was weakened.

The restricted execution environment initially blocked sccache and caused
wait-timeout EPERM failures. Subsequent resource/regression runs used normal
local process/socket permissions with the compile cache wrapper disabled;
product sandbox policy and assertions were unchanged. Other operating systems,
MicroVM/KVM isolation, external PostgreSQL and non-stdio MCP transports were
not verified here.

Graft reported approximately 160,846 tokens (~$0.12) saved by the primary
agent's initial three calls. All subsequent reconnaissance was delegated to
Luna as requested; these are retrieval-tool estimates, not measured billing.
