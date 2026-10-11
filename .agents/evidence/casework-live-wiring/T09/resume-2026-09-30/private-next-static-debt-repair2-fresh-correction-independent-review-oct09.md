# Independent review: repair-2 debt follow-up correction

## Verdict

**REJECT pending one evidence wording correction.** The fresh builder removed
the unauthorized `OBSERVED_DEBT.md` paragraph, moved the follow-up into CW-41,
disclosed the predecessor scope error, and corrected the original blocked-auth
finding. However, the new CW-41 text overstates what the post-wake fixture
asserts about event ordering.

## Evidence and identities

Read the full fresh correction builder record and its reproduced original and
fresh grants, the prior builder/review records, the current full debt files,
and the exact read-only diff for `.agents/DEBT.md` and
`.agents/OBSERVED_DEBT.md`. The artifact identities match the builder record:

| Artifact | SHA-256 |
|---|---|
| `.agents/DEBT.md` | `b8b3b048f446d70ad849686f190440718d047a6c1bbd2f703b1f5ea2b636aee8` |
| `.agents/OBSERVED_DEBT.md` | `be5c4554827d41bc5449f9c37d28434640aac5d459ec88dcc78bec4960f6c12a` |
| Fresh correction builder record | `7f598157012e8e9c649699d9579c97ad1891ef201ac3744fc12bff9fbb8ee8a1` |
| Prior rejection review | `4a960bea606ec65ffc57dfc3ebd8b26bf2bddd3246398b6921dbc03a6ca923a9` |

The read-only diff shows the added private Next paragraph is gone from
`OBSERVED_DEBT.md`; that file has no remaining diff. CW-41 in `.agents/DEBT.md`
contains the bounded source-review follow-up. The previously added CW-40 and
CW-41 remain in the append-only diff; the new follow-up is the sole additional
paragraph. The original `c8e825...` preimage versus current repair-1
`72c379...` distinction and the absence of compiler, test, and RED evidence
are recorded accurately. The prior scope attribution is disclosed, not
silently overwritten.

## Blocking finding

The new CW-41 paragraph says the separate post-wake test “did assert
consumption before its failed second authorization check.” In
`TestRunObservationNextRechecksAuthorizationAfterWake`, the test checks
`len(lease.wake) == 0` only after `nextWithProjector` has returned with the
second authorization failure. That assertion establishes the wake is empty
after the call; it does not independently establish that consumption preceded
the callback failure. The new paragraph correctly says the blocked first
authorization test lacked the assertion that the seeded wake remained
unconsumed during the block. Keep that statement, but describe the separate
post-wake assertion as an after-return observation, without claiming an
ordering it does not prove.

## Limits and next step

This review is documentation-only. No source, status, or debt file was edited;
no Git mutation, compiler, test, formatter, or gate was run. Have a fresh
documentation builder correct only the post-wake timing description, preserve
the restored `OBSERVED_DEBT.md` and all earlier debt entries, then request
independent review of the corrected artifact. The Next source review is a
separate gate and receives no approval from this documentation review.
