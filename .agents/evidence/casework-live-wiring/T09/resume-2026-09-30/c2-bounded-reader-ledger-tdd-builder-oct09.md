# C2 bounded ledger reader: test-first builder freeze

Date: 2026-10-09

## Status and authority

This record freezes the focused test-first slice for independent fixture
review. It does not release reader implementation. The ledger checkpoint is a
candidate position validated for ledger integrity only; the server/caller
still owns the separate `REQ-C2-RANGE-003` event-shape validation required
before ACK eligibility, frontier advancement, or frame emission.

Authority read before editing: the root grant
`c2-bounded-reader-ledger-slice-root-grant-oct09.md`, its ACK-boundary
addendum and independent re-review, the approved C2 supplement and parent
spec, revision-2 proposal plus revision-3/revision-4 overlays and reviews,
the known-field and resolved-filter clarifications, policy receipt, and the
Oct. 9 registration/input-accounting decision and its independent and
normative reviews. Root separately approved only the two tiny checked
arithmetic helper declarations noted below. Native `apply_patch` was used for
the only persistent edits.

## Frozen source identity

- `crates/sea-forge-ledger/src/types.rs`
- SHA-256: `40ac4fa2f04bef5f5b43940a134a2d99189787daf1f0375405f3fbac617ee2f8`
- Scope: bounded-reader record/API declarations, explicit unimplemented
  behavior stubs, checked arithmetic helper declarations, and the focused
  unit tests listed below. No other source file was edited.

The session counter `remaining_raw_bytes` is private production state. Tests
observe it only from the same-module test module; there is no configurable
limit, mutable global hook, alternate reader, or test-only production path.
The byte fixtures are real `entries.jsonl` files written with typed rows and
the existing payload/entry hash helpers. Exact row lengths are calibrated by
padding the payload string before writing those rows.

## Exact staged test names

1. `bounded_reader_returns_exact_typed_positions_and_distinct_completion`
2. `bounded_reader_preserves_unknown_top_level_and_hashed_payload_fields`
3. `bounded_reader_proves_absent_and_zero_byte_streams_empty`
4. `bounded_reader_rejects_whitespace_torn_and_malformed_history`
5. `bounded_reader_rejects_nonregular_and_unreadable_registered_files`
6. `bounded_reader_fails_closed_when_cooperative_lock_is_held`
7. `bounded_reader_accepts_exact_row_cap_and_rejects_cap_plus_one`
8. `bounded_reader_charges_pin_and_forward_reads_to_one_exact_page_budget`
9. `bounded_reader_stops_before_aggregate_budget_cap_plus_one_without_partial_ack`
10. `bounded_reader_scans_at_most_500_global_rows_including_non_events`
11. `bounded_reader_keeps_pin_immutable_when_a_supported_append_follows_pin`
12. `bounded_reader_resumes_from_original_pin_and_acknowledged_predecessor`
13. `bounded_reader_rejects_resume_after_predecessor_bytes_change`
14. `bounded_reader_rejects_corrupt_pin_and_resume_offset_arithmetic`
15. `bounded_reader_offset_addition_is_checked_at_native_u64_boundary`
16. `bounded_reader_rejects_broken_ordinals_linkage_and_typed_hashes`
17. `bounded_reader_never_returns_partial_row_or_treats_limit_as_complete`

## Coverage and fixture notes

- Position vectors compare exact byte start/end offsets, typed ordinal/ULID/
  entry hash and SHA-256 of raw row bytes including LF. The row outcome is a
  ledger-integrity candidate, not an ACK API.
- Compatibility vectors retain unknown top-level fields under typed Serde
  tolerance and verify that unknown payload fields remain in the payload used
  by the existing hash helpers.
- Empty vectors distinguish absent and zero-byte files from whitespace,
  torn, malformed, nonregular, and lock/path I/O failures. The lock vector
  uses an actual cooperative file lock and channels, without a sleep.
- Row-size vectors use exact `2 MiB` and `2 MiB + 1` raw rows, including LF.
  The exact aggregate vector rereads one exact-`2 MiB` row for pinning and
  forward scan, totaling `4 MiB`. The cap-plus-one fixture's minimum
  auxiliary-plus-forward input is `4 MiB + 1`; it verifies no partial
  candidate replaces the prior validated position and exhaustion is not
  completion. Tests observe the reader-owned remaining-byte counter so actual
  boundary probes are charged rather than assumed away.
- A 501-row non-event stream proves the forward cap yields `LimitReached`
  after 500 complete rows, not EOF. An append after session construction
  proves the session stays pinned to its original head.
- Resume vectors carry the original pinned head and acknowledged predecessor
  together; they cover successful resume, changed predecessor bytes, bad pin
  hash/checksum/offset, and bad acknowledged checksum/offset. Typed ordinal,
  predecessor-link, payload-hash, and entry-hash validation vectors are also
  staged.
- `checked_ledger_offset_add` and `checked_ledger_ordinal_add` are private,
  production-used helper declarations approved by root specifically for
  native `u64` boundary coverage. Their staged tests cover ordinary addition
  and `u64::MAX + 1`; neither helper is test-only.

## Stub strategy, exclusions, and execution state

`bounded_reader`, `resume_bounded_reader`, `next_step`, and both checked
arithmetic helpers currently use explicit `todo!` bodies. These are temporary
unimplemented stubs so the API and tests can be reviewed before behavior is
released. The expected Rust RED has not been run by this builder; root owns
the sole compiler and must independently inspect the fixtures, run the
focused tests, and confirm that failures come from the explicit reader stubs
before authorizing implementation. No compiler, test, formatter, gate,
Graft build, Git mutation, or status/debt edit was performed for this slice.

Event payload shape validation, cursor/token/filter DTOs, ACK advancement,
server integration, response-byte accounting, and T09 settlement remain
outside this ledger-source slice. No test or implementation claim is made
for those boundaries. Any fixture flaw or compile failure found by root
remains open until corrected and independently reviewed.
