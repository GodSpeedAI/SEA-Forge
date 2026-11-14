# Current status

**Status revision:** 241

**Stage:** FDE integration branch: case keeper case.record.append, freeze and invariants (WP-14, part 1)

**Summary:** fde/integration: SFWP verb case.record.append (verb case_record_append) in sea-forge-server with a schema registry over schemas vendored by digest into crates/sea-forge-server/contracts/fde; kinds episode.frozen and episode.settled; responses and refusal codes per contracts/fde. Two-phase freeze (prepare writes an intent, the RealityTrace outbox and the anchor outbox; commit appends the hash-chained case record), recover() for a crash between phases, freeze_status/require_complete_freeze and verify_freeze_anchor. Server-side I-1 (settle needs the freeze; a freeze needs its RealityTrace records), I-2 (settling chain must not contain an executor actor of the plan item), I-3 (one freeze per (case, item, n)), idempotent replay, EXPLORATORY_NOT_SETTLEABLE. fde-proof.d wired for AT-04, NT-01, NT-16 (and NT-13, NT-17, NT-42 outside the S1 set).

**Verified:**
- cargo test -p sea-forge-server --test conformance_field_case: 18 passed, 0 failed
- cargo test -p sea-forge-server --lib --test conformance_sfwp --test conformance_identity: 187 + 15 + 23 passed, 0 failed
- mutation checks: removing the I-3 key check, the I-2 chain check or the I-1 FREEZE_MISSING check each makes conformance_field_case fail
- scripts/fde-proof.sh --slice s1 --only AT-04, NT-01, NT-16: PASS
- scripts/check-agent-context.sh passes

**Limits:**
- RealityTrace records are checked through an index file (realitytrace/records.jsonl) and owed through a durable outbox; no in-process RealityTrace writer or delivery agent exists
- The anchor is published only when SEA_FORGE_FDE_ANCHOR_SCRIPT names a publisher; the receipt server push time is the publisher's, not read from GitHub (RD-26)
- I-1 at delegation and governed-operation boundaries (NT-32, NT-33 delegation side) is a library gate (require_complete_freeze) not yet called by any delegation verb: the delegation payload carries no episode_ref until WP-20
- Signed interaction-event emission, Go client, TypeScript contracts and golden SFWP frames are not done in this commit
- Digests use a local RFC 8785 implementation (integers, strings, no floats); sea-forge-core canonical_json is a different profile
- Integration branch only: not merged to main, no PR

**Next:** WP-14 part 2: interaction-event outbox and signing, generated Go/TypeScript contracts and golden frames; WP-20 wires I-1 into delegation.

**Evidence:**
- crates/sea-forge-server/src/sfwp/case_records.rs
- crates/sea-forge-server/tests/conformance_field_case.rs
- scripts/fde-proof.d/_cargo-test.sh
