# T07 independent confirmation — round 1

**Verdict: REJECT.** The gateway implementation has the core authentication and production posture, but it does not implement T07's explicit trusted-origin configuration requirement. This review is independent of the T07 builder. The rejection is based on the final code at HEAD `3e2404f` and the unchanged T07 implementation commit `eef7459`.

## Blocking finding

**F-1 — trusted origins cannot be configured.** T07 requires “the listener address, TLS termination (or reverse-proxy guidance) and trusted origins configurable” while retaining a loopback-only CORS default. The serve config has production/static/rate-limit fields but no trusted-origin field ([config.go:67-88](../../../../../apps/godspeed-casework-go/internal/config/config.go:67)). CORS is instead hardcoded to `http://localhost`, `127.0.0.1`, and `[::1]` ([http.go:55-77](../../../../../apps/godspeed-casework-go/internal/server/http.go:55)); the deployment documentation confirms that cross-origin credentialed requests are disabled and only same-origin serving is supported ([README.md:211-227](../../../../../apps/godspeed-casework-go/README.md:211)). This means an operator cannot configure a trusted browser origin for a deployment that requires cross-origin UI/API hosting. The fixed loopback-only policy is a safe default, but it does not satisfy the planned configurability. No evidence shows an approved scope change removing this requirement.

**F-2 — request logging permits log injection.** The middleware logs the decoded request path directly into a logfmt line ([logging.go:98-100](../../../../../apps/godspeed-casework-go/internal/server/logging.go:98)); there is no quoting or escaping. Since the `correlation_id` for `/api/intents` is also copied from the client-provided `intent_id` ([server.go:213-221](../../../../../apps/godspeed-casework-go/internal/server/server.go:213)), and the intent handler rejects only an empty ID ([intents.go:118-125](../../../../../apps/godspeed-casework-go/internal/intents/intents.go:118)), an authenticated client can include a newline in the JSON string and inject a second apparent log record. The path is independently attacker-controlled and logged before authentication, so encoded newlines in a request path may also forge records without a session. This undermines the requested structured correlation logs and can corrupt downstream log parsing. Escaping logfmt fields or switching to a structured encoder is needed before approval.

## Review performed

The following T07 surfaces are present in code and were reviewed:

- Local Argon2id and OIDC authenticators behind the authenticator interfaces; dev auth and insecure cookies are rejected in production config.
- Session identity is required for protected reads and writes. CSRF middleware wraps intent/preflight/logout POSTs, and the intent handler overwrites its asserted actor from the authenticated session. `/api/world` rejects the old `?actor=&role=` override.
- Session cookies are `HttpOnly`, `SameSite=Strict`, and `Secure` when configured for production. The CSRF token is a synchronizer token.
- Intent requests are limited by session and remote IP. Health is separate from readiness, and readiness invokes the configured kernel readiness port.
- Static UI routes include an SPA fallback, CSP, `no-cache` on the shell, and immutable asset caching. CSP retains `style-src 'unsafe-inline'`; the implementation documents the React runtime rationale and does not claim a strict style policy.
- Request logging is wired in live assembly. The intent ID becomes the correlation value and the SFWP request ID. F-2 records that log fields are not escaped.

## Commands observed

- `GOMAXPROCS=2 GOFLAGS=-p=1 go vet ./...` — EXIT 0.
- `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./internal/server/... ./internal/auth/...` — EXIT 0.
- `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./...` — EXIT 0.
- `GOMAXPROCS=2 GOFLAGS=-p=1 go build -o /tmp/casework-resume-gateway ./cmd/godspeed-casework` — EXIT 0.

Fresh logs are under `gates/`. A fresh `-tags live` runtime rerun of the CSRF, session, and actor-overwrite teeth remains pending the shared Go compile slot; it cannot change F-1 or this verdict. No Cargo or UI gates were run in this review; the parent is coordinating those shared gates.

No product code, status files, or prior evidence were modified. No credentials or secret values were copied into this record.
