# Local cursor ordering Phase 1 — independent RED2 evidence verdict

Date: 2026-10-06  
Verdict: **Accept as bounded negative evidence against the frozen pre-fix adapter, based on the archived run and contemporaneous prior review.** I did not rerun tests. This releases no implementation and makes no full UI/T09 claim.

## Evidence inspected

I read the original assignment, fixture-repair assignment, fresh fixture source review, and the prior `local-cursor-order-phase1-actual-red2-independent-review-oct06.md`. I independently hashed the current candidate test, frozen production adapter, and conformance file; they match the review's identities:

- Test: `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1`.
- Production adapter: `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34`.
- Conformance: `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.

The three archived RED2 capture files independently hash to the values recorded in that contemporaneous review: preflight `d02a284ba2fc69ae9a90e03b7c691feac7752beae67d4d6efb19a0754a608872`, Bun output `19781fc438013e4ec32af891c7b228408a653093e79c02ce470173084671cc2e`, and exit `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22`. The original `/tmp/sea-casework-20261006-ui-cursor-order-red02-{preflight,bun-output,exit}.raw` files are absent in this review environment. Therefore I could not repeat the historical byte comparisons; the earlier reviewer records `cmp` exit 0 for all three, and that remains prior recorded evidence rather than a comparison I performed.

## Semantic assessment

The archived run is recorded as **2 passed, 4 failed, 24 expectations, six tests, 340 ms, exit 1**. The failures are semantic assertions, with no setup/timer/timeout failure reported:

1. Future floor equality was delivered rather than suppressed.
2. Settlement side-event cursor resolved to a retained snapshot.
3. Progress cursor resolved to a retained snapshot.
4. An already queued callback fired after unsubscribe.

These are source-consistent against the frozen adapter. No claim is made that I reran or freshly observed this result. The supplied-H and omitted-boundary chronology checks passed but do not independently prove floor filtering.

Later assertions were not reached: post-floor H+2 delivery, public ordinary-event cursor uniqueness/order and settlement-after-snapshot checks, and the second-case allocator assertions. Parser/invalid-cursor behavior, FIFO/error semantics and capacity, sequence ceiling/exhaustion, constructor validation, broader/full UI coverage, and integration remain unproved. This verdict accepts only the four recorded negative findings; no positive implementation claim follows.

## Guard evidence cross-check

I also read the final guard runtime review and root final acceptance. The current guard and fixture source hashes are recorded as `19bd9a574a25beab1e76824ddf136a3091e418d77e7573068c8d3e9464a3b1c2` and `46caa4f5869d705c1ff2678181890e26143d7903f2b697b533b7a11899e51eea`. The evidence directory contains exactly twelve final capture files for four sequential gates (preflight, output, exit each); their archived hashes match the final runtime review's listed hashes. Root's acceptance records that it previously byte-compared all twelve archive/original pairs successfully and accepted the private guard only. The corresponding `/tmp` originals are unavailable here, so I did not independently repeat those comparisons. This is not my runtime verification and does not extend the guard acceptance to manager/public integration, CI, publication, or T09 settlement.

No compiler, test, scanner, or Git command was run. No production or test source was changed; this file records an independent evidence assessment only.
