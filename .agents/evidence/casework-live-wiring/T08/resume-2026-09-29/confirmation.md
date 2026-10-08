# T08 independent confirmation — round 1

**Verdict: REJECT.** Reviewed T08 commit `ce538bf` plus current downstream T09 `HEAD 3e2404f` because T09 changed the adapter, session bootstrap, and UI intent path. T08 has a working HTTP adapter for several live operations, but the production-only source guarantee and multiple required port behaviors are not satisfied. The findings below are sufficient to withhold confirmation. No product code, current-status file, or prior evidence was changed by this critic.

## Verification performed

- Reviewed `.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml` T08, `.agents/reports/2026-09-23-case-engine-frontend-e2e-journey-mapping.md` Section 0, and `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml`.
- Inspected the T08 diff and current UI/gateway code at `ce538bf` and `3e2404f`.
- Production selector reproduction, with outputs isolated under `/tmp`:
  - `VITE_CASEWORK_SOURCE=local ./node_modules/.bin/vite build --outDir /tmp/T08-prod-local` — exit 0; emitted `assets/localAdapter-Cy-YJ1r8.js` (88.21 kB).
  - `VITE_CASEWORK_SOURCE=typo ./node_modules/.bin/vite build --outDir /tmp/T08-prod-invalid` — exit 0; emitted the same local adapter chunk.
  - Current default `apps/godspeed-cognitive-ui/dist/assets/*.js` has no `LocalContractAdapter|northstarData` marker. This confirms the default build passes while explicit `local` and invalid values still select and bundle local data.
- Fresh-cell live conformance used existing `target/debug/sea-forge-server` and `/tmp/casework-resume-gateway` on isolated `127.0.0.1:4187`. The first sandboxed attempt failed before HTTP because the server could not create its Unix socket (`PermissionDenied: Operation not permitted`). Re-running with the allowed local-process escalation started both processes against `/tmp/t08-live-cell-cFzZEx` and exited 0. The run observed 15 `PASS` lines: anonymous world denied; session login; templates; preflight; case proposal/name; sentry snapshot/action standing; execute; SSE cursors advanced; durable `item_activated`; typed `STALE_PROJECTION`; the script's no-write check; second session identity. The no-write check is invalid as written (`|| true`, finding F-3), so it is not counted as proved. The live run did not exercise artifact resolution, temporal trajectory, disconnect/reconnect, or the UI's stale-refusal path.
- No `bun test`, `bun run typecheck`, default production gate build, Go tests, or Cargo tests were rerun in this round. The recorded T08 gate logs remain builder evidence, not independent results. Alternate production builds above were direct reproduction builds only.

## Findings

### F-1 — Production builds can select the local fixture adapter (blocking)

`src/main.tsx:33-34` accepts `VITE_CASEWORK_SOURCE=local` even when `import.meta.env.DEV` is false, and maps every unrecognized value to `local`. There is no other `VITE_CASEWORK_SOURCE` validation in the app, Vite config, package scripts, `.github`, or justfile. The default build excludes the local marker, but the two production builds above emit the local adapter chunk. This contradicts T08's production HTTP-only requirement and its comment that production cannot select local. A typo in production configuration also silently boots the local fixture.

### F-2 — `resolveArtifact` cannot return artifact content from the live gateway (blocking)

`src/adapters/http/httpCaseworkAdapter.ts:204-229` expects a successful `OPEN_ARTIFACT` intent to contain `resulting_object.content_base64`; without it the adapter throws `UNAVAILABLE`. In `apps/godspeed-casework-go/internal/intents/intents.go:254-258,403-413`, the gateway checks/fetches the digest but returns `nil` as the resulting object. `internal/server/server.go:109-132` registers no artifact content endpoint. The adapter therefore cannot fulfill `resolveArtifact` against this gateway, despite T08 requiring the method and T09 relying on it for the artifact dock. This was established by code inspection; no live artifact request was attempted.

### F-3 — Stale response is shown, but UI does not refetch; live no-write tooth is tautological

