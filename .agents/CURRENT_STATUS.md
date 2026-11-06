# Current status

**Status revision:** 229

**Stage:** Fix: refuse NotDeclared while the settlement command's process group is alive (#86)

**Summary:** The SWE_SEED command's process group, leader start time and boot id are recorded as a create-only launch record on the claimed attempt. Resolving that attempt as NotDeclared is refused with settlement_command_alive while the group exists, and with settlement_command_liveness_unknown when liveness cannot be read. The CLI adds --attest-command-gone (requires --not-declared) for the Unknown case only.

**Verified:**
- cargo test --workspace passes; clippy clean
- cep_settlement orphaned_command race test fails without the guard and passes with it
- Independent review: two rounds, APPROVE on round 2 (zombies are gone, read errors fail closed, a malformed record is Unknown)

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

**Next:** Merge after CI; release, then the owner reinstalls the Horizon gateway with the new binary and unit.
