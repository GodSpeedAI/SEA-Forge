# godspeed-casework-go

The casework system front end for the GodSpeed casework environment (plan T01). It coordinates
governed casework through application-owned ports and never becomes the semantic authority: SEA Forge
owns case meaning and settlement, Gauntlet executes, and this application coordinates and projects.

## Port ownership

`internal/ports` declares the application's own interfaces and value types. Provider, transport and
vendor shape stops at an adapter:

| Port | Provider side (adapter) | What the application sees |
|---|---|---|
| `AuthorityPort` | SEA Forge SFWP over its Unix socket | `CaseSummary`, `SettlementObservation`, `HistoryWindow` |
| `ExecutionPort` | Gauntlet executor | `ExecutionObservation` (an observation, never a settlement) |
| `RepositoryPort` | the repository provider for enabled workflows | `ChangeProposal` |
| `ArtifactStore` | artifact persistence — **optional** | `ArtifactRef` |

This is enforced, not asserted: `internal/boundary` scans the core packages (`ports`, `config`,
`preflight`, `apperr`) for (a) imports that reach an adapter or a provider, (b) provider or vendor
vocabulary in declared identifiers, and (c) untyped `any`/`interface{}` fields where a provider
payload could hide without naming itself. It also proves itself non-vacuous by failing on each of
those shapes in a synthetic source. Test files are exempt from the *import* rule only, because a test
may drive a fake adapter to exercise a contract and that edge does not ship.

The `godspeedai-stack` boundary rule this implies for later tasks (T04, T05): adapters translate INTO
`ports` types. If a required behaviour cannot be expressed without a provider type in a port, that is
a design fault to raise, not a type to alias.

## Configuration authority (one, documented)

Precedence, lowest to highest:

1. **Defaults** — `config.Defaults()`.
2. **Configuration file** — the `-config` flag, else `$GODSPEED_CONFIG`. JSON, and unknown keys are
   rejected (`DisallowUnknownFields`) so a typo fails loudly instead of being ignored.
3. **Environment** — `GODSPEED_CELL_ROOT`, `GODSPEED_EVIDENCE_ROOT`, and per capability
   `GODSPEED_CAPABILITY_<NAME>_{ENDPOINT,CREDENTIAL,REQUIRED}`.
4. **Explicit overrides** — `Options.Overrides` (`cell_root`, `evidence_root`,
   `capability.<name>.{endpoint,credential,required}`), for tests and operator tooling.

No adapter may introduce a competing order. Adapters receive already-resolved values.

### Secrets

A `credential` must be an **indirection**, never a value: `env:NAME` or `file:/path`. A bare string is
rejected with a typed configuration error, because REQ-CONFIG-002 forbids embedding secrets in stable
configuration. `config.Load` returns the resolved document *and* the secrets separately, and the
document keeps only the indirection name, so a secret cannot be logged or serialised with the
configuration by accident.

## Preflight and blast radius

`internal/preflight` evaluates every configured capability before any consequential work and reports a
state per capability:

* **ready** — the capability answered.
* **degraded** — an *optional* capability is unavailable; the application proceeds without it.
* **blocking** — a *required* capability is unusable; the application refuses to proceed (exit 2).

A capability-scoped configuration fault disables **only that capability**: `config.Load` returns every
problem rather than stopping at the first, `config.Fatal` selects the document-level faults that
prevent startup, and capability-scoped faults travel into preflight so unrelated capabilities keep
their own honest states. The end-to-end behaviour is recorded in the T01 teeth.

## Serve mode (fixture-labeled)

`-serve` starts the casework boundary's HTTP+SSE surface after preflight. It binds loopback
(`-addr`, default `127.0.0.1:4179`), blocks on SIGINT/SIGTERM, and shuts down gracefully. Serve
mode still requires a configuration file (the `-config` flow is unchanged; a missing or invalid
config exits 2 exactly as in CLI mode); `configs/fixture-serve.json` is a minimal valid example:

```sh
go run ./cmd/godspeed-casework -serve -config configs/fixture-serve.json
```

In serve mode every configured capability is answered by the FIXTURE-LABELED in-process provider,
so preflight reports them `ready` instead of `unavailable`. The example config marks them all
`required: false`; an operator may flip a capability to `required: true`, which additionally
demands an `endpoint` and a credential indirection per the schema rules below — the fixture
provider never reads the credential, but the configuration discipline still applies.

### Endpoints

Wire contract: `apps/godspeed-cognitive-ui/src/adapters/go/WIRE.md` (authoritative for shapes).