The adapter surfaces an intent refusal and does not retry it (`httpCaseworkAdapter.ts:189-200`). The live run received `STALE_PROJECTION` from one `POST /api/intents`; this confirms the typed refusal path. But the required UI refetch is absent: `src/app/intents.ts:74-92` records the refusal message/code and returns, without calling `getSnapshot` or otherwise refetching the case. The intent may be shown honestly while the projection remains stale. Also, `e2e/live-conformance.ts:233-234` claims to assert no new kernel events with an expression ending `|| true`, so that assertion passes regardless of the event file. The gateway staleness guard itself returns before `execute` (`apps/godspeed-casework-go/internal/intents/intents.go:118-176,437-469`), but the adapter-level runtime tooth does not prove no write.

### F-4 — Required shared port-conformance suite was not extracted or run

T08 requires extracting a shared conformance suite from `localAdapter.test.ts` and running that same suite against both adapters. The local suite remains adapter-specific (`src/adapters/local/localAdapter.test.ts:10-15` constructs `LocalContractAdapter`), while `httpCaseworkAdapter.test.ts` has separate network mocks. `e2e/live-conformance.ts` is a handwritten subset against the real stack, not the shared suite, and its run did not call every port method. The plan's central same-suite claim is therefore unproven and not implemented.

### F-5 — `interrupted` connection state is missing

T08 requires the UI store to surface `live`, `reconnecting`, and `interrupted`. Current state types and actions admit only `live | reconnecting` (`src/model/types.ts:471-472,518`); `connectLive` switches to `reconnecting` on any error and back to `live` on the next event (`src/app/live.ts:50-83`). No terminal/long-outage transition to `interrupted` exists. An outage can remain labeled “reconnecting” indefinitely.

### F-6 — Temporal trajectory method is an explicit unsupported stub

`src/adapters/http/httpCaseworkAdapter.ts:242-250` always throws `UNAVAILABLE` for `queryTemporalTrajectory`, whereas T08 lists it among the live adapter methods. This leaves the method unavailable for every authenticated live case. The implementation comments say cursor history may be used, and `getSnapshotAt` exists; this review does **not** establish whether those sources can fully produce `TemporalTrajectoryResponse`, so I am not declaring a redesign trigger. The builder should either implement the required response from canonical history or provide evidence for a plan-level redesign decision.

## Other observed deviations and limits

- Browser `EventSource` uses user-agent reconnect behavior (`httpCaseworkAdapter.ts:266-280`); the configured exponential backoff fields are used only by the fetch-stream fallback (`:283-336`). The live browser branch does not apply the configured cap. The isolated run covered event delivery but not an actual disconnect/reconnect or `Last-Event-ID` header replay.
- `dispatchIntent` carries the caller's actor fields in its JSON request. Correct attribution relies on the T07 gateway overwriting those fields from the authenticated session before dispatch; the live conformance demonstrated the session perspective, but did not forge the request actor and inspect ledger attribution.
- No Section 0 fact was found to require correction. The T08 deviations above are against the task's own explicit contract.

## Required closeout for a later confirmation round

1. Make the production source compile-time HTTP-only and fail closed for invalid source configuration; reproduce `local` and invalid builds under a temp outDir and assert neither can contain local adapter assets.
2. Return authenticated, digest-verified artifact bytes through a supported live gateway route and make `resolveArtifact` consume that contract.
3. Refetch current projection on stale refusal while preserving the refusal for the user and never replaying the intent; replace the tautological no-write check with a before/after durable event assertion.
4. Extract one reusable CaseworkPort conformance suite and execute it against both adapters, with live suite evidence against a fresh kernel cell.
5. Add and surface the required `interrupted` state; test a prolonged outage and recovery.
6. Implement `queryTemporalTrajectory` or document a spec/plan decision backed by the actual history capabilities and acceptance impact.
7. Re-run T08's UI gate and the real-stack teeth after the fixes; keep all new evidence in a new dated directory.
