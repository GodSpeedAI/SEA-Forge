# Unit5 safe trace-port proposal v3: independent source review

Review result: **ACCEPTED AS A SOURCE-REVIEWED PROPOSAL ONLY**. This is not
fixture, implementation, compiler, runtime, or milestone approval.

Reviewed the original proposal, v2, the first independent review and its
anchor clarification, the v2 independent review, v3
`observation-safe-trace-port-root-proposal-v3.md` (SHA-256
`e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6`), and
current Rust, Go, and TypeScript source anchors.

## Findings

V3 preserves the original safe projection rules and resolves the identified
v2 deviations:

- Only the ten allowlisted kinds are projected. Unknown kinds, their IDs, and
  payloads are omitted and cannot invalidate selected rows. Duplicate IDs are
  checked among selected rows across the full returned array before retention,
  including selected duplicates beyond the newest-1024 boundary. This matches
  the original “duplicate selected event ID” rule and the ten-kind TypeScript
  contract (`types.ts:306-316`).
- Selected rows retain the original nonblank event ID, RFC3339 timestamp,
  source-order, and no-manufactured-identity/time constraints. The safe DTO
  remains limited to event ID, kind, timestamp, and the two optional
  command-finished fields (`types.ts:325-335`).
- Exact run/case/item ownership, allowed standings, blank-input rejection,
  authority-refusal preservation, separate port/decoder, unchanged artifact
  path, 1024 retention, JavaScript-safe integer bounds, and exclusion of raw
  payload fields remain in scope as in the original.
- V3 correctly removes the trace-array/byte-size consistency checks. Rust
  `get` reads trace rows before collecting record metadata
  (`run_views.rs:836-855,870-880`), so those facts are not an atomic snapshot.
  V3 uses record metadata only to require reported trace-file presence.
- V3 defines safe counts and cohort counts as returned-frame counts, not
  source-journal totals, and explicitly says empty returned rows do not prove
  an empty or fully parsed source journal. This matches the source: `read_jsonl`
  can return empty on cap/read failure and stops at a valid prefix after a
  malformed line (`run_views.rs:425-436`); `RecordPresence` supplies only
  filename, presence, and optional `u64` byte size (`:240-250`).
- V3 removes the v2 whole-payload object restriction. Missing, null, or scalar
  payload is treated as absent command metadata; supplied nonnull `execution`
  remains object-validated and supplied optional values retain their exact
  validation. Rust carries payload as arbitrary JSON `Value`
  (`run_views.rs:121-130`; core `types.rs:528-536`), and the frozen safe DTO
  has optional nonnull output fields (`types.ts:325-335`).

Requiring exactly one `trace.jsonl` record with `present: true` and a
nonnegative exact `u64` bytes representation is consistent with the earlier
review's presence repair and the current canonical record inventory
(`run_views.rs:85-95,240-250,870-880`). V3 correctly disclaims that bytes prove
trace completeness. No remaining material deviation from the original rules
or unsupported v2 restriction was found in the proposal text.

## Review correction

The v2 review's statement that v2 required a nonnull whole payload was
imprecise. V2 said “a nonnull `payload` must be an object”; it did not require
payload to be nonnull. V3 accurately removes that added restriction by
explicitly treating missing/null/non-object payload as absent metadata. This
clarification does not change the v2 review's findings about unknown-kind
duplicate rejection or byte/row consistency checks.

## Source anchors and limits

- Original and v3 proposals, plus all preceding reviews/clarifications, are in
  `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/`.
- Rust returned trace, parse behavior, presence metadata, and read ordering:
  `crates/sea-forge-server/src/sfwp/run_views.rs:85-95,121-130,240-250,283-326,425-436,836-855,870-880`.
- Core event payload and typed event kind:
  `crates/sea-forge-core/src/types.rs:478-536`.
- Frozen safe frame kinds, optional metadata, standings, and observation
  counts: `.agents/reports/interface-contracts/typescript/types.ts:306-372`.
- Existing SFWP `NewRunGet` and artifact provenance path:
  `apps/godspeed-casework-go/internal/adapters/sfwp/frame.go:308-313,686-694`;
  current adapter use: `authority.go:744-780`.
- Normative informational boundary and bounds:
  `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-105`.

This review accepts v3 as a coherent source proposal for later test-first
decomposition. It does not authorize a fixture write, production change,
compiler run, or runtime action. No tests/compiler were run and no source,
fixture, status, or Git state was changed.

Graft retrieval preceded direct source-anchor verification; one retrieval call
saved approximately 26,362 tokens.
