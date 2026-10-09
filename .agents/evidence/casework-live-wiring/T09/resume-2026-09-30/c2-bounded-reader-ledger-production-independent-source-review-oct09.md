# Bounded ledger reader production: independent source review

Date: 2026-10-09

## Verdict

**SOURCE_READY_PENDING_RUNTIME.** The implementation is within the reviewed
ledger-only grant and no blocking source defect was found in this read. This
is not final implementation approval: root must run the authorized focused
and crate gates, then provide the actual evidence for an independent full
review. No compile, tests, gates, or runtime commands were run for this review.

## Reviewed identities and test preservation

- Root grant: `c2-bounded-reader-ledger-slice-root-grant-oct09.md`, SHA-256
  `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`.
- ACK-boundary addendum:
  `c2-bounded-reader-ledger-slice-ack-boundary-addendum-oct09.md`, SHA-256
  `1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`.
- Production receipt:
  `c2-bounded-reader-ledger-production-builder-oct09.md`, SHA-256
  `103cf4777a4ea2c08c716f8d131ceaa0195f781fc294f4f9f973102fda0a7bce`.
- Frozen source `crates/sea-forge-ledger/src/types.rs`, SHA-256
  `3045c6d3b089e119ac2ddf9cb186249d726fa3e543da1ed8766223584b8e4425`.
- The accepted RED baseline archive decodes to 105,934 bytes with SHA-256
  `6a22fbd14cafb5312cf81f2b2b694cb2413fbe21afaf35dcc4263a27d25e0c33`.
  Direct test-module comparison confirms every test byte and assertion is
  unchanged after removing only the root-authorized unused
  `READER_PAGE_BYTES` declaration. All 18 tests remain.

## Source review

The interface is confined to `types.rs`: positions and resume inputs have no
Serde serialization derives; the session is created from `&LedgerStream`,
owns its file and fixed limits, and returns boxed row, complete, or explicit
limit-reached outcomes. Positions identify start/end offsets, ordinal, entry
ULID/hash, and SHA-256 of the exact LF-terminated raw row. Existing typed
payload and entry hash functions remain authoritative.

The fresh path opens only the registered entries file, checks regular-file
status while allowing symlinks to regular targets, samples and validates its
tail under the same cooperative file lock used by the existing writer, then
releases the lock before scanning. Absent and zero-byte files can complete
empty only after that under-lock observation. Resume checks the supplied
original pin and optional acknowledged row against actual bytes, typed hashes,
boundaries, and raw checksum under the lock before seeking; it retains the
original pin and ignores later appends.

The lock loop uses `try_lock`, a monotonic `Instant` deadline 50 ms after
attempt timing begins, and checks the deadline before each retry and after a
successful acquisition. The implementation therefore matches the granted
deadline in source; runtime scheduling latency remains for the lock test and
runtime evidence to characterize.

Auxiliary pin and resume reads and forward reads mutate the same private 4 MiB
remaining-byte counter. Each physical read request is capped by that remaining
budget and the fixed 8 KiB buffer, and successful bytes are deducted from the
counter. The reverse tail probe charges complete blocks, including bytes
before the row boundary; resume boundary probes and rereads are also charged.
The tail is reread physically during forward scanning rather than replayed
from an uncharged cache. Forward read-ahead bytes are charged when physically
read, including bytes after the current LF; retained buffer bytes are not
charged again.

Row accumulation is bounded before reserve/extend, includes LF in the 2 MiB
limit, and rejects torn or oversized rows. Budget exhaustion returns a
distinct limit outcome without advancing the validated checkpoint for an
unfinished row. Ordinals and previous-entry linkage are checked during the
forward scan; the immutable pinned position must match exactly when the scan
reaches its end. The exact 500th row can complete at the pin before the next
call tests the row limit; otherwise the 501st row is not returned. No
unbounded ledger reader, verifier, line API, MMR/root proof, cursor grammar,
identity cache, mutable test seam, caller budget, or alternate checksum
protocol was introduced. Arithmetic helpers are used in production offset
and ordinal calculations.

The ACK addendum is respected: these positions remain ledger-integrity
candidates only. Event-shape validation of every event row, including rows
later filtered out, remains a server/caller prerequisite before ACK/frontier
advancement or frame emission. No event validity or acknowledgement claim is
made by this crate slice.

No dependency, persisted schema, ForgeError variant, server route, public wire
DTO, token encoding, async primitive, generated file, or unrelated source
change was identified in the reviewed implementation scope. The complete
implementation is limited to the authorized ledger-reader slice and two
approved arithmetic helpers.

## Limits before final approval

Static source review does not establish compilation, test success, exact
resource use under the host guard, or runtime lock timing. Root owns the
compiler and required gates. After those results are available, an independent
full review must inspect their actual captures and re-review the frozen final
source before implementation approval. The returned checkpoint remains
ledger-integrity validated only; the separately held server boundary and
broader C2/T09 work remain outside this slice.
