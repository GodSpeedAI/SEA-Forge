# Current status

**Status revision:** 232

**Stage:** Horizon desk support: plan digest, add_item dry_run, case read digests

**Summary:** case.preflight returns plan_precondition and case.commit checks it; case.add_item gains dry_run and plan: preconditions; case.get_overview and approval.list return a ready-made digest record; contracts/schema regenerated.

**Verified:**
- cargo build --release -p sea-forge-server; cargo test -p sea-forge-core (67 unit tests); cargo test -p sea-forge-server (573 tests, 0 failed); clippy has no new warnings

**Limits:**
- The case.add_item precondition is checked before the write, not under the same lock as the write
- contracts generated TypeScript under workbench/ is not regenerated here (needs bun)

**Next:** Docs for the new fields follow.
