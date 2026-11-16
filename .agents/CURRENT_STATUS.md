# Current status

**Status revision:** 281

**Stage:** WP-58 SEA-Forge consolidation: compatible-minor group plus jsonschema 0.60

**Summary:** The compatible-minor Dependabot group lands with tokio held at 1.53.3, and the jsonschema bump from Dependabot #70 (0.58.6 to 0.60.0, resolved 0.60.2, used only by sea-forge-realitytrace) joins it. With tokio 1.54.1 the HTTP stub tests in sea-forge-server (conformance_m12 oversize, conformance_m13 endpoint error) failed on hosted runners because the stub closes with the request unread and the client sees a connection reset instead of the HTTP status; the stubs need to drain the request head before tokio 1.54 can land (conformance_m12's oversize stub already does). serde-saphyr 2.1 now rejects tRue and fAlse under flatten and tagged enums, so the case-variant pin table keeps only the four spellings it still accepts.

**Verified:**

- cargo test -p sea-forge-realitytrace with jsonschema 0.60.2: all suites passed
- cargo clippy -p sea-forge-realitytrace --all-targets -D warnings: clean
- cargo test -p sea-forge-core --test yaml_strictness: 67 passed

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
- macOS ProcessGroup claim rests on a best-effort census; live egress allow-control policy shape: owner decisions pending on PR #71

**Next:** Merge, then the Horizon desk support, EGM README, ARM64 packaging and CI optimization PRs. Follow-up: drain the request head in every HTTP stub, then take tokio 1.54.

**Evidence:**

- .agents/reports/fde-convergence-2026-11-12/STATUS.md
- .agents/reports/fde-convergence-2026-11-12/H-egm-review-record.md
- .agents/reports/fde-convergence-2026-11-12/F-implementation-plan.md
- .agents/plans/2026-10-28-consolidated-final-report.md
- .agents/plans/2026-10-27-arm64-jetson-packaging.md
- .agents/plans/2026-10-26-capability-harvest.md
