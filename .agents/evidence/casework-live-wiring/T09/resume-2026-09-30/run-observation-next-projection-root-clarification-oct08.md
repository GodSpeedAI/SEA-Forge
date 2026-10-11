# Next projection: exact gap and malformed-input test oracles

Date: 2026-10-08. Additive to the full projection assignment; source release
still held until root explicitly releases after checkpoint. No compiler grant.

For every reported observed-ID gap, UnobservedLossPossible=true and
UnknownMissedFrameCount=true. A nontruncated current window does not prove that
unseen/malformed source rows never existed. Ordinals/totals are not source-loss
counts. A gap count counts only the exact reported observed identities.

Malformed captured inputs fail closed with an error and zero outputs: nil
state, impossible total/retained counts, P>H, or copied ledger/frame first
ordinals that disagree. Include explicit tests. Copied ledger identities above
captured H are intentionally allowed and ignored for this delta, not malformed.
Do not consult current state or a different ledger to repair bad inputs.

The independent reviewer receives this FULL clarification, original assignment
and resulting actual source/tests. No public or lifecycle integration release.
