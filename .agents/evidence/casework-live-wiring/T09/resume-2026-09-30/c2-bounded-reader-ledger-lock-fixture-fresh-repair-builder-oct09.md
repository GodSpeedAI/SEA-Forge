# Bounded ledger reader lock-fixture fresh repair: builder freeze

Date: 2026-10-09. This is a source-only fixture repair for independent review.
It makes no reader implementation, test execution, runtime, or approval claim.

## Authority and source identity

- Full source grant:
  `c2-bounded-reader-ledger-slice-root-grant-oct09.md`, SHA-256
  `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`.
- ACK-boundary addendum:
  `c2-bounded-reader-ledger-slice-ack-boundary-addendum-oct09.md`, SHA-256
  `1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`.
- Prior fixture repair receipt:
  `c2-bounded-reader-ledger-tdd-fixture-repair-builder-oct09.md`, SHA-256
  `3baddf1ee571964f47204dd7ad137a1fe0cb84d27d6b311e823164779456bf3d`.
- Rejection review:
  `c2-bounded-reader-ledger-tdd-fixture-repair-independent-review-oct09.md`,
  SHA-256
  `72511d706b21e7869c5235eb3bfeb63fe1155502dedbb28aeffc98eac9e9bd3c`.
- The immutable repair-1 source archive
  `c2-ledger-tdd-repair1-frozen-source-oct09.raw.json` was decoded and
  compared byte-for-byte with `crates/sea-forge-ledger/src/types.rs` before
  this edit: both were 105,930 bytes with SHA-256
  `587619ce307c1ee8ed996e0cad0106691c1bb798ec75c880c68d4c848a7f8619`.
- Fresh repaired `types.rs` SHA-256:
  `6a22fbd14cafb5312cf81f2b2b694cb2413fbe21afaf35dcc4263a27d25e0c33`.

## Exact change and limits

The only code change is in
`bounded_reader_fails_closed_when_cooperative_lock_is_held`: its upper elapsed
bound changes from `Duration::from_secs(1)` to `Duration::from_millis(150)`.
The 40 ms lower bound, ready-before-worker ordering, bounded receive, and
release plus JOIN before assertions remain exact. No helper, implementation,
or other code/assertion/declaration/TODO stub changed.

The 150 ms ceiling is scheduling tolerance around the granted 50 ms deadline;
it does not guarantee or establish an OS scheduling-latency bound or prove
exact 50 ms monotonic runtime behavior. After root's accepted RED, an
independent source review must inspect the implementation's actual monotonic
50 ms deadline directly. RED and review are pending. No compiler, test,
formatter, gate, Git operation, or runtime result is claimed.
