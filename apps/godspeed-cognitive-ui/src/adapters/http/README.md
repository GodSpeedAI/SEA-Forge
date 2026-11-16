# HTTP Casework Adapter (live gateway)

## Overview
The production `CaseworkPort`: the contract port served by the Go casework gateway (`apps/godspeed-casework-go`) over same-origin fetch + SSE (`src/adapters/http/httpCaseworkAdapter.ts`). Identity comes from the gateway session — the adapter never sends a client-asserted actor. Refusals surface as the typed T01 envelope, verbatim; the adapter never retries a mutation (a `STALE_PROJECTION` refusal means the UI refetches and shows the refusal).

## Interface
- `HttpCaseworkAdapter` — implements `CaseworkPort`, `SessionPort` and `TemplateSourcePort` [httpCaseworkAdapter.ts:HttpCaseworkAdapter]
  - Session: `session()`, `login(username, password)`, `logout()` (GET/POST `/api/session`, `/api/auth/login`, `/api/auth/logout`)
  - Reads: `getSnapshot` (GET `/api/world?case_id=`), `getSnapshotAt` (adds `&cursor=`), `queryTemporalTrajectory` (GET `/api/trajectory`)
  - Authority: `dispatchIntent` (POST `/api/intents`)
  - Artifacts: `resolveArtifact` (authenticated GET `/api/artifacts/{digest}`)
  - Events: `subscribeEvents` (SSE `/api/events`; EventSource in browsers, a fetch-streaming SSE parser elsewhere)
  - Templates: `getTemplates` (GET `/api/templates`), `preflightTemplate` (POST `/api/templates/preflight`)
  - Narration: `ask(request, signal)` — governed Thoth Ask (POST `/api/ask`) [httpCaseworkAdapter.ts:HttpCaseworkAdapter.ask]
- `HttpRefusalError` — typed pre-dispatch refusal with `refusalKind` and optional `intentId` [httpCaseworkAdapter.ts:HttpRefusalError]
- `HttpAdapterOptions` — `base`, `credentials`, `reconnectBaseMs`, `reconnectMaxMs` [httpCaseworkAdapter.ts:HttpAdapterOptions]
- `SessionIdentity` re-exported for callers [httpCaseworkAdapter.ts:SessionIdentity]

## Invariants
- The `actorId`/`role` arguments of `getSnapshot`/`getSnapshotAt` are accepted for interface compatibility and deliberately ignored for authority — the session's verified perspective is used [httpCaseworkAdapter.ts:HttpCaseworkAdapter.getSnapshot]
- The gateway cursor is monotonic: replayed frames at or below `lastCursor` are dropped, including the last frame at a reconnect boundary [httpCaseworkAdapter.ts:HttpCaseworkAdapter.subscribeEvents]
- `resolveArtifact` requires a session, demands the ref be a SHA-256 hex digest, validates the payload shape, and raises `INTEGRITY_MISMATCH` if the returned digest differs from the requested one [httpCaseworkAdapter.ts:HttpCaseworkAdapter.resolveArtifact]
- Trajectory responses are shape-validated (case-scoped, non-empty points, base/head cursor alignment) or rejected as `INVALID` [httpCaseworkAdapter.ts:isTemporalTrajectoryResponse]
- Governed `denied`/`partial` Thoth answers are successful bodies; transport and identity failures are typed errors [httpCaseworkAdapter.ts:HttpCaseworkAdapter.ask]
- `execution_progress` is accepted as an SSE event kind alongside `snapshot`, `settlement_recorded`, and `resync_required` [httpCaseworkAdapter.ts:HttpCaseworkAdapter.subscribeEvents]
- Reconnects use exponential backoff (base 500 ms, cap 10 s by default); EventSource's own unbounded retry loop is disabled and replaced [httpCaseworkAdapter.ts:scheduleReconnect]

## Side Effects
- HTTP fetches and SSE connections to the gateway origin; CSRF double-submit via the `casework_csrf` cookie and `X-CSRF-Token` header [httpCaseworkAdapter.ts:ensureCsrf, httpCaseworkAdapter.ts:rawPOST]
- A minimal in-memory cookie jar (from response `Set-Cookie` headers) is kept for non-browser runtimes (bun/tests) so session + CSRF cookies ride along [httpCaseworkAdapter.ts:HttpCaseworkAdapter.harvestCookies]
- Cached session identity (invalidated on login/logout) [httpCaseworkAdapter.ts:HttpCaseworkAdapter.identity]

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; adapter tests: `src/adapters/http/httpCaseworkAdapter.test.ts` and `httpCaseworkAdapter.nativeEvents.test.ts`, `src/adapters/http/httpTrajectory.test.ts`)
