# Current status

**Status revision:** 231

**Stage:** Fix review: document the attempts that have no launch record (#86)

**Summary:** Clear RUSTSEC-2026-0459 (libsqlite3-sys 0.38.2 -> 0.39.1 via rusqlite 0.40 -> 0.41) and RUSTSEC-2026-0486 (rustix 1.1.5 -> 1.1.6) so cargo deny passes again.

**Verified:**
- cargo test --workspace passes; clippy clean
- cep_settlement orphaned_command race test fails without the guard and passes with it
- Independent review: two rounds, APPROVE on round 2 (zombies are gone, read errors fail closed, a malformed record is Unknown)
- cargo-deny bans pass with the dev-dependency version

**Limits:**
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

**Next:** Merge after CI; release, then the owner reinstalls the Horizon gateway with the new binary and unit.
