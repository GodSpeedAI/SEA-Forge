# auth

## Overview
The gateway's browser-authentication layer: the pluggable `Authenticator` front door (local user
store and OIDC authorization-code flow), argon2id password hashing per OWASP's recommended
parameters, the in-memory server-side session store, and the browser-user to kernel-actor mapping.
A session's kernel standing is fixed at login; the gateway overwrites any client-asserted actor
with the session's own — the kernel re-verifies every delegation against its own allowlist.

## Interface
- `Identity` — authenticated identity carrying the kernel actor/role standing; `Claim() ports.ActorClaim` [auth.go:Identity]
- `Authenticator` interface — `Authenticate(ctx, username, password)`; `PasswordLogin` and `OIDCFlow` refine it [auth.go:Authenticator]
- `OIDC` — real authorization-code flow (`coreos/go-oidc/v3` + `golang.org/x/oauth2`): `NewOIDC`, `Mode`, `LoginURL`, `Callback`; `OIDCOptions`, `OIDCMapping` [oidc.go:OIDC]
- `MapClaimsToIdentity` — applies claim→kernel-standing mapping rules (first match wins; no match is a refusal) [oidc.go:MapClaimsToIdentity]
- `HashPassword`, `VerifyPassword` — argon2id PHC hashing/verification (`ErrHashFormat` on malformed PHC) [argon2.go:HashPassword] [argon2.go:VerifyPassword]
- `SessionStore` — bounded in-memory sessions: `NewSessionStore`, `Start`, `Resolve`, `Current`, `Destroy`, `Len`, `Sweep`; `Session.Idle` [session.go:SessionStore]
- `CurrentSessionState` — current session view [session.go:CurrentSessionState]
- `normalizeUsername` — username normalization for identity matching [auth.go:normalizeUsername]

## Invariants
- Identity is never accepted from the client; the session's kernel standing is fixed at login [auth.go:package doc]
- Sessions are in-memory and single-process by design; bounded store with sliding idle TTL + absolute expiry [session.go:SessionStore]
- OIDC login states are single-use and bounded (`MaxPendingOIDCStates` 4096, `DefaultStateTTL` 10 min) [oidc.go:MaxPendingOIDCStates] [oidc.go:DefaultStateTTL]
- Claim mapping: first matching rule wins; NO match is a refusal — a verified identity maps to a kernel actor only through an explicit rule [oidc.go:MapClaimsToIdentity]
- Unknown users and wrong passwords are indistinguishable at verification time [auth.go:package doc]

## Side Effects
- Derives argon2id hashes (CPU work) and verifies PHC strings [argon2.go:HashPassword]
- OIDC performs discovery HTTP calls at construction and token exchange on callback; mints random state/nonce values [oidc.go:NewOIDC] [oidc.go:Callback]
- `SessionStore` mutates in-process session memory and generates random session ids [session.go:Start]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
