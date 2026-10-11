# Bounded ledger reader Phase B: independent source review

Date: 2026-10-09

## Verdict

**SOURCE-READY; runtime GREEN and production approval remain pending root-owned verification.** I found no material deviation or remaining defect within the granted synchronous reader slice.

## Frozen inputs and scope

- Reviewed `crates/sea-forge-ledger/src/types.rs`, SHA-256 `0ff435746ca4323ed179609e14556a9739d3bd4cb92941df9f172a72b7f9f6fc`.
- Builder receipt `c2-bounded-reader-ledger-phase-b-production-builder-oct09.md`, SHA-256 `c40838507c20d8658819bfd5e924b1c8697758ab20723ac69763cc8bfa1b726c`.
- Controlling grant SHA-256 `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`; ACK-boundary addendum SHA-256 `1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`.
- Prior rejection SHA-256 `0eea00a7edc4cdf77a4bef2b16261818892319a6c4a38b5357d51d46fdd90608`; accepted RED02 record SHA-256 `007e42e93f2b40252edf12f8031b0d4e8271ac4c00b0c83f599cd663834d4b1b`.
- Source delta from Phase A is exactly 2,431 bytes, SHA-256 `0e834172be2bc7b09a16542c97ff81ab9ed6eb19c1de93005e7b9ca2e1e8b205`, reproduced with `diff -U3 --label types.rs@phase-a-84ee1cba --label types.rs@phase-b` against the archived Phase A bytes. The `#[cfg(test)]` suffix is byte-identical to that Phase A snapshot (37,743 bytes; SHA-256 `55c3680c3df31f4b82eadcf473b0b124d7800c5deb1d4fecbfcde6f9b72483cc`).

## Findings

1. **Remaining-budget allocation bound fixed.** In `read_position_row` (around lines 553–590), the claimed range is first checked for positive length, the 2 MiB row cap, and file bounds. A checked `u64` addition includes the optional preceding-boundary byte; the complete requirement is compared with `remaining_raw_bytes` before seeking, reading, converting for allocation, reserving, or resizing. Actual probe/row reads still use `read_exact_counted`, which charges each byte against the same request budget. Thus a row that cannot fit fails before row allocation or probe consumption.
2. **Resume row physical boundary fixed.** `validate_bounded_entry` requires a final LF and rejects any earlier literal LF in the row bytes before JSON parsing. Escaped `\\n` remains JSON content. No new CR restriction or other row grammar was added. This applies to both resumed positions and forward rows.
3. **All returned errors poison the session.** `next_step` checks `failed`, delegates to `next_step_inner`, then sets `failed` for every `Err` result. Both fresh and resumed session constructors initialize `failed: false`. All parsing, I/O, budget, linkage, pin, and boundary errors from the inner operation return through that wrapper; later calls return an error before they can yield `Row` or `Complete`. `LimitReached` remains a non-error result.

The inspected source delta contains only these production repairs, the session failure field/wrapper, and its initialization in both constructors. Existing tests/assertions were not modified after the accepted Phase A fixture freeze. No dependency, schema, error variant, public DTO, server/token/async boundary, or re-export was changed. The ACK addendum's distinction is respected: returned positions attest ledger integrity only; event payload eligibility remains caller/server validation before acknowledgement.

## Verification boundary

This was a static source review. I ran no compiler, tests, formatter, gates, or Git operations. The reported RED02 is prior root-owned evidence, not GREEN evidence. Root owns focused/full crate verification and all required gates; implementation/runtime acceptance remains open until those results are reviewed.
