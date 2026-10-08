# Unit5 safe trace-port proposal v2: independent source review

Review result: **REJECT** (proposal only; no fixture, implementation, or runtime approval).

Reviewed original proposal
`observation-safe-trace-port-root-proposal.md`, v2
`observation-safe-trace-port-root-proposal-v2.md` (SHA-256
`4a53f3e30ecca6802082016b5157e0e85b7cc049842eccf3981684d0e05e1d03`), first
independent review and anchor clarification, current TypeScript contract, current
Go SFWP adapter, and current Rust run view. This is decomposition/source review
only; it does not approve implementation.

## Blocking deviation: unknown-kind rows now invalidate selected IDs

The original proposal says to select the ten canonical kinds, omit unknown kinds
entirely (including payloads), and require “no duplicate selected event ID”
across the full array, including rows later omitted by retention. V2 changes the
last rule: it requires duplicate checking *before kind filtering* and says a
selected ID repeated in an unknown-kind row invalidates the read. The new test
case explicitly covers that newly rejected response.

That is a material semantic change. “Selected event ID” scopes uniqueness to
rows selected for safe projection; the original separately says unknown kinds
are omitted entirely. The original retained-row clause extends checking past
the retention cutoff, not past kind selection. Current Rust serializes the
trace kind as a string (`crates/sea-forge-server/src/sfwp/run_views.rs:121-129`)
and its core event kind is a typed enum (`crates/sea-forge-core/src/types.rs:528-536`);
the frozen UI frame-kind union is the ten-item allowlist
(`.agents/reports/interface-contracts/typescript/types.ts:306-316`). Nothing in
those sources requires an unknown kind to poison an otherwise safe selected
frame. V2’s own sentence that unknown rows “otherwise contribute no safe frame”
does not reconcile its earlier rejection rule.

Required correction: preserve full-returned-array duplicate checking among
allowlisted/selected rows, including selected rows later omitted by retention;
omit unknown-kind rows without inspecting their payload or ID for this rule.
Alternatively, an explicit authorized change to the original contract is
needed. Without one, this deviation alone blocks approval.

## Other material changes and source fit

1. **Trace-record metadata is a justified repair only when limited to presence.**
   The prior review requested using `records` to reject a reported absent
   `trace.jsonl`, and the clarification allows presence metadata while warning
   it does not prove parsing completeness. V2 correctly scopes counts to the
   returned `trace` array and disclaims source completeness. Rust's
   `RecordPresence` has `record`, `present`, and optional `bytes`
   (`run_views.rs:240-250`); its canonical inventory contains `trace.jsonl`
   (`:85-95`). Requiring one matching entry and `present: true` is consistent
   with the prior review's requested absent-file guard. The record list is
   populated independently after trace reading (`:836-855`, `:870-880`),
   however, so it cannot be treated as an atomic fact about the returned rows.

2. **Zero/nonzero byte cross-checks are unsupported and race-sensitive.** V2
   newly rejects empty returned rows with positive bytes and nonempty returned
   rows with zero bytes. The prior anchor clarification expressly says size is
   not a parse-status signal: a present journal may still produce an empty
   array after read failure/over-cap handling or a valid prefix after malformed
   JSONL (`run_views.rs:425-436`). More directly, `get` reads the trace first,
   then later calls `metadata` to build `records` (`:836-855`, `:870-880`);
   there is no atomic snapshot. Concurrent file change can therefore make
   bytes describe a different instant than the array. Presence and bytes may
   be preserved as metadata, but this cross-check is neither proof of empty
   trace nor a source contract and can spuriously reject a returned array.
   Remove these consistency rejection rules unless a separate source contract
   authorizes them.

3. **Non-null `command_finished.payload` must be an object is an added
   validation rule.** Original rules name missing/null `execution`, `status`,
   and `exit_code` as absence, require nonnull `execution` to be an object, and
   say other payload metadata is ignored. V2 additionally rejects a missing or
   null whole payload. Rust models payload as arbitrary `serde_json::Value`
   (`run_views.rs:121-129`; core `TraceEvent` at `types.rs:528-536`), while the
   frozen safe-frame contract specifies only projected fields and their
   optionality (`types.ts:325-335`). No cited source establishes nonnull object
   payload as a required shape for every command-finished row. This narrows
   acceptance beyond the original proposal; remove it or cite an authorized
   existing invariant and preserve the original missing/null behavior.

4. **Record bytes’ exact-integer rule is representable but not a completeness
   signal.** Rust emits `Option<u64>` and JSON integer bytes. A decoder may
   validate that representation if it chooses to consume bytes, but this does
   not resolve the source-read ambiguity documented above and should not be
   presented as evidence of trace completeness.

## Confirmed preserved rules

V2 retains the separate semantic port/decoder, exact expected ownership,
authority refusal preservation, safe-kind projection, selected-row ID and
timestamp checks, source order, 1024-row retention, returned-array count
boundary, and narrow command metadata projection. The ten kinds, five command
statuses, and optional `execution_status`/`exit_code` fields match the frozen
TypeScript contract (`types.ts:306-335`). Its treatment of the returned trace
as a prefix-capable projection, with no source-total or source-completeness
claim, correctly addresses the first review and clarification. The existing Go
artifact path uses one `Client.Do` for `NewRunGet` and a separate
`RunArtifactProvenanceView` (`apps/godspeed-casework-go/internal/adapters/sfwp/authority.go:758-780`);
the proposal's separate trace decoder preserves that boundary.

## Verdict and next proposal edits

Do not approve this v2 for fixture release or implementation. At minimum:

- restore unknown-kind omission before duplicate-ID validation, while keeping
  duplicate detection across all selected rows before retention;
- remove trace-array/byte-size consistency rejection because the record
  metadata is non-atomic and does not prove parse completeness;
- remove the new whole-payload object requirement unless its separate source
  contract is established and authorized.

Keep the repaired returned-row count and explicit source-completeness
limitation. The v2 changes above are substantive, not merely elaborations of
the original proposal or the prior review's requested presence/prefix repair.
No approval is granted absent proof that these added restrictions are required
by an existing authorized contract.

Graft retrieval preceded source inspection. No tests, compiler, implementation,
fixture, status, or Git commands were run; no source files were modified.
