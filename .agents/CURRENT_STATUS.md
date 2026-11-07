# Current status

**Status revision:** 231

**Stage:** Horizon desk support: external_ref and review marker

**Summary:** PlanItem gains external_ref (github_issue or ticket link) and ItemMarkers gains review; both are omitted from JSON when unset so existing plan hashes are unchanged.

**Verified:**
- cargo build --release -p sea-forge-server; cargo test -p sea-forge-core (67 unit tests); cargo test -p sea-forge-server (573 tests, 0 failed); clippy has no new warnings

**Limits:**
- The case.add_item precondition is checked before the write, not under the same lock as the write
- contracts generated TypeScript under workbench/ is not regenerated here (needs bun)

**Next:** Plan digest, add_item dry_run and digests on case reads follow.
