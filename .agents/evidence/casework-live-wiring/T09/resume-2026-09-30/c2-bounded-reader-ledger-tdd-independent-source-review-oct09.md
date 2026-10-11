# C2 bounded ledger reader TDD scaffold: independent source review

Date: 2026-10-09

## Verdict

**REJECT this frozen test scaffold for root RED; repair the focused vectors
first.** The API declarations/stubs are within the approved first-stage
boundary, but the test suite omits required integrity/resume vectors and has
an unbounded lock test plus a cap-plus-one assertion that overconstrains a
valid implementation. No compiler, tests, gates, or source edits were made
for this review.

## Reviewed identities and scope

- Frozen source: `crates/sea-forge-ledger/src/types.rs`, SHA-256
  `40ac4fa2f04bef5f5b43940a134a2d99189787daf1f0375405f3fbac617ee2f8`.
- Builder receipt: `c2-bounded-ledger-reader-ledger-tdd-builder-oct09.md`,
  SHA-256 `cb61ff605e62a465dbdc1132a479414ce00655adaaa7b82741fd08a1a22ddf98`.
- Full authority: bounded-reader source grant, ACK-boundary addendum, and its
  accepted independent re-review; approved C2 supplement and parent; revision
  2 proposal, revision 3/4 overlays and reviews; operator policy receipt; and
  Oct. 9 registration/input-accounting decision/reviews. The grant requires
  the listed focused vectors before root accepts RED or releases source
  implementation.
- Source diff is confined to `types.rs`: two approved private checked-arithmetic
  helper stubs, the approved bounded-reader API/session declarations, and 17
  unit tests. No dependency, wire serialization, server/API path, schema, or
  unrelated production behavior was added. API ownership and the
  ledger-integrity-only candidate checkpoint correctly reflect the accepted
  grant/addendum. The private remaining-byte observation is only used by
  same-module tests, as root explicitly allowed.

## Blocking findings

### 1. Required protocol-constant and entry-hash rejection vectors are missing

The grant requires origin/hash/linkage/constants and typed payload/entry hash
validation (`c2-bounded-reader-ledger-slice-root-grant-oct09.md:82-85,97-103`).
The staged corruption test changes a later ordinal and predecessor link, then
tests a bad `payload_hash` while recomputing that row's `entry_hash`
(`types.rs:2245-2270`). It never corrupts a stored row's `entry_hash`, nor the
registered `ledger_id`, `version`, `canonicalization`, or `hash_algorithm`.
Normal writer-created rows exercise the positive constants path but cannot
prove rejection of mismatches. Add actual persisted-row negative cases for
each required constant and for an incorrect entry hash with otherwise valid
payload hash/linkage. Also make the origin invariant explicit: ordinal zero
with no predecessor is accepted, while a nonzero first ordinal or a
predecessor at origin is rejected. Current valid fixtures begin at ordinal
zero, but no test asserts rejection of invalid origin state.

### 2. Resume identity validation is only partially covered

The grant requires resume to validate the original pin and acknowledged
predecessor cursor, ordinal, hash, checksum, and byte boundaries before seek
(`...root-grant-oct09.md:48-57,67-74,101-103`). The tests mutate pin entry hash,
pin checksum, pin offsets, acknowledged checksum, and acknowledged end offset
(`types.rs:2184-2234`), but not acknowledged `entry_ulid`, ordinal, entry hash,
or start offset. The on-disk predecessor mutation flips the first raw byte
(`types.rs:2161-2181`), making JSON malformed; it therefore cannot show that a
valid typed predecessor with altered raw bytes is rejected by raw-checksum
validation. There is no test that mutates the pinned-head bytes after session
creation and then resumes against the original pin. Add field-specific resume
vectors for these identities and a byte-preserving typed-row mutation vector
that isolates checksum validation; cover a supported append after the pin
across resume as well as fresh scan if the same immutable-pin guarantee is
claimed for resumed sessions.

### 3. The cooperative-lock test can hang and does not prove bounded timeout

`bounded_reader_fails_closed_when_cooperative_lock_is_held` calls the reader
synchronously while a helper thread holds the lock, then releases it only
after the call returns (`types.rs:2034-2057`). An incorrect blocking lock
implementation will hang this test indefinitely, rather than fail the
50 ms-bound assertion. Run the reader on a worker, wait with a finite channel
timeout, release the held lock on every path, join the worker, and assert the
reader returns a lock-timeout error within a tolerant bound. This makes the
fixture deterministic and actually guards the grant's monotonic 50 ms limit.

### 4. The cap-plus-one test overconstrains budget consumption

The aggregate vector correctly arranges a 4 MiB + 1 minimum read demand and
checks the prior validated position is unchanged (`types.rs:2087-2104`). But
the final `remaining_raw_bytes == 0` assertion requires the reader to consume
all remaining budget while attempting the partial next row. The grant requires
charging before reads, no request beyond the remaining budget, and no partial
ACK; it does not require draining the budget. A reader may use the pinned
boundary/metadata to determine that the row cannot fit and return `LimitReached`
with positive budget remaining. Assert the prior checkpoint and bounded
outcome instead; prove exact accounting with the exact-cap vector and/or a
read-count observation that does not require exhausting unused budget.

### 5. Exact 500-row completion boundary is absent

The 501-row test proves the 500th row is followed by `LimitReached`
(`types.rs:2106-2120`), but does not prove that a history whose actual pinned
head is exactly row 500 returns `Complete` rather than false limit exhaustion.
Add the 500-row exact-head counterpart. The distinction is central to the
grant's row cap and explicit non-EOF limit outcome.

## Other coverage and proof limits

The unknown-field fixture demonstrates an accepted unknown top-level field
and a payload extension whose helper hashes are recomputed (`types.rs:1978-2001`);
it does not by itself isolate rejection when the payload extension changes
without matching hashes. Add a negative hash-binding assertion if the intended
claim is that unknown payload extensions are actually covered by integrity
validation. The directory and missing-parent cases cover nonregular input and
path I/O failure (`types.rs:2024-2032`); they do not demonstrate permission-
denied regular-file behavior, which can be ineffective under a privileged
test user. Consider a deterministic I/O-failure fixture if needed, without
adding a fake reader hook.

All reader entry points and both arithmetic helpers remain explicit `todo!`
stubs (`types.rs:27-35,477-485,581-593`), as allowed at this stage. Since
tests reach those stubs before exercising their row assertions, root's RED
must be reported only as the expected unimplemented-stub failure, not as proof
that these acceptance vectors execute or pass. No compile claim is made here;
root must separately verify compilation and diagnose the RED. There is no
static compile error apparent from the reviewed declarations, but this review
cannot replace the authorized root compiler run.

The event-shape requirement is correctly excluded from this ledger slice:
the candidate position is not ACK eligibility, and server/caller validation
remains governed by `REQ-C2-RANGE-003` and the approved ACK-boundary addendum.
No event-shape test belongs in this ledger-only TDD stage.

Graft was queried first for the bounded-reader and resume source surface; it
saved approximately 8,690 tokens in one retrieval call. Existing `.jolli` and
`.gemini` worktree changes were left untouched.
