# Bounded ledger reader fixture repair: independent source review

Date: 2026-10-09

## Verdict

**REJECT as not yet source-ready for root RED.** The repaired fixture closes the
prior integrity, resume-byte-binding, residual-budget, and lock-holder race
findings. One required boundary remains insufficiently tested: the lock test
allows a reader with a 1-second wait to pass even though the grant sets a
monotonic 50 ms deadline. This review does not run the compiler, tests, or
gates, and makes no implementation or runtime claim.

## Reviewed identities and authority

- Root grant: `c2-bounded-reader-ledger-slice-root-grant-oct09.md`, SHA-256
  `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`.
- ACK-boundary addendum:
  `c2-bounded-reader-ledger-slice-ack-boundary-addendum-oct09.md`, SHA-256
  `1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`; its
  accepted design review is SHA-256
  `8fdc7dfea99cf9e16985e33ec25db894b524cfa5fdbbcddd2a76df42d3e9753e`.
- Repaired source `crates/sea-forge-ledger/src/types.rs`, SHA-256
  `587619ce307c1ee8ed996e0cad0106691c1bb798ec75c880c68d4c848a7f8619`.
- Repair builder receipt:
  `c2-bounded-reader-ledger-tdd-fixture-repair-builder-oct09.md`, SHA-256
  `3baddf1ee571964f47204dd7ad137a1fe0cb84d27d6b311e823164779456bf3d`.
- The source is byte-identical to root's frozen repair snapshot. The receipt's
  unified-diff identity against the original frozen source is 17,410 bytes,
  11 hunks, three context lines, SHA-256
  `0f1d8b7a315dee11966903f5fc44c2595e31c0daed30bf5f7a3e538b585eda5c`;
  root independently reproduced that diff metadata.

The grant and addendum remain controlling. In particular, reader positions
prove ledger integrity only; the server/caller must validate every event row's
known payload shape before filtering or ACK/frontier/frame advancement,
including filtered-out rows. That boundary is correctly left out of this
ledger-only test scaffold.

## Findings

### Blocking: lock test does not establish the granted 50 ms limit

At `types.rs` in `bounded_reader_fails_closed_when_cooperative_lock_is_held`,
the test correctly waits for the holder's lock-acquired signal before starting
the reader worker, uses a bounded result receive, and releases and joins both
threads before assertions. It asserts failure and an elapsed duration from
40 ms through 1 second. The grant requires a monotonic 50 ms lock deadline.
Consequently, an implementation that waits hundreds of milliseconds (up to a
second) before failing passes this test. The fixture demonstrates eventual
fail-closed behavior under contention, but does not discriminate the required
deadline from a substantially longer one. Tighten the upper bound enough to
reject materially late waits while retaining scheduler tolerance; retain the
ready-before-worker ordering and release/join-before-assertion cleanup.

## Findings resolved by this repair

- The integrity vectors now cover protocol constants with recomputed typed
  hashes, invalid origin ordinal and predecessor, ordinal gap, broken link,
  payload-hash corruption with a rehashed entry, and wrong stored entry hash.
- Resume corruption vectors cover pin and ACK positions, offsets, ordinal,
  entry identity/hash, checksum, and arithmetic overflow. Same-length JSON
  object-key reordering preserves the typed entry/hash and row boundary while
  changing raw bytes; resume rejects it for both the acknowledged predecessor
  and original pin. Successful resume retains the original pin despite a
  later append.
- Unknown top-level fields remain accepted. A modified unknown payload member
  without recomputed hashes is rejected.
- The cap-plus-one aggregate-budget assertion no longer requires spending all
  residual bytes; it retains the prior validated-position check and only
  accepts error or `LimitReached`. Exact-cap budget coverage remains.
- The row cap proves both exact 500-row completion and 501-row
  `LimitReached`, with the validated position at row 499.
- The unreadable test was accurately renamed to describe its actual directory
  and missing-parent I/O vectors. Permission-denied access to a regular file
  remains uncovered and is explicitly reported by the builder; it is not a
  grant requirement that can be reliably established in a root test process.
- Comparing the frozen initial source and repaired source confirms the change
  is confined to the test module. The only removed assertion is the invalid
  cap-plus-one requirement that the residual byte counter equal zero. The
  approved production declarations/helpers remain stubs; this is not runtime
  implementation evidence.

## Limits and next action

The source scaffold is otherwise within the grant: no dependency, schema,
error variant, server integration, wire DTO, token encoding, or implementation
was added. CW-37 debt progress is tracked separately by the builder receipt;
this review does not edit debt. No compiler, test, formatter, gate, or Git
operation was run. Repair the lock timing assertion, freeze a fresh source and
receipt, and request a new independent review before root RED.
