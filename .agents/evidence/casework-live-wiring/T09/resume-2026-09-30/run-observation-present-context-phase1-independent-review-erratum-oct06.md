# Present-context guard Phase 1 review — additional fixture findings

Date: 2026-10-06  
This immutable addendum supplements `run-observation-present-context-phase1-independent-review-oct06.md` (SHA-256 `d715a0b5ff36171e0403a2692fff62e0c5bdc42781e28ca1eb4bb7f377f91c16`). It does not replace that record or alter the fixture verdict.

## Additional independently verified findings

1. **The positive immutability comparison fails for a correct implementation because the clone changes nonnil-empty shape.** The fixture helper creates `Snapshot.VisibleObjects: []contract.CognitiveObject{}` (`run_observation_present_context_test.go:49-52`). Its clone helper uses `append([]contract.CognitiveObject(nil), ...)` (`:76`), which turns that nonnil empty slice into nil. The positive test compares `inputs` with `before` after a successful check (`:208-219`); `reflect.DeepEqual` sees empty versus nil even if the checker did not mutate anything. Preserve the nonnil-empty representation when copying, or adjust the fixture/comparison so its pre-call baseline has identical shape. Do not interpret this as a validation failure.

2. **The blank-revision-cursor row also conflates blankness with mismatch.** It sets only `Revision.Cursor = ""`, while snapshot cursor, facts cursor, and relay cursor remain nonempty (`:135,143-147`). The result must fail because those views disagree even if the checker has no explicit blank-cursor rule. For an isolated blank-cursor case, make the compared cursor views blank together; retain a separate nonblank test changing only `Revision.Cursor` while snapshot/facts/relay keep a matching opaque cursor. This independently tests both required properties.

3. **The no-fallback assertion needs a relay value that would permit the bad fallback.** Current `older` is `cursor/old`, `newest` is invalid at `cursor/new`, and relay is also `cursor/new` (`:156-165`). An incorrect fallback to `older` is then rejected by the relay comparison and the test passes. Set the relay cursor to `cursor/old`; a fallback implementation would return the older valid row, while the required newest-only check still rejects the invalid newest row.

4. **The identity-baseline alias and populated Overview slice findings stand.** The rejected-identity tests pass `before` directly as the fake history slice and compare that same slice to itself (`:145-150`); the latest-row test likewise compares the fake's input slice to itself (`:160-165`). The clone helper shallow-copies populated `Facts.Overview.Stages` (`:41,78-86`). These assertions do not establish full-input immutability. Pass a separate source slice to the fake and compare it with a genuinely deep pre-call baseline that copies every populated mutable field.

5. **Do not require a relay read after rejecting a mismatched latest row.** The wrong-case test requires both fake dependencies to have been called with `case_1` even though `Trajectory("case_1")` returns a row for `case_other` (`:276-283`). A correct guard can reject the row immediately and never query Relay. The original assignment requires the exact requested case when querying Relay, but does not require that query after history already fails. Keep the exact-case assertion on a valid-history relay-gap path (`:181-188`) and avoid constraining this unnecessary error-path call order.

## Static stub boundary remains unchanged

The source SHA-256 remains `565ad7a604bf68ab7667c901eaa23928d28476150fdfa82da9e170ba9e61ee86`; the fixture SHA-256 remains `825ab0dcdb80b27338401aab8227474c4422e1a4aea0b3d22073b0ae05255060`. The implementation is still the required always-unavailable Phase 1 stub. Under that stub, positive success tests stop at their first `err != nil` check; copy/freshness assertions after that point are not reached. Negative typed-error/zero-result checks can pass without executing any input-specific validation. Relay-call expectations fail under the stub because the dependencies are not called. No compiler or test was run and no actual RED result is claimed.

The combined independent verdict remains **REJECT the fixture pending these bounded repairs and fresh review**. Source scope remains approved only as the intentional stub; no algorithm, caller, or runtime approval is inferred.
