# Supplementary security implementation coverage (candidate)

This is **not a frozen or complete supplementary acceptance version**. Preserve
v1 at external pin `af41ea2465e38c87dfe86471c33c6b54ea8564e1`, including its gap
inventory. Current additions cover local file witnesses, private-channel
authentication and the stdio MCP
path. Ten v1 integration obligations remain open; no candidate here discharges
workload authentication, credentials, approval lifecycle, active revocation,
trusted exposure, destination controls or delegation.

Implementation tests are in `crates/sea-forge-authority/tests/mcp_boundary.rs`,
`crates/sea-forge-sandbox/tests/observation_file.rs` and
`crates/sea-forge-server/tests/file_effect_observation.rs`. Parser unit tests
live in `src/mcp_protocol.rs` in the server crate. They supplement the immutable
resource-side v1 assertions; they do not replace them.

The operator-approved migration of the legacy approval positive control in
`crates/sea-forge-server/tests/conformance_identity.rs` uses real committed
identity-minting grants and independently issued private channels. It retains
successful approval and journal attribution and verifies authentication
evidence plus actor-switch, public impersonation and self-approval refusals.
This exercises production APIs; it does not establish an isolated workload
launcher, credential secrecy, revocation or host-process identity. It is not a
replacement frozen acceptance test.

Build the actual product broker with
`RUSTC_WRAPPER= cargo build --locked -p sea-forge-server --bin sea-forge-mcp`.
`provision_mcp.py <absolute-broker-path>` translates the frozen driver's trusted
provisioner inputs into product JSON configuration. The declarative policy
permits only `metadata.read` on `security-upstream`; all enforcement is in the
product authority/forwarding path. It contains no gateway simulator or mock.

Run from the repository root (use a fresh output directory):

```sh
export SEA_FORGE_SECURITY_PG_BINDIR=/tmp/sea-forge-pg0-v0.15.2/wrappers
export SEA_FORGE_SECURITY_MCP_COMMAND='["/usr/bin/python3","/home/sprime01/projects/sea-rs/tests/security-v2/provision_mcp.py","/home/sprime01/projects/sea-rs/target/debug/sea-forge-mcp"]'
RUSTC_WRAPPER= python3 tests/security-v1/run.py all --output .agents/evidence/security-v1/<new-run>/all
RUSTC_WRAPPER= python3 tests/security-v2/capture.py .agents/evidence/security-v2/<new-run>/test -- just test
RUSTC_WRAPPER= python3 tests/security-v2/capture.py .agents/evidence/security-v2/<new-run>/check -- just check
RUSTC_WRAPPER= python3 tests/security-v2/capture.py .agents/evidence/security-v2/<new-run>/proof -- just proof
```

The pg0 evidence README records the pinned test-only provisioner and wrappers.
Replace local paths when reproducing elsewhere. An overall nonzero v1 exit
remains required while its original gap rows are NOT-YET-EXECUTABLE. Do not
translate that exit into a green gate; report executable subcases and broader
contracts separately. Capture preserves actual command status and logs.

Before any supplementary freeze, add executable evidence for every remaining
obligation, obtain independent review, hash the complete suite and fixtures,
and retain its separate full Git commit outside implementation write access.
Mode-0600 files and caller-supplied actor names do not separate same-UID workloads.
