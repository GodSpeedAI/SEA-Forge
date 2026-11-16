# Current status

**Status revision:** 280

**Stage:** WP-58 SEA-Forge consolidation: CI optimization, server shard builds the CLI

**Summary:** CI is restructured around the measured critical path. Baseline on main (run 43656019461): verify (macos) 967 s was the slowest required job, test (ubuntu) 614 s, lint 217 s, MSRV 114 s. Changes: Swatinem/rust-cache (pinned by SHA) in every cargo job, saved from main only; the Linux and macOS test sweeps split into two parallel shards (just test-shard server|rest, the union equals just test; the server shard first builds the CLI binary that its tests spawn); the macOS legs run only the platform-dependent gates (just ci-platform) because fmt, status, deny and gitleaks already run on Linux; a scope job classifies docs-only pull requests so the heavy jobs skip while gate still runs and only accepts a skipped job for a docs-only change; setup-bun pinned by SHA; Portability runs on pull requests and main pushes only and skips docs-only changes; the dead ci/openshell-live push trigger is removed. Required checks stay gate and conventional-commit.

**Verified:**

- actionlint on all workflows: only pre-existing shellcheck info notes
- just --dry-run test-shard server and rest expand as intended
- before/after wall times are recorded in the PR and the ledger

**Limits:**

- cargo test --workspace and just ci not run (host memory)
- Cognate/Horizon live suites not re-run
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
- The FDE planning baseline on main is documents only; S1 implementation is on the fde/integration branches, not on main
- Owner decisions OD-1 to OD-6 are open; see .agents/reports/fde-convergence-2026-11-12/H-egm-review-record.md
- The planning baseline is draft 3 and the SOP is a v0.4.1 draft; neither is approved
- cargo test --workspace and just ci not run (host memory); Cognate/Horizon live suites not re-run
- FDE planning baseline is documents only (S1 not started); the hardening work changes product code
- Open: 24.04 fixture defect (non-gating); admission control; F17 allow-control (upstream); Milestone C on-device
- cgroup pidfd sweep cannot reach child-cgroup members (fails closed as survivors); macOS ProcessGroup claim rests on a best-effort census; live egress allow-control policy shape: owner decisions pending on PR #71

**Next:** Read the cold and warm run times from the PR and the first main run.

**Evidence:**

- .agents/reports/fde-convergence-2026-11-12/STATUS.md
- .agents/reports/fde-convergence-2026-11-12/H-egm-review-record.md
- .agents/reports/fde-convergence-2026-11-12/F-implementation-plan.md
- .agents/plans/2026-10-28-consolidated-final-report.md
- .agents/plans/2026-10-27-arm64-jetson-packaging.md
- .agents/plans/2026-10-26-capability-harvest.md