| Endpoint | Behaviour |
|---|---|
| `GET /api/healthz` | `{ "status": "ok", "provenance": "go:fixture:northstar", "liveCursor": N }` |
| `GET /api/world` | `{ "snapshot": … }` at the live cursor; `?cursor=` serves that revision, unknown or malformed cursor → 404 typed error `{"error":{"kind":"invalid","note":"unknown cursor"}}` |
| `GET /api/time` | `{ "positions": [{ "cursor", "at", "summary" }, …], "truncated": false }` — the full revision window, ascending; grows as lease-driven revisions append |
| `GET /api/events?last=<cursor>` | SSE (`text/event-stream`): `hello`, replay of every revision with cursor > `last` (each `revision` event's `id:` field is its cursor), then live `revision` and `lease` events; a `: heartbeat` comment every 15 s; one flush per event; client disconnect cancels all server-side subscriptions |
| `POST /api/intents` | Body `{ "id", "kind", "target", "parameters", "cursor" }`; refusals are NORMAL 200 outcomes `{"status":"refused","reason":"authority_denied|invalid|stale_projection","note":"…"}`; acceptances carry `{ "status": "accepted", "note": "…", "lease": { "id", "state", "summary" } }` (lease present only when one was minted) |
| `GET /api/artifacts` | `{ "descriptors": [{ "ref", "kind", "title", "boundObject" }, …] }` — the artifact catalog (fixture seed plus anything persisted at runtime) |
| `POST /api/artifacts` | Body `{ "ref", "boundObject", "title" }` → `{"status":"accepted"}` or the refusal shape; fixture-scoped durability (see limitations) |
| `GET /api/artifacts/{ref}?level=minimal\|summary\|source` | The level's bytes with its `mediaType` (default level `minimal`); unknown ref → 404 typed error, unknown level → 400 typed error |

POST endpoints are strict: `Content-Type` must be `application/json` (else 415), bodies over 1 MiB
are rejected (413), and JSON decoding rejects unknown top-level fields and trailing values (400).
`reason: "unavailable"` is part of the wire vocabulary but no fixture path emits it — the fixture
provider is in-process and cannot be unreachable.

### Consequential flow (fixture demo)

`POST /api/intents` with kind `propose-consequence`, target `ns-migration` (or `ns-secondary`),
parameters `{ "action": "implement", "cursor": <liveCursor> }`: validate → fixture authority
allowlist → lease `claimed` → `active` → append revisions 1151–1153 (the `ns-fix-layer`
compatibility layer appears, candidate checks run, the world quiets) → `released`. Every revision
streams over SSE; the UI world reorganizes from these events alone. Intent ids are idempotent:
the same id with an identical body replays the recorded outcome; the same id with a different body
is refused `invalid`.

### FIXTURE labeling and limitations (stated, not hidden)

* **No governed authority.** The intent decision point is an allowlist over the Northstar fixture.
  `decide-approval` is refused `authority_denied` because the fixture has no review capability;
  `implement` is accepted only for `ns-migration`/`ns-secondary`; every other consequential request
  is refused with ZERO effects (no lease, no revision — tested).
* **In-memory durability.** Persisted artifacts survive for the process lifetime only; a restart
  reseeds from the fixture.
* **Restart clears leases and idempotency records.** Leases and recorded intent outcomes live in
  the coordinator process; a restarted server starts with none (tested).
* **Fixture dataset drift.** The embedded `internal/projection/fixturedata/northstar.world.json`
  must stay byte-equal to the canonical dataset in the cognitive-ui app; a test enforces this and
  skips only when the sibling app is absent from the checkout.
* Real SEA-Forge/Gauntlet wiring is later milestone work; nothing served here is governed
  integration.

## Authentication, sessions and production posture (T07)

The live surface (`internal/server`) is session-gated. Every route except `GET /api/healthz`,
`GET /api/readyz` and the auth entry points requires an authenticated session; unauthenticated
reads are `401 {"error":{"kind":"unauthorized",…}}` and state-changing POSTs without a valid CSRF
token are `403 {"error":{"kind":"csrf_refused",…}}` BEFORE any kernel call. Identity is
session-bound: the pre-T07 `?actor=&role=` override is refused (`400`), and every intent's
client-asserted `actor` is overwritten with the session's kernel standing — the kernel still
verifies each delegation against its own T02 allowlist.

### The `auth` section (serve postures only)

```jsonc
"auth": {
  "mode": "local"           // local | oidc | dev
  , "users": [ { "username": "operator", "display_name": "Ops",
                 "password_hash": "$argon2id$v=19$m=19456,t=2,p=1$…",   // local mode only
                 "actor_id": "operator_local", "role": "operator" } ]   // the T02 kernel standing
  , "oidc": { "issuer": "https://idp…", "client_id": "…",
              "client_secret": "env:GODSPEED_OIDC_SECRET",             // indirection, always
              "redirect_url": "https://host/api/auth/callback",
              "username_claim": "preferred_username",
              "mappings": [ { "claim": "groups", "equals": "operators",
                              "actor_id": "operator_local", "role": "operator" } ] }
  , "insecure_cookie": false        // loopback dev only; REFUSED with production
  , "session_idle_minutes": 30      // sliding; sessions also die at the absolute bound
  , "session_absolute_hours": 12
}
```

* **local** — users from config, argon2id-verified (OWASP parameters m=19456/t=2/p=1, PHC format;
  `auth.HashPassword` mints hashes). Unknown users and wrong passwords are indistinguishable.
* **oidc** — real authorization-code flow (`coreos/go-oidc/v3` + `golang.org/x/oauth2`):
  discovery at startup (an unreachable issuer refuses to start), single-use bounded states,
  nonce-verified ID tokens, and claim→kernel-standing mapping rules (first match wins, NO match
  is a refusal). The mappings are the deployment's trust boundary: a verified identity maps to a
  kernel actor only through an explicit rule.
* **dev** — test/dev ONLY: password verification skipped, plus an optional static bearer token
  (`"static_token": "env:…"` + `"static_token_user"`; `Authorization: Bearer …` authenticates the
  named user per-request, which is why bearer POSTs skip CSRF — bearer credentials are explicit,
  not ambient). The production posture refuses this mode at configuration validation and the
  process exits 2.

Fail-closed defaults: a serve posture with NO `auth` section refuses to start; `serve.production`
refuses `mode: dev` AND `insecure_cookie: true`; a `static_token` in any non-dev mode is refused.

### Sessions, cookies and CSRF

Server-side sessions (`internal/auth.SessionStore`): in-memory, single-process, 256-bit random
ids, bounded sliding idle TTL (default 30 min) + absolute expiry (default 12 h), store bounded at
1024 sessions with a sweep loop. Cookies: `casework_session` is `HttpOnly; SameSite=Strict`
(+`Secure` unless `insecure_cookie`); `casework_csrf` mirrors the session's synchronizer token and
is deliberately readable (the UI echoes it back). CSRF mechanism (documented choice): a
**synchronizer token** minted at session start, presented in `X-CSRF-Token` on every
state-changing POST and compared constant-time; the pre-login POST (no session yet) uses
**double-submit** against the cookie any GET issued. Logout destroys the server-side session.

### Rate limits, health, correlation

`POST /api/intents` passes a token bucket **per session AND per remote IP** (config
`serve.rate_limit`: `intents_per_minute` 60, `burst` 20 by default) — refusal is a typed
`429 rate_limited` and the kernel is never called. `X-Forwarded-For` is deliberately NOT trusted:
behind a proxy the per-IP bucket degrades to per-proxy, and the per-session bucket is the
effective per-user control. `GET /api/healthz` stays cheap (no kernel call); `GET /api/readyz`
performs the kernel's `readiness.get` through the SFWP client (3 s budget) and reflects its
verdict (503 when not ready). Request logs are logfmt with a `correlation_id` per request; for
`POST /api/intents` the correlation id IS the intent id — the `request_id` the kernel correlates
on — so gateway log lines join to the kernel's durable request records.

### Static UI serving and CSP

`serve.static_root` (e.g. `apps/godspeed-cognitive-ui/dist`) serves the built UI: hashed
`assets/*` get `Cache-Control: public, max-age=31536000, immutable`, `index.html` (and the SPA
fallback for unknown non-API GET paths) gets `no-cache`, unknown `/api/*` paths are typed JSON
404s and never fall through to the shell. Documents carry
`Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'self';
frame-ancestors 'none'`. Honesty note: `script-src 'self'` is strict (the current build has no
inline scripts and no eval); `style-src` keeps `unsafe-inline` because React 19's runtime contains
a `<style>`-injection path we cannot yet prove never executes at runtime — tightening it is a
T08/T09 follow-up with runtime verification, not a claimed impossibility.

### Deployment: TLS termination and reverse-proxy guidance

The gateway speaks plain HTTP on `-addr` (default loopback 4179). In production, put it behind a
TLS-terminating reverse proxy:

* Terminate TLS at the proxy; set `serve.production = true` so the gateway enforces `Secure`
  cookies and refuses the dev auth surface. Do NOT expose the raw listener beyond the proxy.
* Keep the proxy and gateway on the same host or a trusted network; the socket to the kernel is a
  Unix socket and carries no auth of its own beyond SO_PEERCRED.
* Same-origin serving is the intended browser topology: either proxy `/api/*` and the static
  assets from ONE origin, or use `serve.static_root` and serve everything from the gateway. The
  loopback-only CORS default is unchanged and cross-origin credential-bearing requests are not
  enabled; if you proxy, preserve the `Origin`-independent behaviour of the API (no cookie-based
  routing between tenants).
* Pass `Connection`/hop-by-hop headers as your proxy defaults dictate; set a generous read
  timeout for `/api/events` (the SSE stream holds connections open with 15 s heartbeats).
* Rate limiting per IP uses the proxy's address (see above); per-session limits are unaffected.

## Gate

`just casework-go-check` (repository root) runs `gofmt -l`, `go vet ./...` and `go test ./...` in this
module. It is plan gate `GATE_GO`.

## Not implemented here (by design)

No live adapters and no governed work acceptance: those are later tasks. Until an adapter is
registered, a capability reports a typed `unavailable` error rather than pretending to be
connected; the serve-mode surface above is fixture-labeled end to end.
