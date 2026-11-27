# Current status

**Status revision:** 288

**Stage:** Dependency security: url RUSTSEC-2026-0592

**Summary:** Bump url to 2.5.9 (and form_urlencoded to 1.2.3) in the lockfile to clear the Url::origin() stack-overflow advisory.

**Verified:**

- Lockfile-only change (url 2.5.8 to 2.5.9, form_urlencoded 1.2.2 to 1.2.3); no manifest changed.
- just deny now reports only RUSTSEC-2026-0543 (chrono unmaintained); advisories still fail until that policy decision, bans, licenses and sources are ok.
- cargo check --workspace --all-targets --locked passes; sea-forge-agent tests 43+3 pass and sea-forge-realitytrace tests 1+4+6 pass.

**Limits:**

- The chrono unmaintained advisory (RUSTSEC-2026-0543) remains and no deny.toml waiver was added; cargo-deny advisories and the Security workflow stay red until that policy decision.
- Full CI has not run on this branch.
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

**Next:** Merge this fix, then merge it into the integration branch; decide the chrono advisory policy separately.

**Evidence:**

- Cargo.lock
