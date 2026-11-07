# Current status

**Status revision:** 234

**Stage:** Horizon desk support: regenerated TS contracts

**Summary:** workbench/packages/contracts/generated is regenerated for the plan digest, add_item dry_run, external_ref and the case read digests (14 files).

**Verified:**
- cargo build --release -p sea-forge-server; cargo test -p sea-forge-core (67 unit tests); cargo test -p sea-forge-server (573 tests, 0 failed); clippy has no new warnings

**Limits:**
- The case.add_item precondition is checked before the write, not under the same lock as the write
- contracts generated TypeScript under workbench/ is not regenerated here (needs bun)

**Next:** Hand to the Horizon desk adapter.
