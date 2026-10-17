# Security claim / evidence / status ledger — 2026-10-17 (round 2)

Written by the builder. These are not settlement decisions. Each status
below is granted independently; none is inferred from another.

| Status | Meaning here |
| --- | --- |
| IMPLEMENTED | Enforcement code exists on the branch. |
| LOCALLY TESTED | The builder's supplementary tests pass, each with a mutation control that fails when the enforcement is removed. |
| INDEPENDENTLY VERIFIED | A context-isolated reviewer attacked it and it held. Applies only to the commit that reviewer saw. |
| SETTLED | An authority other than the builder (operator or governance) accepted the row. **No row is SETTLED.** |
| BLOCKED | Progress needs a decision or action this builder may not take. |
| NOT-YET-EXECUTABLE | The frozen v1 row cannot run against its fixed interface. |

## Baseline (unchanged)

Frozen v1 is tag `sea-forge-security-v1-2026-10-16` at commit `af41ea2`, manifest
SHA-256 `986454…0a1e66`. The pinned verifier passes. The runner reports 6 PASS
and 10 NOT-YET-EXECUTABLE, exiting 1:

- **PASS:** SEC-01.same_uid_approval, SEC-03.mcp, SEC-04.mcp_canonical,
  SEC-05.postgres_native, SEC-06.child_environment, SEC-12.false_settlement.
- **NOT-YET-EXECUTABLE:** SEC-01.workload_binding, SEC-05.workload_access,
  SEC-12.end_to_end_attribution, SEC-02, SEC-06.alternate_paths, SEC-07,
  SEC-08, SEC-09, SEC-10, SEC-11.

The supplementary results below do not change those rows. Converting a row
needs independent re-qualification against the frozen contract.

## Independent adversarial review (reviewer saw `0b25e08`)

Evidence: `.agents/evidence/security-v2/adversarial-20261017/`. The review
recorded 38 attempts: 20 HELD, 9 PARTIAL, 5 BYPASS and 4 INCONCLUSIVE. Every
attempt is kept, including the unsuccessful ones.

| Finding | Severity | Status after round 2 |
| --- | --- | --- |
| ADV-A19/A20: workload outlives a SIGKILLed launcher; its slot UID is reissued while it runs | high BYPASS | IMPLEMENTED, LOCALLY TESTED (lifeline pipe plus PID-namespace init; slot reaping before reissue). Not re-verified. |
| ADV-B07: PostgreSQL backend outlives `VALID UNTIL` when no reaper runs | high BYPASS | IMPLEMENTED, LOCALLY TESTED (the next issuance reconciles expired roles; idle-session timeout per role). **Residual:** a backend kept busy survives until the next issuance on that cluster. Bounding it with no launcher present needs an out-of-process supervisor. That is a candidate revision. |
| ADV-A06: `/mnt/c` readable | high BYPASS | IMPLEMENTED, LOCALLY TESTED (isolated read allowlist). |
| ADV-A07: control-plane root world-readable | high BYPASS | Landlock layer IMPLEMENTED, LOCALLY TESTED: the root is outside the isolated read set. The DAC layer is unchanged: the tree is still created 0755/0644. Making it 0700 with traverse-only ACLs for workloads affects other principals (gateway socket clients). That is a candidate revision for the operator. |
| ADV-A23: harvest reads before its size check and follows a planted `.agent-harness` symlink | high PARTIAL | IMPLEMENTED: no-follow open, bounded read, symlink refusal. Exercised only by existing harvest tests; no new adversarial test. |
| ADV-B09: classified data in an error payload goes unlabelled | medium PARTIAL | IMPLEMENTED, LOCALLY TESTED (`classified_error_responses_expose_the_context`). |
| ADV-A16/A14: io_uring and nested user namespaces reachable | medium PARTIAL | NOT ADDRESSED. Landlock held. A seccomp filter in the helper needs a new dependency (candidate). |
| ADV-C02: workspace, artifacts and stdout outside exposure | medium PARTIAL | PARTLY ADDRESSED. Issued database credentials are now redacted from the persisted transcript (LOCALLY TESTED). Labelling workspace and artifact outputs and gating their export remains open. |
| ADV-A05/A22/B05/B06/C03 | low/medium | A05 and A22: the read allowlist now blocks host files and symlink read-through outside the granted trees (A22 granted-tree symlinks remain DAC/Landlock-bound). B05 (catalog metadata), B06 (LISTEN/NOTIFY) and C03 (DB labels only at issuance) are open. |

## Coverage matrix (requirement → mechanism → evidence → status)

