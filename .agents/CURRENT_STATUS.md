# Current status

**Status revision:** 246

**Stage:** FDE review fixes: F2 torn tail and corrupt lines in the Field Case store (WP-14)

**Summary:** fde/integration: case.record.append repairs a torn trailing fragment (cut to the last newline, noted durably in field-repairs.jsonl with the dropped text and digest) before every append, and refuses with a store-integrity error, writing nothing, when any records, intents, decisions, RT-outbox or anchor-outbox line is an unparseable complete line; freeze_status, verify_freeze_anchor, recover and deliver_anchors fail closed on the same check.

**Verified:**
- cargo test -p sea-forge-server --test conformance_field_case: 20 passed, 0 failed
- mutation: repair_torn_tail removed and integrity check removed, each fails the new test, restored

**Limits:**
- Single-process lock only: a second server process appending to the same case is not serialised by this layer
- Reads that feed no invariant (interaction outbox, spool, RealityTrace index) stay lenient

**Next:** F1 (I-1 wiring), F6, F7, F3, F4, F5, F8, F9/F10

**Evidence:**
- crates/sea-forge-server/src/sfwp/case_records.rs
- crates/sea-forge-server/tests/conformance_field_case.rs
- .agents/reports/fde-convergence-2026-11-12/evidence/s1/wp-14.md (in the sea-rs checkout)
