# T08 independent recovery review — bounded findings

**Verdict: REJECT (round 2, static review).** This is an independent review of the fresh T08 recovery changes against the T08 contract. It records defects found in the trajectory and browser event-stream implementations. It does not settle T08: the shared real-kernel conformance run and focused tests remain pending, and the stale-refetch case-switch changes have not yet received test execution under the exclusive compile-token rule.

## Evidence and review boundary

- Read repository, Workbench, and agent-workbench instructions; current status and machine status; the complete casework-environment spec; and the complete T08 plan section.
- Reviewed the current T08 diff and fresh trajectory, artifact, stale-refetch, and live-connection source/tests. No source was changed by this critic.
- No command was run before the parent granted the focused Bun token. No Go, Cargo, typecheck, build, vet, browser, or shared conformance command was run in this review.
- Graft retrieval identified the HTTP adapter, port contract, live state, and server event path before direct source inspection.

The parent later granted this critic the exclusive compile token for four focused Bun files only. `free -m` showed 2331 MiB available. Ran `bun test src/app/intents.test.ts src/app/live.test.ts src/adapters/http/httpCaseworkAdapter.test.ts src/adapters/http/httpTrajectory.test.ts` from `apps/godspeed-cognitive-ui`: **exit 1, 17 pass / 3 fail / 70 assertions**. Exact output is preserved in `bun-focused-round2.log`. The two new stale-refusal tests fail their judgment-outcome assertions because the test setup never invokes/opens judgment before `submit(..., { judgment: true })`; their case-switch and wrong-case assertions pass. The existing `queryTemporalTrajectory is honestly unsupported` HTTP test is obsolete and attempts `gw.test`, ending in `getaddrinfo ETIMEOUT`. `live.test.ts` and `httpTrajectory.test.ts` pass. The compile token is released after this focused run; no Go command or broad Bun gate was run.

## Findings

### F-7 — Trajectory counts rejected work as completed (blocking)

`checkpointFromRevision` increments `CompletedPlanItemsCount` for both `COMPLETED` and `REJECTED` objects (`apps/godspeed-casework-go/internal/server/trajectory.go:94-100`). The focused endpoint test deliberately includes one `COMPLETED`, one `REJECTED`, and one `WAITING` object, then expects a completed count of two (`apps/godspeed-casework-go/internal/server/trajectory_test.go:19-26,75-76`). Rejected work is not completed work; this makes historical trajectory counts materially false. The T08 change requires the live adapter's trajectory to implement the existing typed response (`.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml:479-487`).

**Required:** count only source statuses that mean completed, add a regression assertion for a rejected item, then re-run the focused and shared live conformance checks after the compile token is released.

### F-8 — Known lifecycle events are reported as non-consequential (blocking)

The T08 endpoint maps a retained kernel event kind directly into `event_type` and derives its `consequential` flag from `knownConsequentialEvent` (`trajectory.go:84-90`). That function marks only `case.submitted`, `plan_mutated`, `item_activated`, `settlement_recorded`, and `item_completed` (`trajectory.go:124-132`). The authoritative `TraceKind` includes other state-changing kinds such as `CaseClosed`, `MilestoneAchieved`, `ItemEnabled`, `ItemFailed`, `ItemTerminated`, `HumanTaskCompleted`, `CaseReopened`, and `CaseTerminated` (`crates/sea-forge-core/src/types.rs:501-525`); the SFWP mutation publisher emits every appended trace kind as `case.trace.<snake_case-kind>` (`crates/sea-forge-server/src/sfwp/case_mutations.rs:141-166`). These known lifecycle changes are currently returned with `consequential: false`, which understates material history. Unknown kinds should remain false; omission of known kinds is not a safe unknown fallback.

**Required:** classify all source-known state-changing/lifecycle kinds explicitly, keep genuinely informational/internal and future unknown kinds non-consequential, and cover representative positive and negative cases in trajectory tests.

### F-9 — Browser EventSource does not implement the required bounded backoff (blocking)

T08 explicitly requires `EventSource` subscription with Last-Event-ID resume **and backoff** (`.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml:479-486`). The browser branch constructs one native `EventSource`, then relies on its internal retry; `reconnectMaxMs` appears only in the error message (`apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:278-292`). The configurable exponential backoff is implemented only in `subscribeEventsFetch` (`httpCaseworkAdapter.ts:295-347`), selected only when `EventSource` is absent (`:272-276`). The Go server accepts Last-Event-ID and `?last=` and replays retained revisions (`apps/godspeed-casework-go/internal/server/server.go:276-310`), but does not emit an SSE `retry:` directive (`:288-334`). Thus the production browser path has resumability but no adapter-controlled/configured backoff cap or exponential policy; a cap mentioned in diagnostics is not behavior.

**Required:** implement and test the required bounded retry behavior for the browser path, preserving credentials and last-cursor replay; exercise a disconnect/reconnect against the real gateway, including retained replay and interruption/recovery state.

## Other material scope observations

- The new common conformance file is present, but I have not observed a completed same-suite run against both the local adapter and a fresh live kernel. A mocked HTTP test is not live-stack evidence.
- The adapter's new trajectory method casts the response body directly to `TemporalTrajectoryResponse` without validating the returned `case_id` or shape (`httpCaseworkAdapter.ts:250-256`). Conformance must prove that the endpoint and adapter remain case-scoped.
- The artifact route returns actual bytes and checks actual SHA-256, but its response only has run-level provenance populated; case, item, and invocation fields remain empty because the injected `ArtifactContent` exposes only digest, run ID, evidence ID, URI, size, and bytes (`apps/godspeed-casework-go/internal/server/artifact.go:24-33,86-93`; `internal/ports/ports.go:355-363`). Do not claim full provenance until the real shared suite establishes the accepted contract or the source-backed missing fields are handled.
- The fresh stale-refetch source includes a deferred case-switch guard and wrong-case test, but no test/typecheck was run. Static review is not confirmation of that race fix.

## Required disposition

Keep T08 unconfirmed. Preserve this rejection, assign fresh builders to F-7 through F-9, then obtain the exclusive compile token and run the T08 UI gates plus shared conformance against local and the real kernel. Re-review the source and runtime evidence independently before settlement.
