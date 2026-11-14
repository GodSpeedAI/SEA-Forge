# Current status

**Status revision:** 247

**Stage:** FDE review fixes: F1 F6 F7 settlement gates in the Field Case kernel (WP-14)

**Summary:** fde/integration: episode.settled needs a COMPLETE freeze (committed intent, case record, RealityTrace records, and the published anchor when a publisher is configured) and refuses a second settlement of the same episode key under another idempotency key (IDEMPOTENCY_CONFLICT, closest existing code); I-2 fails closed (missing or unreadable case-events.jsonl, event without actor_id, non-string plan_item_id all refuse), compares identities canonicalised (trim, placeholder refusal via check_identity_string, case-folded), the server passes the gateway delegation chain from the verified actor and an empty principal (refused) when no identity was verified; a replay of a freeze that crashed after the record now completes the committed mark.

**Verified:**
- cargo test -p sea-forge-server: lib 189, conformance_field_case 27, conformance_interaction_events 10, conformance_sfwp 23, conformance_identity 15, conformance_cep_authority 13, conformance_world_pin 1; 0 failed
- mutations seen failing then restored: complete-freeze gate off, repeat-settlement check off, executor read fail-open, canonicalisation off

**Limits:**
- Execution-side I-1 gate not wired: no entry point (item_execute, case_advance, agent_run.delegate) carries n or an episode_ref, only WP-20 adds it
- Settlement requires the anchor only when SEA_FORGE_FDE_ANCHOR_SCRIPT is configured
- The gateway hop is plumbed; the agent-run to delegator link for case items is not recorded anywhere

**Next:** F3, F4, F5, F8, F9/F10

**Evidence:**
- crates/sea-forge-server/src/sfwp/case_records.rs
- crates/sea-forge-server/tests/conformance_field_case.rs
- .agents/reports/fde-convergence-2026-11-12/evidence/s1/wp-14.md (in the sea-rs checkout)
