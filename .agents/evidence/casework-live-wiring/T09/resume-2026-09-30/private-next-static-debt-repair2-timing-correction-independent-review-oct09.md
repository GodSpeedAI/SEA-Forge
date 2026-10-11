# Private Next debt repair2 timing correction independent review

Date: 2026-10-09  
Verdict: **APPROVE this bounded CW-41 wording correction**

## Scope and evidence

I reviewed the fresh correction grant and builder record
`private-next-static-debt-repair2-timing-correction-builder-oct09.md` (SHA-256
`fd0df0a065f5c4e4f5eaf813ebfe8f53a964e81c911ebe3a4cc776a6fbfbeda1`), the
prior rejection `private-next-static-debt-repair2-fresh-correction-independent-review-oct09.md`,
and the exact test `TestRunObservationNextRechecksAuthorizationAfterWake`.
The current CW-41 follow-up is in `.agents/DEBT.md` (SHA-256
`989f048300718cb94ffc9cd1d4cc9654ef32aa5f0bd73042a52bfa25e96d6149`).

## Review result

The prior review's precise finding is corrected. CW-41 now says the separate
post-wake test checks `len(lease.wake)==0` only after `nextWithProjector`
returns with the failed second authorization check. The test source checks the
second authorization result before checking the empty wake, which supports
that wording and makes no claim about consumption timing within the call.
The wording also retains the distinct finding that the blocked first-auth
fixture had been missing an assertion that the seeded wake remained unconsumed
during the block.

The builder's quoted diff is limited to that phrase in CW-41; the surrounding
follow-up and prior entries remain intact. The builder records that
`.agents/OBSERVED_DEBT.md` remains restored at SHA-256
`be5c4554827d41bc5449f9c37d28434640aac5d459ec88dcc78bec4960f6c12a`, and the
correction did not add source, status, test, or gate claims. No material
deviation from the narrow documentation grant is evident.

This approval is limited to the one-phrase debt correction. It does not approve
the Next fixture source, compiler readiness, or any behavioral result.