| SEC | Boundary per mode / transport | Supplementary evidence | Status |
| --- | --- | --- | --- |
| SEC-01 identity | ACP `isolated` and **runtime `execute_command` `isolated`** (new): one subordinate UID per execution. Lifeline-bounded PID namespace. Slots reaped before reissue. Lock directory shared host-wide | `workload_isolation.rs`, `workload_boundary.rs`, `isolated_execute_command.rs`, `t16_4b` | IMPLEMENTED, LOCALLY TESTED. Cross-UID attacks INDEPENDENTLY VERIFIED at `0b25e08` (A01/A02/A04/A12 held) |
| SEC-06 alternate paths | Reads: system trees plus declared runtime paths only. Abstract sockets and signals scoped. TCP: netns, plus Landlock layer tested alone. Pathname UNIX sockets: DAC only (Landlock ABI 7 has no socket-connect right) | `workload_boundary.rs` (read posture with widened-scope mutation; network layers), adversarial A10/A11 | IMPLEMENTED, LOCALLY TESTED. Pathname sockets: residual |
| SEC-03/04 MCP | stdio: per-workload broker (`SO_PEERCRED`). **HTTP/SSE/streamable HTTP: not representable in config** (unit test). Isolated agents have no network to reach one directly | `t16_4c`, `mcp_workload_broker.rs`, `network_mcp_transports_are_not_configurable` | IMPLEMENTED, LOCALLY TESTED. B08 frame tricks INCONCLUSIVE |
| SEC-05 database | Native role per session. Peer-only admin | `postgres_broker.rs`, `t16_4d` | IMPLEMENTED, LOCALLY TESTED. B01–B04 INDEPENDENTLY VERIFIED |
| SEC-02/07/08 lifecycle | Renewal re-decides. Reaper ends sessions. Expired roles reconciled at the next issuance. JIT deadline is a pure function (injected-clock boundary tests T+9/T+10/T+11) | `postgres_broker.rs`, `postgres::tests` | IMPLEMENTED, LOCALLY TESTED. C01 INCONCLUSIVE in review |
| SEC-09/10/11 exposure | MCP-forwarded classified responses, including errors. DB scopes at issuance. Lineage | `exposure_flow.rs` | IMPLEMENTED, LOCALLY TESTED for MCP and DB routes. Workspace, artifacts, stdout and pathname-socket routes are not DLP-covered |
| SEC-12 RealityTrace | SEC-12 slice. **Phase 3: every brokered attempt (success and failure) projected with categories and native-observation digests**, reconstructed by an independent auditor (tamper, drop and forge mutations) | `sec12_trace.rs`, `broker_attempt_trace.rs` | IMPLEMENTED, LOCALLY TESTED. RealityTrace-side additions BLOCKED (below) |

## Uncovered paths (builder analysis, not yet attacked)

These are the practical routes that remain around the enforcement architecture:

1. **Database writes after exposure.** Write-capable database sessions are
   issued at episode start. A context exposed later keeps them, so it can
   copy classified data into tables other principals read. Exposure gates
   MCP destinations only, not live database sessions. Fix candidate: revoke
   or refuse write scopes when the context's labels exceed the scope's
   `permitted_exposure`.
2. **Workspace and artifact export** (ADV-C02). Agent-written files leave
   through the SWE_SEED harvest, artifact bundles (`sea-forge-cell` bundle)
   and HTTP artifact fetch, all without exposure labels.
3. **Transcript and stdout.** Issued database credentials are redacted;
   classified plaintext an agent prints is not. The full-mode transcript is
   stored in plaintext under `runs/`.
4. **Pathname UNIX sockets.** DAC only. Every socket a subordinate UID can
   reach (for example a world-writable socket in a traversable directory)
   is a channel Landlock ABI 7 cannot close.
5. **Non-isolated classes.** `local` and `jail` keep the shared launcher
   UID by design. They are not offered MCP or database resources (fail
   closed), but any untrusted code run under them has the launcher's reach.
6. **Kernel attack surface.** io_uring, user namespaces nested inside the
   workload, and userfaultfd remain reachable (no seccomp).

The network itself is not an exfiltration path for isolated workloads: they
get a fresh network namespace (loopback only), and Landlock denies TCP as a
second layer, which is tested on its own. That holds for the `isolated`
class only; MCP-mediated exposure is not network-wide DLP for other classes.

## Blocked: RealityTrace contract additions

Commit `8c95489` is on the local `~/projects/sxr` branch
`security/acceptance-evidence-20261017`. It is not pushed, and SEA-Forge is
still pinned to `0e2a97b`.

| Addition | State |
| --- | --- |
| A distinct `security_acceptance` proof producer, bound per cycle (`ingest_from`), exclusive (refuses `swe_seed` and the actor `sea_forge`). The producer is named in the comparison and in the E8 provenance | IMPLEMENTED, LOCALLY TESTED in sxr. T07 16/16, sxr-core suites green, including the frozen T08 golden |
| Field-level `difference.fields`: JSON-Pointer entries, `changed`/`missing`/`unexpected`, per-side digests, bounded at 256 with `fields_truncated` | IMPLEMENTED, LOCALLY TESTED in sxr |
| `evidence.recorded` record kind | NOT IMPLEMENTED. Proposal at `.agents/plans/evidence-recorded-registration-proposal-20261017.md` (in sxr). The registry gate requires CEP Evidence semantics (`question_ref`, `direction`, `reliability`), and the CEP owner decides those |

**Blocker:** adopting any of this changes RealityTrace's public contract
(exclusive producer authority, E8 shape). It needs operator or spec-owner
approval and a push of the sxr branch, so that SEA-Forge can re-pin
`sxr-core` and drop the `swe-seed` label workaround.

## Gitleaks

Control results: `.agents/evidence/security-v2/gitleaks-control-20261017/`.
The exception was narrowed to exact recomputed values, and all 13 canaries
behave as specified. `gitleaks detect` misses staged-but-uncommitted content
(candidate gate change).

## Required for settlement

1. Independent re-verification of round 2 at its commit, aimed at the new lifeline, reconciliation and read posture.
2. Operator decisions on the candidate revisions:
   - DAC hardening of the state tree;
   - an out-of-process database supervisor;
   - seccomp;
   - DLP for workspace and artifact export;
   - RealityTrace adoption.
3. Promotion review of the supplementary candidates through governance (mapping in `tests/security-v2/candidate-manifest.json`).
4. Settlement authority: the operator, not this builder.
