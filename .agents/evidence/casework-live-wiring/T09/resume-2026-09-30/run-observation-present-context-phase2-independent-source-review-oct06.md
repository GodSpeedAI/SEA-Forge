# Present-context guard Phase 2 — independent source review

Date: 2026-10-06  
Verdict: **APPROVE implementation source for root consideration of separately assigned runtime gates.** This is static source review only. No compiler, test, scanner, Graft build, Git, or network command was run. It does not authorize a caller, manager integration, production readiness, or continuous freshness claim.

## Frozen evidence

- Original eight-requirement assignment `run-observation-present-context-original-assignment-oct06.md`: SHA-256 `f15838d11468f6bade76a7a759ebc9676c7d1c9e5d260c64440d0de3df9e262d`.
- Root's focused RED acceptance `run-observation-present-context-phase1-root-red-acceptance-oct06.md`: independently read; it accepts only the focused semantic RED against the stub and releases this private source implementation, with source/runtime review and further serialized gates still required.
- Implementation `apps/godspeed-casework-go/internal/server/run_observation_present_context.go`: SHA-256 `19bd9a574a25beab1e76824ddf136a3091e418d77e7573068c8d3e9464a3b1c2`.
- Frozen fixture `apps/godspeed-casework-go/internal/server/run_observation_present_context_test.go`: SHA-256 `fffa033505fa19b937da12963df91a0cdc2af30f39ecb50031fc2a17ec64a67b`.
- Phase 1 focused RED run was session 13235, Go exit 1 against the intentionally unavailable stub. Root's acceptance says its direct original-copy capture is byte-identical to `/tmp`; this review makes no new runtime claim.

## Requirement-by-requirement source findings

1. **Blank request, nil dependencies, and unavailable history.** The entry guard rejects whitespace-only case IDs and both dependencies before calling either (`run_observation_present_context.go:38-40`). It rejects a nil or empty returned history by `len(revisions) == 0` (`:42-45`).

2. **Only newest retained row, exact case, no fallback or lexical cursor ordering.** It selects only `revisions[len(revisions)-1]` (`:42-49`) and requires its `CaseID` to exactly equal the requested `caseID`. There is no sorting, cursor comparison ordering, older-row fallback, kernel/hydration dependency, or third dependency in the two private seams (`:11-17`). The newest-row error path returns before relay lookup if its horizon or identity is invalid; this is consistent with the repaired fixture, which does not require unnecessary Relay reads on invalid history.

3. **All independent identity/cursor views.** The newest revision must have nonblank cursor and exact case; snapshot case must match, snapshot cursor and facts cursor must exactly equal the revision cursor, Facts must be nonnil, and Record/Overview/Horizon refs must exactly equal the requested case (`:47-53`). Cursor whitespace is used only to identify blankness; cursor values otherwise remain opaque and equality is exact.

4. **Horizon and parent ID validation.** Nil `Horizon.Items` is rejected while a nonnil empty slice is accepted (`:52-53,56-65`). Each ID is rejected if whitespace-only; duplicate detection and map keys use the full original ID without trimming, normalization, or suffix truncation (`:56-65`).

5. **Observed relay cursor.** Relay is queried with the exact requested case only after candidate facts validate (`:67`). Missing observation, whitespace-only cursor, or any exact-value mismatch with the newest revision cursor returns unavailable (`:67-70`). There is no lexical or prefix comparison.

6. **Success ownership and source nonmutation.** The result uses the requested case, exact newest cursor, and a newly allocated map populated from source strings (`:56-65,71-75`). The function does not assign through or mutate any input pointer, map, or slice. A fresh map is created for every invocation. The frozen populated/empty success tests compare complete input history and relay maps against separate pre-call baselines, mutate the returned map and source items in turn, and repeat the check (`run_observation_present_context_test.go:354-406`). These assertions are part of the fixture but were not reached in the stub RED; they still require a later GREEN runtime result.

7. **Recheck advancement and no stale parent reuse.** There is no cached result or mutable function state: each invocation reads the history's final row and queries relay anew (`run_observation_present_context.go:42-75`). The unchanged fixture exercises relay advancement before capture, then a new retained matching row with a removed parent and independent earlier result (`run_observation_present_context_test.go:408-443`). Those dynamic assertions remain unproven until a separately authorized GREEN gate.

8. **Uniform failures and no payload disclosure.** Every explicit rejection calls the same closure, which constructs the zero result and typed `apperr.KindUnavailable` (`run_observation_present_context.go:30-36,38-70`). Messages and metadata are fixed constants; no source facts, cursors, IDs, or raw error values are attached. Success is returned only after all checks, so validation does not leak a partial parent map.

## Typed-nil dependencies and limits

`nilRunObservationPresentContextDependency` handles both a nil interface and a typed nil dynamic value. It checks nilable reflect kinds before calling the dependency methods (`run_observation_present_context.go:78-89`); pointer-backed nil implementations are therefore rejected by the same entry guard. The frozen fixture tests nil interface history and relay (`run_observation_present_context_test.go:209-243`), but does not construct typed-nil implementations. I found the implementation branch correct by source inspection; its typed-nil behavior has no direct assertion in this fixture and is not claimed as runtime-tested.

No material implementation deviation from the eight original requirements was found. The source is private and unwired. The approval is limited to source review: the exact success, immutability, repeat, newest-only, and advancement paths still need the runtime evidence root assigns. There is no watcher, manager, caller, authorization decision, atomic snapshot/relay guarantee, or prevention of between-check races here.
