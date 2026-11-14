# Current status

**Status revision:** 268

**Stage:** PR #71 review-thread fixes on top of the main merge

**Summary:** FDE convergence planning baseline recorded under .agents/reports/fde-convergence-2026-11-12/: deliverables A-H for the FDE convergence program (EGM-reviewed, draft 3) and the SOP v0.4.1 draft, with the S0 baseline evidence. S0: readiness GO 30/30 on 2026-11-13 after the DomainForge 0.21.2 lockstep, and the e2e world loop runs OK. The FDE planning baseline itself changed no product code. S1 not started. Owner decisions OD-1 to OD-6 are open (see H-egm-review-record.md). Security hardening on branch feat/hardening-capability-harvest (PR #71): Final consolidated report published (.agents/plans/2026-10-28-consolidated-final-report.md). Capability harvest: six findings implemented and verified through three independent review passes (final: C0/H0/M2/L4, Mediums fixed); PR #71 checks green at HEAD lineage (required gate, windows, macos, kernel 22.04); one open non-gating lane defect (24.04 workload_boundary fixture ack-EOF) precisely recorded with diagnostics in place. ARM64: artifacts delivered from run 40614104830 (verified hashes, built-from, clean-container installs); release publisher requires arm64 assets; Milestone C pending on-device. Review threads: the server dispatch path now records lifecycle_containment in its command_finished trace payload and execution_result evidence (conformance_case_episode asserts it); write_file removes its staging directory when new_temp fails; egress-control.py closes its log and socket via context managers; the live egress allow-control test prints guest output before its status assertion; OBSERVED_DEBT records the WSL2 post_start_internal_error test failure.

**Verified:**

- scripts/check-agent-context.sh: ok
- 2026-11-13 readiness harness (godspeed-readiness/scripts/readiness.sh, local_operator profile, heads synced to origin main) reported GO 30/30 after the DomainForge 0.21.2 lockstep; evidence in the S0 baseline files under the report directory
- 2026-11-13 scripts/e2e-world-loop.sh against pinned origin heads: final line 'e2e-world-loop: OK' (revision 233)
- gitleaks scan of the report directory: no leaks found
- PR #71 checks: 20+ green incl. gate/test/windows/macos/kernel-22.04; run 40614104830 all four legs success; independent artifact verification
- sea-forge-server and sea-forge-sandbox tests pass on this WSL2 host (workload isolation and cgroup-delegated lanes report NOT_RUN); fmt and clippy -D warnings clean

**Limits:**

- cargo test --workspace and just ci not run (host memory); Cognate/Horizon live suites not re-run
- 0.21.0 prior-identity fixtures kept deliberately: they model the older producer
- docs/how-to/upgrade-domainforge-producer.md still records a 0.21.1 check and its a71ff6 digest as history
- A detached grandchild that survives the process-group kill could append after the operator attests NotDeclared; lease safety assumes the authority call is bounded
- --declared <ref> is not checked against a ledger declaration
- A failure inside declare before the transport spawns is held for an operator, which is conservative
- The CLI builds an Operator from --actor, the same local trust model as approve
- The Horizon policy renderer and the sea-rs fixture are not tied by a test (follow-up)
- The server test covers a nonzero exit only; the timeout variant is covered at the CLI
- The world-load log is INFO level; SFWP has no way to list loaded worlds
- The liveness guard sees the recorded process group only: a process that left it (setsid/setpgid), a resolver in another pid namespace on the same boot, and the instant between fork and the ledger write are not seen
- An Unknown liveness result or a malformed launch record can be overridden only by the operator's audited --attest-command-gone; a group that is positively alive is always refused
- Exactly-once across an operator NotDeclared and a late declaration still relies on the external authority treating authorization_request_sha256 as an idempotency key (contract documented in sea-forge-settlement declaration.rs)
- Server-crash containment comes from the Horizon systemd unit (KillMode=control-group, Horizon#26); it takes effect once the owner reinstalls the unit
- An attempt with no launch record keeps the lease-only rule: NotDeclared is accepted after the lease lapses without a liveness check or --attest-command-gone. This covers claims made before launch recording existed, a transport without launch evidence, and a request with no authorization_ref. A failed launch-record write stops the command, so it does not produce such an attempt.
- Only sea-forge-capability was tested (the sole rusqlite user); full workspace tests, CI and MSRV 1.92 job not run locally; rusqlite 0.41.1 declares rust-version 1.89
- FDE planning baseline is documents only (S1 not started); the hardening work changes product code
- Owner decisions OD-1 to OD-6 are open; see .agents/reports/fde-convergence-2026-11-12/H-egm-review-record.md
- The planning baseline is draft 3 and the SOP is a v0.4.1 draft; neither is approved
- Open: 24.04 fixture defect (non-gating); admission control; F17 allow-control (upstream); Milestone C on-device
- cgroup pidfd sweep cannot reach child-cgroup members (fails closed as survivors); macOS ProcessGroup claim rests on a best-effort census; live egress allow-control policy shape: owner decisions pending on PR #71

**Next:** Owner to answer the open PR #71 review questions; then merge on green. Owner to decide OD-1 to OD-6; then start S1 per F-implementation-plan.md. Review and merge the e2e-world-loop fix (revision 233) if not yet merged. PR #71: address review threads, then merge on green; Session terminal. Follow-ups are recorded in OBSERVED_DEBT and the consolidated report §4.

**Evidence:**

- .agents/reports/fde-convergence-2026-11-12/STATUS.md
- .agents/reports/fde-convergence-2026-11-12/H-egm-review-record.md
- .agents/reports/fde-convergence-2026-11-12/F-implementation-plan.md
- .agents/plans/2026-10-28-consolidated-final-report.md
- .agents/plans/2026-10-27-arm64-jetson-packaging.md
- .agents/plans/2026-10-26-capability-harvest.md
