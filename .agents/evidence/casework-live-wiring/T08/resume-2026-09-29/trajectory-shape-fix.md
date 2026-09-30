# T08 F13 trajectory shape builder note

Date: 2026-09-29

## Original assignment

> Fresh bounded builder to repair independent T08 critic F13. Read root AGENTS/current status + Graft skill + nearest instructions; source-only until token granted. Sole source files to edit: `apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts` and `httpTrajectory.test.ts` alongside it. Critic original finding: `isTemporalTrajectoryResponse` checks requested case/nonempty base/head/points array/point cursor only, then `queryTemporalTrajectory` casts malformed missing fields canonical. The canonical `.agents/reports/interface-contracts/typescript/types.ts` `TemporalCheckpoint` requires timestamp, event_type, summary, actor_id, actor_role strings, consequential boolean, completed_plan_items_count/total_plan_items_count numbers; active_stage_id optional string. Same-case `{cursor:'01A'}` was accepted silently missing all required fields. Implement smallest correct full canonical shape validator, allow honestly unknown actor empty strings, optional stage absent, existing valid server DTOs. Reject invalid/missing/wrong-typed required fields via CaseworkHttpError INVALID; enforce finite/count domains if supported by semantics and document material differences. Add table-driven regressions for all required fields missing/wrong types; positive empty actor/optional stage; preserve case ID/cursor existing tests and native stream logic unchanged. No dependencies/contracts/global refactor. Do not run tests/build/vet/compile: T07 independent critic owns sole compile token. Source inspect/Graft/diff check only. Provide source diff, exact original instructions/result/material deviations and pending test commands in this fresh evidence file.

## Result and source diff

- `httpCaseworkAdapter.ts` now validates each point as a `TemporalCheckpoint`: required string and boolean fields, optional string `active_stage_id`, and numeric counts before the response is narrowed to `TemporalTrajectoryResponse`.
- Count checks reject non-finite/non-integer values (via `Number.isInteger`), negative values, and completed counts greater than total. Go emits integer counts derived from enumerated work items/milestones, and completion is a subset of the total, so these are supported wire semantics.
- `httpTrajectory.test.ts` retains the prior valid server DTO, wrong-case, and missing-cursor tests; it adds a table-driven missing/wrong-type matrix for each required response/checkpoint field, invalid optional-stage type and count-domain cases, and a positive case for empty unknown actor strings plus omitted active stage.
- No stream code, public contract, dependency, or unrelated source was changed. Full reviewable diff is in the two source paths above.

Validator diff:

```diff
 function isTemporalTrajectoryResponse(value: unknown, caseId: string): value is TemporalTrajectoryResponse {
   if (!isRecord(value) || value.case_id !== caseId) return false
   if (typeof value.base_cursor !== 'string' || value.base_cursor.length === 0) return false
   if (typeof value.head_cursor !== 'string' || value.head_cursor.length === 0) return false
-  return Array.isArray(value.points) && value.points.every((point) =>
-    isRecord(point) && typeof point.cursor === 'string' && point.cursor.length > 0,
-  )
+  return Array.isArray(value.points) && value.points.every(isTemporalCheckpoint)
 }
+
+function isTemporalCheckpoint(value: unknown): value is TemporalCheckpoint {
+  if (!isRecord(value)) return false
+  const completed = value.completed_plan_items_count
+  const total = value.total_plan_items_count
+  return (
+    typeof value.cursor === 'string' && value.cursor.length > 0 &&
+    typeof value.timestamp === 'string' &&
+    typeof value.event_type === 'string' &&
+    typeof value.summary === 'string' &&
+    typeof value.actor_id === 'string' &&
+    typeof value.actor_role === 'string' &&
+    typeof value.consequential === 'boolean' &&
+    (value.active_stage_id === undefined || typeof value.active_stage_id === 'string') &&
+    typeof completed === 'number' && Number.isInteger(completed) && completed >= 0 &&
+    typeof total === 'number' && Number.isInteger(total) && total >= 0 &&
+    completed <= total
+  )
+}
```

## Material deviations and semantics

- No requested scope was omitted. The numeric checks are stricter than the TypeScript interface's bare `number` annotations, matching the Go `int` counters and their derived meaning.
- Other required strings are type-checked but not required to be non-empty: the Go producer can truthfully emit empty timestamp/summary/event strings when source data is absent, as well as empty actor attribution because relay revisions do not identify the originating actor. Cursor fields keep the existing non-empty rule.
- `active_stage_id` may be absent as the Go DTO's `omitempty` behavior requires; when present it must be a string. An empty `points` array remains structurally accepted because the existing contract requires an array but defines no minimum length; the current Go endpoint returns not-found rather than a successful empty trajectory.

## Verification status

- Ran source-only retrieval with Graft and `git diff --check`; diff check passed with no output.
- Did not run any test, build, vet, or compile command, per the sole-token restriction.
- Pending after the token owner releases it: `cd apps/godspeed-cognitive-ui && bun test src/adapters/http/httpTrajectory.test.ts`.
- Pending milestone gate after token release: `just casework-ui-check` from repository root.
