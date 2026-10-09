# Bounded ledger production repair Phase B receipt

Date: 2026-10-09

## Authority and reviewed RED

Phase B was released by root after accepting the focused RED02 run. The
controlling source grant is
`c2-bounded-reader-ledger-slice-root-grant-oct09.md` (SHA-256
`f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`), with
ACK-boundary addendum
`c2-bounded-reader-ledger-slice-ack-boundary-addendum-oct09.md` (SHA-256
`1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`). The
accepted Phase A fixture review is
`c2-bounded-reader-ledger-phase-a-regression-independent-review-oct09.md`
(SHA-256
`f5f9cf1c29055a0ded11e00d0d0734a2708cfcef9aba04b74575810cca57368f`). The
controlling production rejection is
`c2-bounded-reader-ledger-production-independent-source-review-followup-oct09.md`
(SHA-256
`0eea00a7edc4cdf77a4bef2b16261818892319a6c4a38b5357d51d46fdd90608`).

Root accepted `ledger-reader-regression-red02`: the focused command compiled
in 10.64 seconds, the test child exited 101, and the result was 0 passed, 2
expected behavioral failures, and 32 filtered. All six command, preflight,
preflight-exit, stdout, stderr, and exit captures are stored as
`ledger-reader-regression-red02-*-oct09.raw.json`; root reports byte-for-byte
comparison with its captures succeeded. Independent evidence review
`c2-bounded-reader-ledger-red02-independent-review-oct09.md` (SHA-256
`007e42e93f2b40252edf12f8031b0d4e8271ac4c00b0c83f599cd663834d4b1b`)
confirms both failures at their expected assertions. The run targets the pretty-JSON
resume and malformed-row post-error continuation findings. The phase A review
accepted the two-row fixture as the same stale-position coverage as the
requested 0/1/2 example. Neither RED finding covers allocation ordering.

## Frozen source and exact scope

The Phase A source before-image was decoded from
`c2-ledger-phase-a-frozen-source-oct09.raw.json` and matched byte-for-byte:
SHA-256 `84ee1cba001defd573f93a71f6465b77e7e41d5770395a3d6b580782b7e16ed3`.
The Phase B source is `crates/sea-forge-ledger/src/types.rs`, SHA-256
`0ff435746ca4323ed179609e14556a9739d3bd4cb92941df9f172a72b7f9f6fc`.
The test module from `#[cfg(test)]` to EOF is byte-identical to the Phase A
snapshot, retaining all 20 test cases and their assertions. The unified diff
against Phase A is 2,431 bytes, SHA-256
`0e834172be2bc7b09a16542c97ff81ab9ed6eb19c1de93005e7b9ca2e1e8b205`, with
labels `types.rs@phase-a-84ee1cba` and `types.rs@phase-b`, three context
lines, and LF line endings.

Only the three reviewed production defects were addressed:

1. `read_position_row` checked-adds the optional one-byte preceding-boundary
   probe to the claimed row length as `u64` and proves the total fits
   `remaining_raw_bytes` before seeking/reading the probe or reserving and
   resizing the row. The existing counted reads still charge actual bytes to
   the same request budget.
2. `validate_bounded_entry` rejects an LF before the required final LF, so a
   resume position cannot treat pretty-printed multi-line JSON as one physical
   JSONL row. Escaped backslash-n remains ordinary JSON content; no CR rule or
   other row grammar was added.
3. `LedgerReadSession::next_step` delegates to a private inner operation and
   marks the session failed for every returned `Err`. Later calls return an
   error before they can produce a row or completion. `LimitReached` remains
   its separate non-error result.

No test, helper hook, dependency, limit, schema, error variant, server/token/
wire/async boundary, or public re-export changed. No deviation from the
released Phase B scope is recorded.

## Verification boundary and disposition

The builder ran no compiler, tests, formatter, gate, or Git operation. Root
owns compilation and gates. This source is frozen for a fresh independent
source review; no GREEN result, implementation acceptance, or runtime
approval is claimed. The three findings remain pending that review and the
root-owned verification. CW-37's associated Phase B progress note is
`.agents/DEBT.md`, SHA-256
`d00fbb3dd41dee33e718e78d9e9351eccc1fc5fd0ef8acd1bbe90dea285975e4`.
