# Live cursor V4 complete candidate revision 2 — independent review

Date: 2026-10-08  
Reviewed candidate SHA-256: `f0d7611730c4282699543a8ced8e16209695601bcdb1ea1982c126cc0b7f757c`.  
Verdict: **REJECT for exact operator approval; retain HOLD.** The four corrections requested by root are materially present, but the candidate still leaves architecture decisions open and one restart/stale-intent boundary unresolved. This is a document-only review; no implementation or runtime proof is approved.

## Scope and lineage

I read the complete original assignment (`00e15701923138ffdf47c3672f9f270f955f09d320489d1c50defc8215eaf254`), base candidate (`ffdfbdb38a9e435633a7f6e9b6dcbe79fdc591fa4302f8c712501eef9b4ac0b5`), root review (`f7da23097fe1f947fc868ada51c65eae217d2127ef0116a5f824ee512c9670c0`), and revision 2. I also checked the governing casework spec, C-2/D-2 decision log, T09 plan, V4 bootstrap revision-3 proposal/review, V3 proposal/rejection, and source/schema anchors. The revision-2 SHA above matches the assigned candidate. The three source/doc corrections below are material blockers; no test or build was run.

## Corrections that are adequately addressed

1. **Lossless ordinal wire:** revision 2 specifies canonical decimal strings across JSON, checked `u64` parsing/overflow, and TypeScript `BigInt` ordering (candidate §2). This fixes the base’s unsafe JSON-number/`Number` contract while preserving opaque event IDs.
2. **Replay versus live delivery and accounting:** candidate §3 keeps bounded immutable replay separate from the two-frame live queue, defines aggregate canonical-byte charges through in-flight writes and cleanup, and disclaims heap/RSS equivalence.
3. **Invalidation and response writes:** candidate §3 provides generation checks and locked permits before headers and each frame, keeps network I/O/cancel outside the mutex, and states the single permitted in-flight-write residual rather than promising instantaneous revocation.
4. **Restart capture identity:** candidate §4 derives a digest from the public snapshot without minting event IDs or replaying invented historical Facts. It identifies the new snapshot, historical GET, SSE reconnect, schema, and client changes that require exact approval.

These corrections are proposals, not runtime evidence. The revision preserves the revision-3 bootstrap/create preflight boundary and the D-2 identity rules (candidate §1; decision log D-2). Its all-writer HOLD remains explicit in §5, as required.

## Blocking findings

### 1. Root architecture choices remain unresolved

Candidate §§7 and 8 (revision 2 lines 96–102; base §8) explicitly leave root to choose the recovery ceiling, resource caps and unavailable behavior, legacy ID admission, error vocabulary, mutations without a global event, concurrent no-case semantics, `events.get_range` compatibility/versioning, the JCS profile, and the write-deadline residual. These are behavioral and public-interface choices, not merely implementation details. The governing spec requires bounded reconciliation/recovery without invented history (`.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:130-138,501-507`), while C-2 says public contract/runtime/version changes remain stopped pending reviewed semantics (`.agents/reports/casework-live-wiring/decision-log.yaml:113-131`). The candidate cannot yet be presented as one exact operator decision.

**Required next step:** root resolves or explicitly rejects each listed architecture choice, then updates the candidate’s approval set so the operator is asked to decide only the concrete remaining policy choices. Preserve the all-writer HOLD; it must not be represented as proven by this review.

### 2. Capture-digest resync does not invalidate same-cursor stale intents

Candidate §4 says a process restart can recapture different public Facts at the same `(caseID,cursor,ordinal)`, uses `capture_digest` to reset historical GET/SSE state, and explicitly says the digest does not authorize a mutation (lines 70–74, 92). Candidate §2 keeps `InteractionIntent.client_cursor` as only the exact event cursor (line 28). The existing gateway stale guard compares only that cursor: `stalenessRefusal` rejects when `cursor != in.ClientCursor` and otherwise proceeds (`apps/godspeed-casework-go/internal/intents/intents.go:437-462`); `InteractionIntent` has no capture digest (`:93-103`). Therefore, after restart, a request based on the old capture can carry the same exact cursor and pass the projection stale check even when its digest differs from the new baseline. An SSE/GET resync eventually clears the client cache, but the proposal does not establish that resync wins before a queued or direct old intent reaches the gateway.

**Required root decision:** either bind the digest (or an equivalent exact capture identity) into the stale-intent precondition and specify its untrusted-client validation, with the resulting public contract approval; or narrow the guarantee to history/cache resynchronization and explicitly accept that same-cursor old intents are not distinguished by the gateway. Do not leave the current text implying both restart-safe capture identity and cursor-only stale checks without stating this residual.

### 3. Strict inventory needs a rule for directories without case records

Base §4 makes a missing/partial `case.json` in a case directory an inventory error. Current Rust `case_views::list` deliberately treats a directory with no `case.json` as “not a case” while noting it may be partially created (`crates/sea-forge-server/src/sfwp/case_views.rs:310-341`). The candidate does not define how the strict inventory distinguishes an unrelated directory from a partial case directory, or whether every child under `cases/` is guaranteed to be a case ID. That distinction determines whether a list is complete-empty/complete or unavailable and affects the revision-3 bootstrap gate.

**Required root decision:** define which directory names are inventory members and how an absent record is classified, including the actual/legacy ID grammar and partial create state. Keep the existing honest failure behavior; do not silently reinterpret current omission as complete inventory.

## Contract alignment and implementation boundary

The source anchors confirm the need for the proposed changes: `events::get_range` materializes `read_entries()` before applying its 500-frame cap (`crates/sea-forge-server/src/sfwp/events.rs:169-208`; `crates/sea-forge-ledger/src/types.rs:784-801`); `ServerState::publish_event` runs append work on blocking tasks and then broadcasts (`crates/sea-forge-server/src/lib.rs:208-234`); current Go `Store.Subscribe` puts replay into a channel sized from replay length (`apps/godspeed-casework-go/internal/projection/store.go:193-227`); SSE writes 200/hello before checking old-cursor retention (`apps/godspeed-casework-go/internal/server/server.go:293-376`); `LiveSource.Facts` fetches current authority views and the builder captures a current timestamp/stable `world-<caseID>` (`apps/godspeed-casework-go/internal/projection/live.go:43-140`; `internal/projection/builder.go:54-129`). The existing schemas also conflict with the actual ID/cursor producers (`.agents/reports/interface-contracts/schemas/cognitive-world.schema.json:19-37`; `interaction-intents.schema.json:7-56`).

The future matrix in candidate §6 is appropriately labeled unrun. The proposal also correctly states serialized-byte counters are not an RSS proof and does not claim filesystem inventory or `Facts` are atomic with the event head. V4 bootstrap’s earlier DOCONLY acceptance does not lift its implementation/no-case-success HOLD, and the V3 complete contract remains rejected (candidate §1, lines 20; V4 bootstrap review; V3 independent review). No public source, schema, test, generated artifact, or runtime behavior should be released from this review.

## Disposition

Retain C-2 HOLD. Revision 2 fixes the four root-review gaps, but it is **not ready for exact operator approval** until root resolves findings 1–3 or explicitly narrows the candidate’s claims and approval scope. The all-writer audit remains an explicit implementation-readiness HOLD. No source release or runtime proof is authorized.
