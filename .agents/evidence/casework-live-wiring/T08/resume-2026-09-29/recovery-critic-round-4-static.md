# T08 independent recovery review — round 4 static findings

**Verdict: REJECT remains in force.** This review is static only. I did not run Bun, Go, Rust, typecheck, build, browser, or live-stack commands because the exclusive compile token has not been granted. I reviewed the current committed recovery source and prior evidence; builder logs are reported evidence, not independent verification. The fresh trajectory-shape builder now owns `httpCaseworkAdapter.ts` and `httpTrajectory.test.ts`; this report records the defect in the source reviewed before that builder's final diff and must be rechecked against its stable result.

## Findings

### F-13 — Trajectory response validator accepts incomplete canonical checkpoints (blocking)

`isTemporalTrajectoryResponse` verifies the requested top-level `case_id`, non-empty `base_cursor` and `head_cursor`, that `points` is an array, and only that each point has a non-empty string `cursor` (`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:101-107`). `queryTemporalTrajectory` returns that body as `TemporalTrajectoryResponse` (`:265-274`). The canonical `TemporalCheckpoint` requires string `timestamp`, `event_type`, `summary`, `actor_id`, and `actor_role`; boolean `consequential`; and number `completed_plan_items_count` and `total_plan_items_count`. If present, `active_stage_id` must be a string (`.agents/reports/interface-contracts/typescript/types.ts:336-346`).

A same-case response containing `points: [{ cursor: "01A" }]` therefore passes and is exposed as a canonical checkpoint with all other fields absent. Wrong-typed counts or booleans also pass. Consumers and future renderers receive trusted typed data with missing or malformed required fields; the common conformance suite reads checkpoint cursors and assumes trajectory endpoints agree with the first/last point (`apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:70-75`). The canonical type declares counts as `number`; I found no additional documented numeric domain, so non-negative integer enforcement should be tied to an explicit source rule if desired rather than claimed as an existing schema requirement.

Current adapter tests reject a foreign case, missing points/cursor in one malformed response, and a typed not-found; they do not reject absent required checkpoint fields, wrong field types, empty points, or base/head values that disagree with the point endpoints (`apps/godspeed-cognitive-ui/src/adapters/http/httpTrajectory.test.ts:61-99`). The original T08 strict identity/shape requirement and this recovery round require runtime malformed-field regressions. Do not approve this unit until the validator rejects those malformed shapes and focused tests exercise them.

### F-14 — Native EventSource reconnect/resync is not verified end-to-end (blocking evidence gap)

The current native EventSource tests use `FakeEventSource` and manual timers to check capped retry growth, open/drop cycles, cursor resume, and same-ID `resync_required` followed by its snapshot (`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts:41-123,159-242`). The Go server test separately verifies `resync_required` followed by replay of two retained revisions (`apps/godspeed-casework-go/internal/server/server_test.go:649-693`). The real-stack shared suite in `apps/godspeed-cognitive-ui/e2e/live-conformance.ts:188-253` runs normal retained-event resume through `HttpCaseworkAdapter`, but Bun selects the fetch fallback when `EventSource` is absent, and that suite does not disconnect/reconnect a native EventSource or exercise a one-retained-revision window.

These separate tests do not prove the production browser EventSource path against the server's resync protocol. Require a fresh live gateway integration using the native EventSource transport, force retention so the requested cursor predates a single remaining revision, and assert receipt/order of `resync_required` and that same-cursor authoritative snapshot without suppression. Also exercise a real interruption/reconnect and retained replay on this path. A fetch-stream test is not a substitute for the required native path.

## Findings from round 3 rechecked statically

- F-10 appears addressed: stale refresh now requires exact `fresh.case_id === caseId` before projection/merge (`src/app/intents.ts:82-85`).
- F-11 appears addressed: retry reset happens only after a parsed non-control source event; `onopen` does not reset the counter (`src/adapters/http/httpCaseworkAdapter.ts:319-337`).
- F-12 appears addressed in adapter ordering: the resync control event does not advance `lastCursor`, and its same-cursor snapshot may pass the duplicate guard (`src/adapters/http/httpCaseworkAdapter.ts:323-335`). F-14 remains because this interaction is covered only with a fake source, not against the live gateway.
- Artifact resolution now joins `artifact_get` to `run_get` for actual run/case/item/evidence linkage. Empty `invocation_id` is consistent with the interface string field and source evidence; no unsupported invocation identity should be inferred.
- The shared conformance suite is one implementation imported by both local tests and the live runner. The live no-write boundary compares case-event and approval ledger bytes. Its fresh real-stack run remains outstanding.

## Required independent verification after the fresh builder's stable diff

Check available RAM and receive the exclusive compile token before any command that invokes Bun, Go, Cargo, Vite, TypeScript, or a browser runtime. Then independently run and preserve output for:

1. The affected UI gate: from `apps/godspeed-cognitive-ui`, `bun run typecheck && bun test && bun run build`, including the new required-field regression tests.
2. Production adapter exclusion for the default build and explicit `VITE_CASEWORK_SOURCE=local` and invalid values. Build variants to isolated `/tmp` output directories and verify none emits a local-adapter chunk/marker; do not rely on a prior default-only bundle result.
3. The UI journey gate: from `apps/godspeed-cognitive-ui`, `bun run e2e`, recording any known baseline floor separately.
4. Fresh Go verification for changed gateway source: from `apps/godspeed-casework-go`, `go vet ./...` and `go test -race ./...` (serialize with repository-prescribed memory limits if needed).
5. The real shared port suite against a newly booted kernel and freshly rebuilt gateway: `CASEWORK_LIVE=1 bun run e2e/live-conformance.ts` from the UI directory. Verify the process/cell and output are the fresh instances; current script uses fixed `127.0.0.1:4179`, so rule out a pre-existing listener masking startup failure.
6. The F-14 native EventSource live integration described above, against the fresh gateway and a one-retained-revision scenario, including real interruption/reconnect and same-cursor resync snapshot delivery.

Until F-13 is repaired and independently tested, F-14 is exercised against the actual native/live path, and the listed applicable gates pass, T08 remains unconfirmed. No source, status, or commit was changed by this critic.
