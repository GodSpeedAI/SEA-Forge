# T08 stale-refetch case-switch repair

## Original bounded builder instructions

Fix the independently observed race in `src/app/intents.ts`: a typed stale refusal awaits
`getSnapshot(originalCaseId)` and then merges into the currently selected history without
checking its case identity. Preserve the typed refusal, one refetch of the original case,
no intent replay, and honest refresh failure. A response must never merge the original case
snapshot into a newly selected case. Validate the returned snapshot case identity. Add a
deferred-promise case-switch regression and a wrong-case-response regression. Own only
`src/app/intents.ts` and `src/app/intents.test.ts`; no dependency, contract or identity changes.
Do not compile or run tests while the exclusive compile token belongs to T07's critic.

## Builder result

Fresh Luna builder `/root/stale_race_builder_recovery` implemented an active-history case
guard after refetch, a returned `fresh.case_id` mismatch check through the existing refresh
failure note, and both requested regression tests. The builder reported `git diff --check`
passing. The builder did not run tests, typecheck or build. Independent T08 re-review remains
pending; this note is a reported builder result, not approval or settlement.

The prior daemon interrupted an earlier fresh builder before its race edits; the recovered
builder checked existing source before implementing. Historical rejection evidence is retained.
