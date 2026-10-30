# godspeed-casework (command)

## Overview
The casework front end's entrypoint. Without `-serve` it does one thing end to end: load its single
configuration authority, resolve secret indirections, run preflight, and report each configured
capability's state with a typed error — it deliberately does NOT start any transport or accept work
in that mode. With `-serve` it runs the same preflight first, then serves the live cognitive
projection over HTTP+SSE: snapshots built from the governed kernel through the SFWP client, intents
translated one-to-one onto governed kernel verbs with the effective actor sent as `on_behalf_of`,
and kernel event frames relayed as SSE revisions keyed by the kernel's own event cursor. The dev/demo
fixture stack exists only behind `-tags casework_fixture`.

## Interface
- `main()` — flag parsing (`-config`/`$GODSPEED_CONFIG`, `-serve`, `-addr` default `127.0.0.1:4179`, `-metrics-addr`/`$GODSPEED_METRICS_ADDR`), config load, preflight, serve or report [main.go:main]
- `exitBlocked` = 2 — distinct exit code when a REQUIRED capability is unusable (also fatal config, non-loopback bind, metrics addr) [main.go:exitBlocked]
- `serveLive(ctx, addr, metricsAddr, registry, resolved, authority, client, secrets)` — assembles the live stack: session sweep, authenticator, relay, intents handler, `server.NewWithArtifacts`, metrics listener [main.go:serveLive]
- `newHTTPServer(addr, handler)` — HTTP server with slowloris bounds (read timeouts) [main.go:newHTTPServer]
- `sweepSessions(ctx, sessions, interval)` — periodic session sweeper goroutine [main.go:sweepSessions]
- `buildAuthenticator(ctx, authCfg, secrets)` — builds the `server.AuthOptions` authenticator per auth mode (local/oidc/dev) [main.go:buildAuthenticator]
- `liveAuthorityForServe` / `adapterOf` — select the live authority capability for serve mode [main.go:liveAuthorityForServe]
- `report(err)` — typed error reporting to stderr [main.go:report]
- Fixture build gate: `fixtureStackAvailable` is `false` in the default build (`main_fixture_off.go` refuses to serve fixtures) and `true` under `-tags casework_fixture` (`main_fixture.go` serves the fixture stack) [main_fixture_off.go:fixtureStackAvailable] [main_fixture.go:fixtureStackAvailable]

## Invariants
- Process exit is 2 whenever startup is blocked: fatal configuration, a required capability unusable after preflight, a non-loopback bind without `serve.production`, or an invalid metrics address [main.go:exitBlocked]
- Live serve requires an authority capability with `adapter: "sfwp"`; preflight probes the REAL kernel (`system.hello`, `readiness.get`, `identity.get`) and the fixture provider never stands in for it [main.go:package doc]
- Secret values are held outside `resolved` and passed only to adapters that need them [main.go:main]
- The default (production) build refuses to serve the fixture stack; only `-tags casework_fixture` builds can [main_fixture_off.go:fixtureStackAvailable]

## Side Effects
- Binds the loopback HTTP listener (and the separate loopback metrics listener when enabled) [main.go:serveLive]
- Opens the kernel's Unix socket through `sfwp.New` for every capability selecting `adapter: "sfwp"` [main.go:main]
- Prints the configuration summary line (cell root, evidence root, capability and secret counts — never secret values) [main.go:main]
- Runs the periodic session sweep goroutine and blocks on SIGINT/SIGTERM with graceful shutdown [main.go:sweepSessions]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
