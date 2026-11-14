# Current status

**Status revision:** 239

**Stage:** FDE integration branch: WP-11 review amendments to contracts/fde

**Summary:** fde/integration: applied the WP-11 review to contracts/fde (still v1, unconsumed): frozen-reference body carries a required comparison_profile {id, sha256}; falsifiers are non-empty for evaluated and present-but-possibly-empty for exploratory; case.record.append refuses episode.settled S or F for an exploratory episode with EXPLORATORY_NOT_SETTLEABLE (no payload change); the test canonicaliser now errors on non-ASCII or non-integer input. DIGESTS, README, fixtures and three new scenarios regenerated.

**Verified:**
- cargo test --locked -p sea-forge-realitytrace --test fde_contracts: 8 passed, 0 failed
- scripts/check-fde-contract-digests.sh passes
- scripts/check-agent-context.sh passes

**Limits:**
- The refusals are specified with scenario fixtures; nothing enforces them yet (WP-14, WP-21)
- Full RFC 8785 conformance is tested in the CEP repository only
- Integration branch only: not merged to main, no PR

**Next:** WP-12 evaluate schemas; WP-14 case keeper enforces the refusals; WP-20 adds the other four kinds.
