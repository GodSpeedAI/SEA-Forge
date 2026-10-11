# T11 security review: casework gateway and identity delegation

Scope: apps/godspeed-casework-go, T02 identity delegation (sea-forge-server), recent kernel changes
(sea-forge-ledger types.rs, continuation codec/key), deploy/systemd, cell backup/restore scripts,
apps/godspeed-cognitive-ui. HEAD 7446ccf plus uncommitted review changes. No open high findings.
A first reviewer was killed by a rate limit; its partial work (CW-44-class items below marked
"prior") was inspected, found correct, built and kept.

## Findings

| # | Sev | Where | Attack | Status |
|---|-----|-------|--------|--------|
| F1 | high | cmd/godspeed-casework/main.go + config/auth.go `ValidateBindPosture` | `serve.production` defaults false; a production config omitting it ran dev auth (any password) on a reachable interface | FIXED (prior, verified): non-loopback bind refused unless production=true. Test TestValidateBindPostureRefusesReachableBindWithoutProduction |
| F2 | medium | server/session.go handleLogin | online password guessing, argon2id memory exhaustion by login flood, oversized credentials | FIXED (prior, verified): failed-login throttle per IP and per username, 8 concurrent verifications, 1 KiB field cap. Tests TestT11LoginBruteForceIsThrottled, TestT11LoginRefusesOversizedCredentials |
| F3 | medium | server/server.go handleEvents | one session opens unbounded SSE streams (goroutines/fds) | FIXED (prior, verified): 8 per session. TestT11SSEPerSessionCap |
| F4 | medium | cmd main.go, server/http.go | slowloris on headers/body; no idle timeout; unbounded headers | FIXED (prior, verified): IdleTimeout 2m, MaxHeaderBytes 64K, 30s body deadline; no Read/WriteTimeout (would cut SSE). TestNewHTTPServerCarriesSlowlorisBounds |
| F5 | medium | auth/oidc.go LoginURL, Callback | GET /api/auth/login is unauthenticated and stored a state per hit with no cap (memory growth); id_token WITHOUT a nonce was accepted (only a wrong nonce was refused), enabling token replay | FIXED: pending-state cap 4096 (refuse, not evict), per-IP login-start limiter, nonce required and compared constant time. TestT11OIDCMissingNonceIsRefused, TestT11OIDCPendingStatesAreCapped (both fail with the fix reverted) |
| F6 | medium | server/session.go handleReadyz, oidc callback, login | unauthenticated /api/readyz echoed the raw kernel error (socket paths); OIDC callback echoed IdP error text | FIXED: generic bodies, detail to the server log. TestT11ReadyzDoesNotLeakKernelErrorText (fails reverted) |
| F7 | medium | intents/intents.go | free-text reopen/terminate reason (up to 1 MiB) written to kernel event ledger and trace; intent_id unbounded and echoed to a response header, logs and request_id; replay map `outcomes` grew without bound | FIXED: reason max 2000 bytes, valid UTF-8, no control chars; intent_id 1-128 printable ASCII; replay cache bounded FIFO 20000. TestT11IntentInputBounds (pre-fix it does not compile: new symbols; behaviour is otherwise new validation) |
| F8 | medium | sea-forge-ledger types.rs materialize_aggregate_view | view temp name is predictable (pid + counter) and was opened with fs::write (follows symlinks): a local writer in the view dir redirects the write onto another file | FIXED: create_new, stale leftover unlinked then retried. view_temp_never_writes_through_a_planted_symlink (fails reverted). Requires write access to the cell dir, so it is local-only |
| F9 | low | scripts/casework-cell-restore.sh | the .sha256/.manifest only detect corruption; a forged archive with symlink/absolute/.. members | HARDENED (defense in depth): members must be regular files or directories with no absolute or .. names, checked before extraction. NOT reproducible: GNU tar already refused the symlink-escape payload in my local test (old script exited with a tar error, nothing written outside). No automated test added |
| F10 | low | login throttle design | per-username bucket lets anyone lock a user out for the refill window; behind a TLS proxy the per-IP bucket is shared by all users | ACCEPTED/DEBT CW-47 |
| F11 | low | server.go world/templates/preflight handlers | authenticated 502 bodies include kernel err.Error() | DEBT CW-48 |
| F12 | low | auth/oidc.go | login CSRF: the OIDC state is not bound to the initiating browser (no state cookie), no PKCE | DEBT CW-49 (protocol/UX change; nonce and single-use state mitigate token replay) |
| F13 | low | server logging.go | first 12 hex chars of the session id are logged | DEBT CW-50 (info) |
| F14 | low | API responses | no `Cache-Control: no-store` on authenticated JSON; no Referrer-Policy/X-Frame-Options on API (CSP frame-ancestors covers documents) | DEBT CW-50 |
| F15 | low | kernel reopen/terminate | kernel itself does not bound `reason` (only the gateway now does); direct SFWP clients under a bound uid can write large reasons | DEBT CW-51 |
| F16 | low | deploy/systemd/sea-forge-server.service | lighter hardening than the gateway unit (no SystemCallFilter, address families include INET) | DEBT CW-52 |
| F17 | info | identity delegation | live single-machine topology: one uid holds the gateway binding and all end-user bindings, so any process under that uid can claim any bound actor directly. Inherent to uid-based identity; the ADR records it | no change |
| F18 | info | approval lifetime | `execution_timeout_sec` is operator config only (1..7 days, kernel clamps); not settable from a request | no change |
| F19 | info | continuation codec/key | codec is unwired dead code; Ed25519 verification (no hand-rolled compare), 4 KiB token and 2 KiB payload caps, canonical-signature check; key is a fresh zeroized per-process seed from getrandom, entropy failure aborts startup | no finding |
| F20 | info | ledger shared lock | shared-lock readers, never taken inside the exclusive lock; lock file is created by the exclusive path; signing key files 0600; no deadlock path found | no finding |
| F21 | info | UI | no dangerouslySetInnerHTML/innerHTML/iframe/srcdoc/eval; markdown and artifact text render as React text nodes; no token in web storage (CSRF from cookie); `?focus=` etc only dispatch store actions, no navigation/redirect; CSP script-src 'self' (style-src keeps unsafe-inline, documented) | no finding |

Verified-sound areas (no finding): session ids and CSRF tokens 256-bit CSPRNG; HttpOnly, Secure(prod),
SameSite=Strict cookies; new session id on every login; logout destroys server-side; idle + absolute
expiry; session store bounded (1024, oldest evicted); CSRF synchronizer on all session POSTs, login
double-submit; Origin allowlist enforced on POST (single header, exact match); intent actor
overwritten from the session; kernel re-verifies the session perspective; replay keyed by
intent_id + body hash (includes actor); artifact endpoint: strict sha256 pattern, digest and size
re-verified, UTF-8 only, content type from a closed set, JSON string body (no sniff, nosniff set);
static handler uses path.Clean + http.ServeFile; metrics listener loopback-only and not on the
public mux, route-pattern labels (bounded cardinality); socket published 0600 via staging rename;
delegation requires gateway uid (SO_PEERCRED) + gateway actor + allowlist (supervisor and gateway
excluded at config load) + role narrowing, audit record fail-closed.

## Verification
See the task's final report for the commands and results.
