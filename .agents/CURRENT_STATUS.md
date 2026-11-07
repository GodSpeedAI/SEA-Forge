# Current status

**Status revision:** 235

**Stage:** Horizon desk support: blind-critic fixes for the case keeper

**Summary:** Fixes eight review findings: dry_run replay bypass, add_item check-then-write race (case lock, precondition first), template read once for preflight and commit, no authority records from dry runs or stale rejections, overview digest consistent with its state, normalized dry-run item.

**Verified:**
- cargo build --release -p sea-forge-server; cargo test -p sea-forge-core (67 unit tests); cargo test -p sea-forge-server (573 tests, 0 failed); clippy has no new warnings

**Limits:**
- The case.add_item precondition is checked before the write, not under the same lock as the write
- contracts generated TypeScript under workbench/ is not regenerated here (needs bun)

**Next:** Hand to the Horizon desk adapter.
