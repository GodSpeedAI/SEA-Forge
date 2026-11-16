# livestack

## Overview
Assembles the full live gateway over one `livetest.Cell`: the same wiring `main.go`'s `serveLive`
performs (SFWP client → authority → live source → relay → intents → HTTP surface, plus the session
layer: dev-mode authenticator, sessions, CSRF, rate limits). It exists so the `-tags live` tests in
other packages exercise the gateway exactly the way production wires it, without import cycles into
the packages under test. Test-support package (imports `testing`); used by live integration tests,
not shipped in the serving binary.

## Interface
- `Stack` — the assembled live stack (client, authority, live source, relay, store, intents handler, `server.Server` options) [stack.go:Stack]
- `AssembleStack(t, cell) *Stack`, `AssembleStackWithRetention`, `AssembleStackObserved` — assembly over a running kernel cell [stack.go:AssembleStack]
- `TestAuthOptions` — dev-mode `server.AuthOptions` used by the stack [stack.go:TestAuthOptions]
- `(*Stack).Login(t, base, username) *Session` — establishes a browser session against the stack's HTTP surface [stack.go:Login]
- `(*Stack).HTTPServer(t) *httptest.Server` — the stack's HTTP server [stack.go:HTTPServer]
- `Session.Get`, `Session.PostJSON` — cookie-carrying HTTP helpers [stack.go:Get] [stack.go:PostJSON]
- `(*Stack).CommitSentryChain`, `CommitTemplate`, `WaitRevision` — governed commit and revision-wait helpers [stack.go:CommitTemplate]
- `NewCell` re-exported from `livetest` [stack.go:NewCell]

## Invariants
- Wiring mirrors `main.go`'s `serveLive` exactly so live tests exercise the production topology [stack.go:package doc]
- Sessions, CSRF and rate limiting are active on the assembled surface (dev-mode authenticator) [stack.go:package doc]

## Side Effects
- Starts an `httptest` HTTP server and gateway goroutines (relay, session sweep) [stack.go:HTTPServer]
- Makes live kernel calls over the cell's socket (commits, event subscriptions) [stack.go:CommitTemplate]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
