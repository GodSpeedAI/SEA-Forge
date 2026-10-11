# Unit6 fixture source preparation

Independent read-only preparation by t09_ask_fixture_independent_critic; no test or
implementation write release, no compiler. Original canonical contract, approved proposal
and ui-observation-boundary-root-decision.md govern. Root retains fixture specifications.

Minimal new adapter fixture: src/adapters/http/httpCaseworkAdapter.observations.test.ts.
Use nearby native FakeEventSource/manual timers and fetch mockFetch/ReadableStream patterns.
Adapter native/fetch routing lives at httpCaseworkAdapter.ts:297-447; shared permissive
parseStreamEvent at530-549. Both need strict observation validation before case cursor
comparison, native event allowlist, no-ID informational cursor isolation and unchanged
legacy delivery. Test reconnect URL retains ordinary cursor, stale/equal observations
deliver, ordinary replay filters and later ordinary revisions pass.

Root requires the optional case_id request seam in BOTH transports and server unit5;
the current lack of that URL parameter is an implementation omission identified by source
preparation, not an authorization block. Existing URL fixtures will need scoped assertions
while proving legacy replay semantics. Do not infer requested case from global Store state.

Coverage: exact canonical envelope/budget/run/frame shapes, counts and enums, unsafe or
fractional exits, command metadata only on actual command_finished, all frame/cache caps,
pair identity(run,event), per-run/global oldest-frame eviction and replay after eviction,
terminal-key LRU, full protected-key admission rejection and typed capacity notice with
ordinary revisions flowing. Every admitted run has map entry even[]; rejected runs absent.
Cache survives internal reconnect, clears on unsubscribe/case switch. Original canonical
envelope/counts stay intact. Malformed observation handling needs an explicit root decision;
the preparer's suggested transport-error behavior is not approved by this document.

Required separate app/model/view fixture: app/live.observations.test.ts plus actual render
guard test, using existing live.test.ts callback fake and Store initialState/loadHistory/
setTime patterns. app/live.ts16-140 has no observation path; model/types.ts439-480 has no
sidecar. App.tsx348-362/606 currently selects/renders current executions without past guard.
Prove sidecar bounded outside WorldHistory, case cleanup/dispose and actual historical view
suppression; Store-only assertions do not prove rendering. Both units6/6b remain required.

Open root choice before fixture release: terminal predicate for run-key LRU and poll stop.
The approved proposal explicitly orders completed/failed/terminated execution as terminal,
but future assignment must state its chosen semantics and settlement freshness limits.
No fixture may silently invent a settlement-to-execution terminal rule.
