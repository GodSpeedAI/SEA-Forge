# Bounded ledger reader production builder receipt

Date: 2026-10-09

## Frozen source

- Source: `crates/sea-forge-ledger/src/types.rs`
- SHA-256: `3045c6d3b089e119ac2ddf9cb186249d726fa3e543da1ed8766223584b8e4425`
- TDD baseline: root-frozen `6a22fbd14cafb5312cf81f2b2b694cb2413fbe21afaf35dcc4263a27d25e0c33`; the accepted RED run is recorded in `c2-bounded-reader-ledger-red01-progress-oct09.md`.
- Test scope: retained all 18 accepted RED tests and their assertions; removed only the unused `READER_PAGE_BYTES` test declaration as root specifically authorized. No test or compiler command was run for this production freeze.

## Implementation

Added the bounded reader API, session state, row positions, and resume input in
`types.rs`. Fresh reads acquire the registered stream's cooperative lock,
sample the entries file length, and pin/validate the terminal LF row with
fixed 8 KiB reverse reads. Resume validates the supplied original pin and
acknowledged row bytes, typed hashes, boundaries, and checksum under that same
lock. Both auxiliary phases charge actual reads against the session's one
4 MiB budget. The lock is released before the caller scans forward.

Forward scanning uses fixed 8 KiB reads, bounded row allocation, a 2 MiB raw
row cap including LF, and the same remaining byte budget used while pinning or
resuming. It validates typed ledger identity and existing payload/entry hashes,
then checked ordinal and predecessor linkage. It returns distinct row,
complete, and limit-reached outcomes and caps forward complete rows at 500.
Completion requires the scanned terminal row to match the immutable pin.
Unknown top-level serde fields remain accepted; payload fields remain part of
the existing typed payload hash. Raw-row SHA-256 is a private continuation
boundary checksum, not a replacement entry hash or an event-validity claim.

Root-approved private arithmetic helpers `checked_ledger_offset_add` and
`checked_ledger_ordinal_add` are used in production position/read/linkage
calculations. Limits are fixed constants; no test hooks, override, alternate
budget, dependency, schema, core error variant, or server/token behavior was
added. Candidate positions express ledger integrity only and do not themselves
acknowledge rows or establish event validity.

## Test vectors retained

The 18 root-accepted behavioral tests are:

- `bounded_reader_returns_exact_typed_positions_and_distinct_completion`
- `bounded_reader_preserves_unknown_top_level_and_hashed_payload_fields`
- `bounded_reader_proves_absent_and_zero_byte_streams_empty`
- `bounded_reader_rejects_whitespace_torn_and_malformed_history`
- `bounded_reader_rejects_nonregular_and_path_io_failures`
- `bounded_reader_fails_closed_when_cooperative_lock_is_held`
- `bounded_reader_accepts_exact_row_cap_and_rejects_cap_plus_one`
- `bounded_reader_charges_pin_and_forward_reads_to_one_exact_page_budget`
- `bounded_reader_stops_before_aggregate_budget_cap_plus_one_without_partial_ack`
- `bounded_reader_scans_at_most_500_global_rows_including_non_events`
- `bounded_reader_keeps_pin_immutable_when_a_supported_append_follows_pin`
- `bounded_reader_resumes_from_original_pin_and_acknowledged_predecessor`
- `bounded_reader_rejects_resume_after_predecessor_bytes_change`
- `bounded_reader_rejects_valid_typed_resume_byte_changes`
- `bounded_reader_rejects_corrupt_pin_and_resume_offset_arithmetic`
- `bounded_reader_offset_addition_is_checked_at_native_u64_boundary`
- `bounded_reader_rejects_broken_ordinals_linkage_and_typed_hashes`
- `bounded_reader_never_returns_partial_row_or_treats_limit_as_complete`

## Verification boundary

No compiler, test, formatter, gate, status, or Git mutation was run by this
builder after release. This receipt reports the frozen source and test scope;
it makes no claim that the production implementation compiles or passes tests.
Root retains sole compiler ownership and the required independent source review.
