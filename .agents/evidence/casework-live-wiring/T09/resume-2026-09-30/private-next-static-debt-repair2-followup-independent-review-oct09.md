# Independent review: repair-2 debt follow-up

## Verdict

**REJECT the follow-up record as scoped and attributed.** The evidence itself
largely describes the repair-1 review and its limits, but the follow-up wrote
to `.agents/OBSERVED_DEBT.md` instead of the explicitly authorized
`.agents/DEBT.md` CW-41 entry. Its builder record incorrectly attributes that
path to root, and one evidence phrase misstates the review finding.

## Evidence inspected

Read the original CW-41 builder grant and record, the prior independent source
review `b5693180`, the fresh-repair source review, the full current `.agents/DEBT.md`,
the relevant `.agents/OBSERVED_DEBT.md` and its appended item, and the
repair-2 follow-up builder record. Read the exact read-only diff for
`.agents/DEBT.md` and `.agents/OBSERVED_DEBT.md`.

The builder-record hash matches the current record:
`7c4e65a99f52e4110c00f5b164576006b13d43c5b0a2b00f1d26d5cd7425cf16`.
The edited file hash matches its recorded identity:
`90aeb3c78f58ad298b911a766408ef3538f659107f51cc72fee6cf33c4701e57` for
`.agents/OBSERVED_DEBT.md`. `.agents/DEBT.md` remains at the previously
reviewed CW-41 identity `be651245fd6a593b239d73775f8e2fa954e2ab04995b7ee73935cd7270a6324b`.

## Blocking findings

1. **The edited ledger path exceeded the grant.** The original CW-41 builder
   grant, reproduced in `private-next-static-debt-builder-oct09.md`, directed
   an append to `.agents/DEBT.md`, preserving CW-40. The standing operator
   instruction in the current status also requires encountered debt in
   `.agents/DEBT.md` despite the general `.agents/AGENTS.md` routing to
   `OBSERVED_DEBT.md`. The read-only diff shows the follow-up instead appended
   a new “private Next fixture source provenance” item to
   `.agents/OBSERVED_DEBT.md`; it made no repair-2 follow-up to CW-41 in
   `.agents/DEBT.md`. This is a material scope and ledger-authority deviation.
   The record's claim that root's instruction called for reading and appending
   to `OBSERVED_DEBT.md` conflicts with the original grant and standing
   operator instruction. A fresh builder should put the bounded factual
   update at CW-41 in `.agents/DEBT.md` under the correct instruction.

2. **The wake finding is misdescribed.** The appended item says “a seeded-wake
   oracle [was] placed after the wait instead of before it.” Review `b5693180`
   did not find an oracle placed at the wrong point. It found that the blocked
   authorization callback test had no assertion that the seeded wake remained
   unconsumed while the callback was blocked. The separate post-wake test
   asserted that the wake had been consumed after the failed second check.
   The new wording turns a missing assertion into an observed ordering defect.
   Replace it with the precise missing-oracle description; retain the actual
   post-wake evidence as distinct.

3. **The preimage caveat is otherwise accurate.** The original Next test file
   was untracked, and its `c8e825...` bytes were not retained. The repair-1
   `72c379...` snapshot is not that original. The record appropriately avoids
   claiming exact historical assertion preservation, and reports no compiler,
   test, or behavioral RED evidence. The root-reported gofmt probe is
   consistently described as read-only; its format issue belongs to the
   source repair, not this debt update.

## Required next step and limits

Have a fresh documentation builder make only the authorized CW-41 update in
`.agents/DEBT.md`, correct the wake-oracle wording, preserve all existing debt
entries, and record the final file identity and exact diff. Root can separately
decide how to handle the unauthorized `OBSERVED_DEBT.md` addition; this critic
does not remove or rewrite it. No source, status, Git mutation, compiler,
test, or gate was performed for this review.
