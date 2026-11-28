# Current status

**Status revision:** 288

**Stage:** Chrono migration step 1: golden timestamp tests

**Summary:** Pin today's exact chrono timestamp bytes and parse acceptance with golden tests on main, before SEA moves to the time crate (RUSTSEC-2026-0543).

**Verified:**

- Eighteen golden tests pass on chrono and change no production code: authority 8, ledger 4, server 4, core 1, trace 1; fmt and clippy -D warnings on those five crates are clean.
- Pinned through public seams: obligation window_closes_at normalization (fractions 0, 1 ms, 1 us, 1 ns, 999999999 ns, other fractions, six offsets, leap second rendered as :60) and the accept/refuse set for 20 spellings; discrimination and verification-run time parsing; execution floor and closure stamps; approval expiry equal to the escalation commit instant plus the ttl to the nanosecond and the Z-seconds issued expiry; decision decided_at; ledger committed_at, checkpoint created_at, global checkpoint and witness signed_at shapes with signatures verifying from the bytes on disk, and the ULID millisecond prefix; correlation stamps stored verbatim and never parsed; case created_at; the approval inbox expired flag as a string comparison against a +00:00 now; generated id timestamps and trace timestamps.
- The recon's Millis+Z site (sfwp/case_records.rs:500) does not exist on main, so nothing there is pinned.

**Limits:**

- Not pinned through a public seam: postgres.rs Secs+Z session expiry (needs PostgreSQL; session_deadline is private), case_dispatch.rs and delegation.rs approval requested_at/expires_at and the other now-stamped wire sites (need full delegation flows), authority lib.rs approval grant expiry parsing, and the case-runner, CLI, thoth, settlement, sandbox and cell stamps.
- Now-derived stamps can only be pinned by shape and arithmetic, not by fixed instants; fixed instants are pinned only where the instant is an input (obligation claims and runs).
- The chrono leap-second rendering (:60) cannot be represented by the time crate and needs an explicit decision in the migration.
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

**Next:** Migrate to the time crate with these tests unedited (DomainForge, then RealityTrace, then SEA).

**Evidence:**

- crates/sea-forge-authority/tests/timestamp_golden.rs
- crates/sea-forge-ledger/tests/timestamp_golden.rs
- crates/sea-forge-server/tests/timestamp_golden.rs
- crates/sea-forge-core/tests/timestamp_golden.rs
- crates/sea-forge-trace/tests/timestamp_golden.rs
