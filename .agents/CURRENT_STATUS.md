# Current status

**Status revision:** 238

**Stage:** FDE integration branch: contracts package S1, SEA-Forge contracts (WP-11)

**Summary:** fde/integration: added contracts/fde with frozen-reference.v1.json (episode_ref digest input = SHA-256 over RFC 8785 of body), anchor.v1.json (fde-anchor tag and verifier receipt) and case-record-append.v1.json (request, response, error tables; kinds episode.frozen and episode.settled; refusal codes FREEZE_KEY_EXISTS I-3, FREEZE_MISSING and FREEZE_INCOMPLETE I-1, SELF_SETTLEMENT_REFUSED I-2, IDEMPOTENCY_CONFLICT), a README of field and error-code tables, fixtures with eight refusal scenarios, a DIGESTS manifest and scripts/check-fde-contract-digests.sh. Schemas only: no runtime behaviour.

**Verified:**
- cargo test --locked -p sea-forge-realitytrace --test fde_contracts: 7 passed, 0 failed (3 valid-and-invalid fixture sets, scenarios, README codes, DIGESTS, drift detection)
- cargo clippy --locked -p sea-forge-realitytrace --test fde_contracts -- -D warnings: clean
- scripts/check-fde-contract-digests.sh: 3 contract digests match
- scripts/check-agent-context.sh passes

**Limits:**
- The refusals for I-1, I-2, I-3 and idempotency are specified and have scenario fixtures; nothing enforces them yet (WP-14, WP-21)
- The anchor errors ANCHOR_PREDATES_FAILED and MODEL_ANCHOR_LATE need a ledger and a remote and are specified, not tested
- The test digest check canonicalises ASCII-only integer/string values itself; the RFC 8785 rule is verified against a full library in the CEP repository (CEP-0008 section 16.5 vectors)
- The evaluate facts and output schemas are WP-12 and are not provided
- The test lives in sea-forge-realitytrace because it already carries jsonschema; no dependency was added
- Integration branch only: not merged to main, no PR

**Next:** WP-12 evaluate schemas; WP-14 case keeper enforces the refusals against the scenario fixtures; WP-20 adds the other four kinds.
