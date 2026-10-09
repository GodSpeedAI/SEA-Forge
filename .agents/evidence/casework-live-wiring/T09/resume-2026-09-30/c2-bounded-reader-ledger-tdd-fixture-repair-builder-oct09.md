# Bounded ledger reader fixture repair: builder freeze

Date: 2026-10-09. This freezes a test-scaffold repair for independent source
review. No reader implementation or runtime result is claimed.

## Authority and identities

- Source grant: `c2-bounded-reader-ledger-slice-root-grant-oct09.md`, SHA-256
  `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`.
- ACK-boundary addendum:
  `c2-bounded-reader-ledger-slice-ack-boundary-addendum-oct09.md`, SHA-256
  `1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`;
  accepted design re-review SHA-256
  `8fdc7dfea99cf9e16985e33ec25db894b524cfa5fdbbcddd2a76df42d3e9753e`.
- Initial builder receipt SHA-256
  `cb61ff605e62a465dbdc1132a479414ce00655adaaa7b82741fd08a1a22ddf98`.
- Rejection review SHA-256
  `72511d706b21e7869c5235eb3bfeb63fe1155502dedbb28aeffc98eac9e9bd3c`.
- The frozen initial source archive was decoded and compared byte-for-byte
  before editing. Both decoded input and original source measured 96,079 bytes
  with SHA-256
  `40ac4fa2f04bef5f5b43940a134a2d99189787daf1f0375405f3fbac617ee2f8`.
- Repaired source: `crates/sea-forge-ledger/src/types.rs`, SHA-256
  `587619ce307c1ee8ed996e0cad0106691c1bb798ec75c880c68d4c848a7f8619`.
  Its exact 17,410-byte unified diff against the frozen source (11 hunks,
  three context lines) has SHA-256
  `0f1d8b7a315dee11966903f5fc44c2595e31c0daed30bf5f7a3e538b585eda5c`.
  Changed spans are confined to the `#[cfg(test)]` module, including test-only
  imports/helper; production reader and checked-arithmetic bodies remain the
  original explicit `todo!` stubs.

## Repair coverage

- Integrity rejection now covers each registered ledger/version/
  canonicalization/hash-algorithm constant with recomputed typed entry hash,
  wrong stored entry hash, nonzero origin ordinal, origin predecessor, ordinal
  gap, broken link, and bad payload hash. The compatibility vector still
  accepts an unknown top-level field, and now rejects a changed unknown payload
  member when its hashes are not recomputed.
- Resume vectors independently alter pin and acknowledged cursor, ordinal,
  entry hash, raw checksum, start offset, and end offset. A byte-reordered JSON
  object has equal length and deserializes to the same typed entry/hash but has
  a different raw checksum; resume rejects it both as the acknowledged
  predecessor and as the on-disk original pin. The successful resume vector
  also appends after the original pin and proves the resumed scan completes at
  that pin.
- The lock vector runs the reader in a worker with a finite response wait,
  starts that worker only after the helper signals successful lock acquisition,
  measures a tolerant 40 ms–1 s return window around the 50 ms deadline, and
  always releases and joins the lock holder before assertions. It then joins
  the reader worker before assertions. If a defective reader remains stuck
  after lock release, the join is bounded only by root's outer 180 s gate
  timeout; this fixture does not claim an independently bounded thread join.
- The aggregate cap-plus-one vector retains the previous validated-position
  assertion and requires only error or `LimitReached`; it no longer requires
  spending all residual bytes. The separate exact-cap 4 MiB vector remains.
  The row-boundary vector now proves exact 500-row head completion as well as
  501-row `LimitReached`.

The former “unreadable” test is accurately named
`bounded_reader_rejects_nonregular_and_path_io_failures`: it covers a
directory at the registered entries path and a missing-parent I/O failure. It
does not test permission-denied access to a regular file; that fixture remains
uncovered. No fake reader hook or permission policy was introduced.

## Freeze limits

The only removed assertion is the rejected cap-plus-one requirement that
`remaining_raw_bytes == 0`; prior checkpoint and bounded-outcome assertions
remain. No other original assertion was intentionally removed or weakened.
`types.rs` still contains 18 bounded-reader tests and the held
`bounded_reader`, `resume_bounded_reader`, `next_step`, and checked-add stubs.
No compiler, test, formatter, gate, dependency change, source behavior, schema,
server integration, or Git operation occurred. Root owns expected RED; the
independent critic must review this exact source and receipt before it.

CW-37 progress was updated in `.agents/DEBT.md`; its resulting SHA-256 is
recorded by the root after review. No other documentation or status file was
changed by this task.
