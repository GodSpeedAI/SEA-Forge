# Current status

**Status revision:** 248

**Stage:** FDE review fixes: F5 anchor receipt and order check, recover under the case lock (WP-14)

**Summary:** fde/integration: ScriptPublisher builds its receipt from what scripts/fde-anchor.sh prints (the script now also prints tag_object_id) and is bounded by a timeout; the push runs after the case lock is released (anchor_work, record_anchor_receipt); verify_freeze_anchor returns distinct states (store_integrity, no_freeze_record, anchor_absent, receipt_missing, malformed_run_event, no_run_events, push_not_before_first_run) and reports a local-clock push time as AnchorTimeLocalOnly, never externally verified; recover has a server entry point that takes the case lock.

**Verified:**
- cargo test -p sea-forge-server --test conformance_field_case: 30 passed, 0 failed; scripts/fde-anchor.sh verify fde-s1: OK
- mutations seen failing then restored: no-run-events Ok, publish timeout off, local time reported as external

**Limits:**
- The remote server push time cannot be read with plain git; every real receipt is AnchorTimeLocalOnly (no external time claim)

**Next:** F3, F4, F8, F9/F10 size caps

**Evidence:**
- .agents/reports/fde-convergence-2026-11-12/evidence/s1/wp-14.md (in the sea-rs checkout)
- crates/sea-forge-server/src/sfwp/case_records.rs
- scripts/fde-anchor.sh
