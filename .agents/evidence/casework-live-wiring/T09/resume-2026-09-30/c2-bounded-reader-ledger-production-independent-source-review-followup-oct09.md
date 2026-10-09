# Bounded ledger reader production review: blocking follow-up

Date: 2026-10-09

This follow-up preserves the earlier source review and changes its disposition
after root raised three concrete safety concerns for independent assessment.

## Verdict

**REJECT pending fresh bounded repair and review.** All three findings below
are confirmed at the frozen implementation. They affect the grant's
pre-allocation byte bound, physical JSONL row validation on resume, and
fail-closed behavior after a caller continues a session following an error.
The test module and existing assertions remain byte-identical to the accepted
RED snapshot except for the approved unused-constant removal; this does not
cover these implementation paths.

## Frozen source and prior record

- Source `crates/sea-forge-ledger/src/types.rs`, SHA-256
  `3045c6d3b089e119ac2ddf9cb186249d726fa3e543da1ed8766223584b8e4425`.
- Builder receipt `c2-bounded-reader-ledger-production-builder-oct09.md`,
  SHA-256 `103cf4777a4ea2c08c716f8d131ceaa0195f781fc294f4f9f973102fda0a7bce`.
- Initial source review
  `c2-bounded-reader-ledger-production-independent-source-review-oct09.md`,
  SHA-256 `b3b118ffebdb2cea38647d4e90a1938d4e113cf1505ede63b2d19d9f6a8c9e7a`.
- Controlling grant:
  `c2-bounded-reader-ledger-slice-root-grant-oct09.md`, SHA-256
  `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`;
  ACK-boundary addendum SHA-256
  `1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`.

## Blocking findings

### 1. Resume allocates the claimed row before proving the remaining budget

At `types.rs:553-564`, `read_position_row` bounds the claimed length by the
2 MiB row cap. It then charges an optional preceding-boundary byte at
`types.rs:566-574`, but at `types.rs:578-583` it reserves and resizes the full
claimed row before `read_exact_counted` checks whether the remaining page
budget can supply those bytes. A valid caller position of up to 2 MiB can
therefore cause that allocation when only a small amount of the 4 MiB request
budget remains; the read then errors only after allocation. This violates the
grant's requirement to bound allocation/growth against the remaining raw-byte
budget before use. Establish that the boundary probe plus complete row fit the
remaining budget before allocating the row.

### 2. Resume validation does not require one physical LF-delimited row

`validate_bounded_entry` at `types.rs:502-529` checks that the byte slice ends
in LF and parses all preceding bytes as JSON. JSON whitespace permits interior
newlines. `read_position_row` accepts any range whose claimed start follows
LF and whose last byte is LF (`types.rs:566-587`), without rejecting another
LF inside that range. A caller can therefore present a position over a
pretty-printed, multi-line JSON object with valid typed hashes and matching
raw checksum; Serde parses it and the computed position matches. Such a range
is not a physical JSONL row. Forward scanning splits at the first LF, so this
resume-only bypass can accept bytes that the ordinary reader would reject.
Require resume positions to cover exactly one physical row (one terminating
LF, with no earlier LF in the claimed bytes).

### 3. A session can return a row after an earlier validation error

`next_step` propagates `read_forward_row`/validation errors with `?` and does
not poison the session (`types.rs:720-775`). `read_forward_row` advances the
file/read-buffer position when it extracts a complete line (`types.rs:782-796`),
while `next_offset` and the validated checkpoint update only after the later
checks succeed (`types.rs:749-774`). For a concrete case, start with a valid
ordinal-0 row, insert `garbage\n`, then append an otherwise valid ordinal-1
row linked to ordinal 0. The first call returns row 0. The next call consumes
`garbage\n` and returns a parse error. A subsequent call consumes row 1;
ordinal and predecessor checks still match row 0, so the method can return a
candidate position using the stale `next_offset`, which does not identify the
bytes actually parsed. The error already failed the page, but the API does not
make continued use terminal. Poison the session after any `Err` so later calls
cannot return `Row` or `Complete`.

## Boundary of this review

The source review remains static only. Root owns the compiler and any gates;
no compiler, tests, gates, Git, source, debt, or status operations were run by
this follow-up. The previously checked 50 ms monotonic lock loop, shared
4 MiB accounting model, row and scan caps, typed hashes, immutable pin, and
ACK-boundary scope do not resolve these three defects. A fresh source freeze
and independent review are required before further runtime approval.
