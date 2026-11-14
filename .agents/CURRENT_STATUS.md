# Current status

**Status revision:** 242

**Stage:** FDE integration branch: interaction-event outbox for case.record.append (WP-14, part 2, step 1)

**Summary:** fde/integration: every case ledger record and every I-2 authority decision written by case.record.append yields exactly one CEP execution_trace envelope carrying extensions[godspeed.interaction_event] (schema vendored by digest into crates/sea-forge-server/contracts/cep). The event is derived from the stored record (records now carry recorded_at; I-2 decisions are stored as dec-NNNN in field-decisions.jsonl), enqueued right after the ledger line into field-interaction-outbox.jsonl, rebuilt by reconcile_events if the enqueue is lost, and delivered idempotently (default sink: interaction-events/<case>.jsonl). Emitter failure never blocks the case write. semantic_ref comes from the WP-13 map (Episode, evidence::Settlement, no_self_settlement); a world the map does not cover records an unmapped gap row and invents nothing. Events are unsigned (E 5, before WP-33).

**Verified:**
- cargo test -p sea-forge-server --test conformance_interaction_events: 10 passed, 0 failed
- cargo test -p sea-forge-server --test conformance_field_case: 18 passed, 0 failed
- mutation check: disabling the outbox dedupe fails conformance_interaction_events
- scripts/check-agent-context.sh passes

**Limits:**
- Unsigned: the schema has no unsigned marker, so the emitter object is omitted and the outbox row records signature absent_before_WP-33
- No network transport: delivery appends to a local spool for RealityTrace ingest (WP-16)
- The authority decision on this path is the I-2 check (policy no_self_settlement); a freeze has no authority decision and reports not_required
- actor.role is the SEA-Forge identity role, not a SOP role
- Go client, TypeScript contracts, golden frames, SFWP spec note and AT-05 wiring are later steps
- Integration branch only: not merged to main, no PR

**Next:** WP-14 part 2 steps 3-6: generated contracts and golden frames, SFWP note, AT-05 (PARTIAL until WP-15).

**Evidence:**
- crates/sea-forge-server/src/sfwp/interaction_events.rs
- crates/sea-forge-server/tests/conformance_interaction_events.rs
- crates/sea-forge-server/contracts/cep/DIGESTS
