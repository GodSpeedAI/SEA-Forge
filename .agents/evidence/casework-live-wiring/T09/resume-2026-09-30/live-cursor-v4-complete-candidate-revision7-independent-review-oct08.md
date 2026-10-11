# Independent review — Live Cursor V4 candidate revision 7

**Verdict: APPROVE the complete DOCONLY design candidate.** This approves the candidate as a design proposal only. It grants no source, schema, runtime, migration-readiness, or implementation authorization. C-2 remains held pending authored specifications and schemas, complete writer-participation proof, and the required focused and canonical verification.

## Review scope and identities

I reviewed the full original candidate assignment, the full revision-5 repair assignment, revision 6 and its build result, the complete revision-6 independent rejection, revision 7 and its build result, the root legacy/u64 clarification, and the operator's six-recommendation approval record. I compared the complete revision-6 and revision-7 candidate texts. The exact candidate reviewed is `live-cursor-v4-complete-candidate-revision7-oct08.md`, 29,077 bytes, SHA-256 `ad82478fbe008358841b3be2ff0e29139f0327f1dd7cbbcbaf798a2795bd09b6`. Revision 6 is 25,674 bytes, SHA-256 `7e9009c5d733c8517531e705b9e5b2d6478bda5c9e4a551ce03e65a46e8b258d`. The revision-7 build result is 5,363 bytes, SHA-256 `4409e44e8d408f29b22b6e88281977ca5afcd3794891ae6914443bbb99597e2b`.

The review also used the full revision-5 assignment (SHA-256 `535ea0bd1ed5a7717edbddbcb7422b779ed9949fc01653ed18e434d072672187`), root clarification (SHA-256 `44e82f523b4063d4aafc88ea5fd716fc457632cf2b0bd1e0f545ad558099a8ef`), revision-6 independent rejection (SHA-256 `9cc1b3dcfcdc8d300deabe7c8549f87ef0703afb4b2fb65116a2258d35b2fc93`), and operator approval record (SHA-256 `7651f333fcf27b741db557d0dcbd4aba140e06b670edb49d70968f267be6be4b`). No source, tests, compiler, formatter, scanner, build, or Git operation was run.

## Findings

Both revision-6 blockers are resolved in the candidate itself:

1. **Legacy bounded reads fail closed unless complete.** Revision 7 §2, lines 21–22, preserves the old vector shape but makes successful return conditional on validating bounded coverage through the real pinned head. Row, byte, lock-wait, or time ceilings that prevent proof return typed unavailable with no vector and no partial frames. It preserves requested-prefix behavior only after full coverage, routes large histories to v2 paging, and rules out an unbounded fallback. This directly answers the sparse-history ambiguity in the revision-6 rejection. It is consistent with the root clarification: the legacy format has no continuation/completion field, so it cannot claim an incomplete scan is complete.

2. **The full-u64 ordinal contract is precise across languages.** Revision 7 §1, lines 13–14, specifies decimal-string wire values, canonical grammar, a 20-digit maximum and checked BigInt range through `18446744073709551615`; comparisons use validated BigInts only, with Number conversion, parseInt/parseFloat, and lexical ordering prohibited. It rejects malformed and overflowing forms and leaves opaque cursors and digests untouched. §1 and §5's verification list require round trips at zero, `Number.MAX_SAFE_INTEGER`, its successor and u64 maximum, plus overflow and malformed-input rejection. These rules match the root clarification and close the missing TypeScript precision contract identified in the v6 rejection.

The v6-to-v7 textual delta otherwise preserves the candidate's global-frontier correction and the prior bootstrap, IDs, shared journal, recovery, writer inventory, capture digest, authorization, history/SSE, Store, error, and cap decisions. The old case-local/global gap ambiguity remains corrected: v7 retains global continuity proof before event filtering, distinct global and case watermarks, and cell-wide unavailable/drain for unknown-impact global failures (§§2 and 5). Revision 5's lost-preimage disclosure is retained; no missing material is reconstructed.

## Approval boundary and material differences

Revision 7 changes its approval language to reflect the operator's recorded approval of the six recommendation-level areas. This is accurate: the approval record records “i approve all, following your recommendations.” The candidate does **not** claim that the operator read or approved the exact revision-7 hash, or each granular mapping. It keeps a specific-decision requirement for material expansion outside the six areas and retains independent design review, authored specs/schemas/ADRs, complete writer audit, TDD, and runtime gates as pending (§§1, 5). The operator's six-area approval is not being requested again.

The material changes from v6 are therefore the legacy completion rule, the explicit TypeScript/full-u64 behavior, the matching test matrix, and status wording recognizing the existing six-area approval while retaining exact-hash and implementation holds. They are all within the v5/v6 original assignment and the root clarification. I found no additional authority change, dependency, kernel verb, fabricated history, unbounded fallback, or other unapproved design expansion.

This is complete design approval only. The candidate's own claims remain proposals: the current reader still materializes entries before filtering; the bounded reader, journal, writer migration, digest, Store/SSE behavior, and clients are not proven implemented. The non-exhaustive writer table and explicit migration-coverage HOLD remain material. No runtime pass, exact-hash operator approval, or T09 completion is claimed.
