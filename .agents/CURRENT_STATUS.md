# Current status

**Status revision:** 233

**Stage:** Horizon desk support: protocol reference

**Summary:** The SFWP protocol reference now covers plan: refs, case.add_item (request_id, dry_run, preconditions, external_ref), the review marker and where a client reads each precondition digest.

**Verified:**
- cargo build --release -p sea-forge-server; cargo test -p sea-forge-core (67 unit tests); cargo test -p sea-forge-server (573 tests, 0 failed); clippy has no new warnings

**Limits:**
- The case.add_item precondition is checked before the write, not under the same lock as the write
- contracts generated TypeScript under workbench/ is not regenerated here (needs bun)

**Next:** Hand to the Horizon desk adapter.
