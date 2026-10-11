# T08 F6 trajectory builder note

Date: 2026-09-29

## Original assignment

Implement T08 F6 `queryTemporalTrajectory` from existing source-backed retained revisions, or identify an actual contract redesign trigger if the source cannot satisfy the existing `TemporalTrajectoryResponse`. Add bounded, case-specific, cloned history access; expose an authenticated live `GET /api/trajectory`; mirror the existing TypeScript shape; do not invent history, attribution, or effects. Keep changes to the Go projection/store and trajectory endpoint plus the HTTP adapter method and focused tests. Do not change the contract or unrelated T08 work. Do not run Go or Bun verification until the parent grants the exclusive compile token.

## Implementation

- `internal/projection/store.go`: added `Store.Trajectory(caseID)` which filters the store's globally bounded retained revisions by case and returns deep snapshot copies. Snapshot cloning now also protects pointers and nested slices from caller mutation while preserving nil slices.
- `internal/projection/store_test.go`: added case-scope, cursor-order, retention-eviction, and clone-isolation coverage.
- `internal/server/trajectory.go`: added a private Go mirror of the already-existing TypeScript `TemporalTrajectoryResponse`/`TemporalCheckpoint` wire fields and the GET handler. It validates exactly one non-empty `case_id`, requires a cookie-backed login session, verifies the session perspective through `PerspectiveVerifier` before querying retained history, and emits typed invalid, authority-denied/unavailable, and not-found responses.
- `internal/server/trajectory_test.go`: added endpoint coverage for authenticated case-scoped history, response fields, unknown actor attribution, invalid/empty history, unauthenticated access, and verifier refusal.
- `src/adapters/http/httpCaseworkAdapter.ts`: replaced the unsupported stub with an authenticated GET to `/api/trajectory?case_id=...`, preserving typed gateway refusal handling. Updated the nearby adapter comment.
- `src/adapters/http/httpTrajectory.test.ts`: added focused HTTP adapter success and typed-not-found tests.
- `internal/server/server.go`: route registration and the `RevisionHistory.Trajectory` interface seam were added by the separate T08 adapter builder, not by this builder.

## Semantics and limits

The response contains only revisions actually retained by the in-memory relay store. Retention is globally bounded by the store's 512 revision default; the endpoint does not reconstruct history from current state or read kernel ledgers directly. If there are no retained revisions for a requested case, the endpoint returns `404 not_found` with a message explaining that the case may be unknown, relay replay may not have completed, or its revisions may have been evicted. Those cases cannot be distinguished from this retained store API. If some older points were evicted, the response reports the available retained `base_cursor`, `head_cursor`, and points; the existing contract has no completeness/eviction indicator.

Kernel relay frames expose cursor, event kind, case ID, and timestamp, but not originating actor metadata. `actor_id` and `actor_role` are therefore empty strings; the snapshot's serving perspective is not substituted as event attribution. `event_type` and `summary` use the source frame kind. Consequential is marked true only for the explicitly listed known state-changing kinds (`case.submitted`, `case.trace.plan_mutated`, `case.trace.item_activated`, `case.trace.settlement_recorded`, `case.trace.item_completed`); unknown kinds do not receive inferred effects. Active stage and item counts are derived from each retained snapshot's stage/work-item/milestone objects and statuses.

No contract or spec semantics were changed. The existing fields are sufficient when truthful unknown actor fields and bounded available history are accepted; no plan-level redesign trigger was identified.

## Verification

- `gofmt -w` on the four Go source/test files: run after source completion (exit 0; no output).
- `gofmt -l` on those files: run after formatting (no output).
- Scoped `git diff --check`: run after source completion (exit 0; no output).
- Go tests/build/vet and Bun tests/build/typecheck: **not run by this builder**, per the exclusive compile-token instruction. The T07 builder's gates ran while the initial store patch already existed, but this builder did not run or claim those gates. Parent assigned the exclusive compile token to the T07 critic after this source was reported stable.

This note was staged at `/tmp/T08-trajectory-builder.md` because the repository evidence tree is read-only under the active filesystem permission profile. It is not an immutable repository evidence artifact yet.

## Round 2 vet correction

The independent T07 round 2 vet found `net/http/httptest` imported but unused in
`internal/server/trajectory_test.go`. The fresh correction removed only that import and ran
`gofmt -w` plus `gofmt -d` on the file; formatting verification was clean. No compile, test, or vet
was run for this correction because the compile token belongs to the critic. The independent
critic/root must rerun vet before treating this source as verified.
