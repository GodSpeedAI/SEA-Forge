# server

## Overview
The casework boundary's HTTP+SSE surface in two shapes: the LIVE server (`Server` — canonical
spec-04 CognitiveWorldSnapshots built from the governed kernel through the SFWP client, intents
translated onto governed SFWP verbs, kernel event frames relayed as SSE revisions keyed by the
kernel's own event cursor, sessions/CSRF/rate limits) and the FIXTURE-LABELED surface
(`FixtureServer` — the in-process Northstar stack, no sessions, no auth, build-tag gated). The wire
contract is `.agents/reports/casework-live-wiring/adr-wire-contract.md`; refusals are typed
outcomes, never transport errors. The live server holds no case state of its own: world source,
intent dispatcher, template source, revision history and relay are injected, and there are no
globals.

## Interface
### Route registration (live `Server.Handler()`, [server.go:Handler])
| Route | Guard chain |
|---|---|
| `GET /api/healthz` | none (cheap, no kernel call) |
| `GET /api/readyz` | none (kernel `readiness.get`) |
| `GET /api/auth/login` / `POST /api/auth/login` / `GET /api/auth/callback` | auth entry points |
| `POST /api/auth/logout` | `requireSession(requireCSRF(...))` |
| `GET /api/session` | `withIdentity` |
| `GET /api/world` | `requireSession` |
| `GET /api/events` | `requireSession` |
| `GET /api/templates` | `requireSession` |
| `GET /api/artifacts/{digest}` | `requireSession` |
| `GET /api/trajectory` | `requireSession` |
| `POST /api/intents` | `requireSession(requireCSRF(rateLimitIntent(...)))` |
| `POST /api/ask` | `requireSession(requireCSRF(rateLimitAsk(...)))` |
| `POST /api/templates/preflight` | `requireSession(requireCSRF(...))` |
| `GET /api/` catch-all | typed JSON 404, never SPA fallback |
| `GET /` (when `static_root` set) | `staticHandler()` with CSP |

### Route registration (fixture `FixtureServer.Handler()`, [fixture_server.go:Handler])
`GET /api/healthz`, `GET /api/world`, `GET /api/time`, `GET /api/events`, `POST /api/intents`,
`GET /api/artifacts`, `POST /api/artifacts`, `GET /api/artifacts/{ref}` — no sessions, no auth.

### Types and constructors
- `Server` — live surface: `New(world, dispatcher, tpl, store, relay, opts)`, `NewWithArtifacts(...)`; `Handler()` [server.go:Server]
- `FixtureServer` — fixture surface: `NewFixtureServer(proj, coord, arts, opts)`; `Handler()` [fixture_server.go:FixtureServer]
- Injected-role interfaces: `WorldSource`, `PerspectiveVerifier`, `TemplateSource`, `IntentDispatcher`, `RevisionHistory`, `RelayCursors`, `ArtifactGetter`, `ReadinessProbe` [server.go:WorldSource] [session.go:ReadinessProbe]
- `Options` — heartbeat, auth, static root, rate limits, trusted origins [http.go:Options]
- `Relay` — kernel-frame relay: `NewRelay(feed, source, store, opts)`, `Run(ctx)`, `Head`, `CursorForCase`, `WaitForCaseAdvance`; `KernelEvent`, `EventFeed`, `NewSFWPFeed` [relay.go:Relay] [feed.go:NewSFWPFeed]
- Session layer: `withIdentity`, `requireSession`, `requireCSRF`, `resolveIdentity`, `handleLogin`, `handleLoginStart`, `handleLoginCallback`, `handleLogout`, `handleSession`, `handleReadyz`, `setAuthCookies`, `clearAuthCookies`, `AuthOptions`, `requestIdentity` [session.go]
- Run-observation subsystem (private): `runObservationManager`, `runObservationLease`, poller workers, retained-state images, delta windows, hydration cap (1 MiB), selection limit (8 runs) — per the package comment this manager is not wired into `Server` or an SSE path [run_observation_manager.go:runObservationManager] [run_observation_hydration_cap.go:boundRunObservationHydration]
- Helpers: `writeJSON`, `writeTypedError`, `requireJSON`, `decodeStrict`, `writeSSE`, `withSecurityHeaders`, `withLocalCORS`, `withOriginCORS`, `newCSRFToken`, `correlationID`, `mintCorrelationID` [http.go] [csrf.go] [logging.go]

## Invariants
- Every route except `GET /api/healthz`, `GET /api/readyz` and the auth entry points requires an authenticated session [server.go:Handler]
- State-changing POSTs require a valid CSRF synchronizer token BEFORE any kernel call [session.go:requireCSRF]
- Identity is session-bound: the pre-T07 `?actor=&role=` override is refused and the client-asserted actor is overwritten with the session's kernel standing [session.go:resolveIdentity]
- Refusals are typed outcomes (HTTP 200), never transport errors [server.go:package doc]
- POST bodies are size-limited (1 MiB general, 8 KiB ask) and strictly decoded (unknown fields, trailing values rejected) [http.go:maxBodyBytes] [ask.go:maxAskBodyBytes]
- At most 8 SSE streams per session (`maxSSEPerSession`) [server.go:maxSSEPerSession]
- Unknown `/api/*` paths are typed JSON 404s and never fall through to the SPA shell [static.go:package doc]
- Served documents carry the fixed CSP in `cspHeader` (including the documented `style-src 'unsafe-inline'` honesty note) [static.go:cspHeader]
- Relay rebuild failure never stalls the relay: the observed cursor still advances; mutation waiters advance only after a revision is retained [relay.go:package doc]
- `X-Forwarded-For` is not trusted for rate limiting; buckets are per session AND per remote IP [ratelimit.go:remoteIP]
- The run-observation manager constants bound cohorts/pollers at 16 each; hydration is capped at 1 MiB [run_observation_manager.go:runObservationCohortLimit] [run_observation_hydration_cap.go:runObservationHydrationMaxBytes]

## Side Effects
- Serves HTTP+SSE: sessions, cookies (`casework_session`, `casework_csrf`), SSE streams with 15 s heartbeats [session.go] [server.go:handleEvents]
- Calls the kernel through injected sources (world, templates/preflight, intents, readyz readiness, artifacts, run trace) [server.go]
- Rate limiting mutates in-process token buckets (bounded at 4096) [ratelimit.go:MaxBuckets]
- Serves static UI files from `static_root` with cache headers and CSP [static.go:staticHandler]
- Maintains per-case cursor waiters and subscriber channels in the relay [relay.go:Run]
- Emits request logs with correlation ids; `POST /api/intents` correlation id IS the intent id [logging.go:correlationID]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
