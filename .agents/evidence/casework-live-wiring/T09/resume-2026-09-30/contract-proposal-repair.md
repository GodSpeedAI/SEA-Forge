# T09 rejected contract proposal repair

Date: 2026-09-30

## Original assignment and rejected findings

Repair the rejected design proposal only; no implementation, build, compile, test, vet, Bun, or browser/runtime work. The independent review identified three gaps: the new execution_observation event could not pass the UI adapter listener allowlist/parser and would be discarded by ordinary cursor dedupe; the draft relied on intent-only rate limiting and the generic 1 MiB JSON cap for a durable Ask route; and the proposal pointed to the wrong canonical contract path and did not identify the actual kernel Ask request tag/schema artifact.

## Material differences

- The proposal now distinguishes catalog label thoth.ask from actual SFWP tag verb: ask (Request::Ask), documents the actor_id/asker field alias, and preserves verified gateway/session delegation.
- Observation frames use identity (run_id,event_id), a separate bounded dedupe ring, bypass comparison/update of case lastCursor, carry only an informational real case cursor, and have no SSE id. Reconnect re-reads run.get; current observations remain separate from immutable historical snapshots.
- Added concrete proposed poller/frame caps (16 active runs, 2 concurrent reads, 1-second minimum interval, 1,024 safe metadata frames) plus truncation counts, terminal/no-subscriber stop, and cancellation/drain. These are recommendations pending operator approval, not existing settings.
- Ask now lists the exact nine question kinds, subject/case validation, 500 UTF-8 byte purpose bound (from Rust str::len()), 8 KiB proposed request cap, and independent proposed session/IP limiter defaults (6/min burst 2; 20/min burst 4). It does not alter intent limiter accounting or assume generic POST limiting.
- Corrected the canonical source path to .agents/reports/interface-contracts/typescript/types.ts, verified existing schema/golden/conformance paths, and named the new authored schema .agents/reports/interface-contracts/schemas/thoth-ask.schema.json. The report contract directory has no schema generator.
- Preserved all ThothAnswerView fields and added exact disposition, freshness, claim-class, and claim-status wire enums.

## Result and verification

The proposal remains design-only and operator approval is pending. No source, normative spec, contract file, status file, dependency, or commit was changed. No test/build/vet/compile/Bun/browser/runtime command was run. Independent static critic review is pending.
