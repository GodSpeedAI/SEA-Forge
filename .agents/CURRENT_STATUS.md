# Current status

**Status revision:** 182

**Stage:** Horizon gateway: runtime socket access (opt-in socket_gid)

**Summary:** Branch horizon/socket-group-access adds optional socket_gid to server.yaml. Unset keeps the 0600 owner-only socket. Set publishes the staging socket as 0660 with that group before the rename. Unit tests cover both modes.

**Verified:**
- cargo test -p sea-forge-server --lib: 168 pass
- new socket_publication tests: 2 pass
- cargo clippy -p sea-forge-server --all-targets -D warnings: clean
- full cargo test -p sea-forge-server: all pass except t16_8 in conformance_m16, which also fails on origin/main (pre-existing)

**Limits:**
- Not yet verified live: the Horizon gateway must run this binary with a socket_gid before the runtime can connect
- PR not merged: merge is held until the live check passes

**Next:** Deploy the built binary to the Horizon gateway, check that the runtime connects, then merge the PR to main.

**Recorded at:** 2026-11-02T20:19:13Z
