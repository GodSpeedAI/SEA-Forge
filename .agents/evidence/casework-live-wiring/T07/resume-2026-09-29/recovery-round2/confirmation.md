# T07 independent confirmation — recovery round 2

**Verdict: REJECT.** The builder fixed round-one findings F-1 (trusted origins) and F-2 (log injection), and the fresh Go gates pass. This review found two additional authenticated-read perspective defects that make the protected world endpoints unsafe for multiple mapped actors. No product files were changed by this critic.

## Round-one fixes reviewed

- `serve.trusted_origins` is now parsed, validated as an exact origin allowlist, and passed through live assembly. The default remains loopback-only; production rejects HTTP origins. Configured CORS is credentialed and exact-match, while POSTs with untrusted or duplicate Origin fields are refused before the route. See `internal/config/auth.go`, `internal/config/config.go`, `internal/server/http.go`, `cmd/godspeed-casework/main.go`, and `internal/server/origin_security_test.go`.
- `logLine` now applies `strconv.Quote` to all string-valued fields (correlation ID, method, path, remote, auth and actor). `TestRequestLogEscapesClientControlledFields` checks newline, quote and identity escaping.
- The README now documents production origin configuration and the `SameSite=Strict` cross-site cookie constraint.

These changes match the omitted T07 requirements and address both round-one findings. The origin config has acceptance/rejection coverage (canonical HTTPS production origins, loopback HTTP development origins, wildcard/userinfo/path/query/fragment/unsupported scheme/whitespace/malformed port/duplicate refusals); the request-log regression checks exactly one physical output line.

## Blocking findings

**F-3 — `/api/events` sends a different actor's role-filtered snapshots to every authenticated session.** `serveLive` constructs the relay with the single configured `serve.perspective_actor_id/role` (`cmd/godspeed-casework/main.go:238-240`). `projection.Build` writes that actor into `snapshot.Perspective` and derives each action offer from `facts.Actor.Role` (`internal/projection/builder.go:54-57, 95-104, 107-122, 208-218`). `handleEvents` writes the stored `rev.Snapshot` without rebuilding or checking its perspective against the session (`internal/server/server.go:269-285, 323-333`). The UI consumes each snapshot event as application history (`apps/godspeed-cognitive-ui/src/app/live.ts:104-110`). The checked-in live config sets the relay perspective to `operator_local/operator`, while it also configures an `rso_local/R-SO` user. Thus an R-SO session receives operator-perspective action descriptors over the protected SSE stream. Intent-time role enforcement prevents those descriptors from authorizing a kernel mutation, but it does not make the exposed snapshot or displayed authority offers truthful for that session. The code comment in `server.Options` says SSE keeps the configured perspective and clients should refetch `/api/world`; the live UI does consume SSE snapshots, so that comment does not establish a safe boundary.

**F-4 — cursor history bypasses actor-override refusal and kernel perspective verification.** `handleWorld` checks `?cursor=` first and immediately returns the retained relay snapshot (`internal/server/server.go:172-181`). The `?actor=`/`?role=` refusal and effective session actor extraction happen after that branch (`:183-188`); the current-world branch then invokes `s.world.Snapshot` with the session claim (`:202-205`), whose live source verifies that perspective. Consequently, any authenticated session can request a retained cursor and receive the configured relay actor's cached snapshot; adding `?actor=` or `?role=` does not reach the intended refusal. This also bypasses the kernel verification performed for current snapshots. The newer trajectory handler verifies the session perspective before serving history, but the world cursor path does not.

Both findings are reproducible directly from the checked-in live wiring and handler order. The fresh-cell live session test proves distinct sessions are supported for current `/api/world`, but no existing live test covers role-specific SSE events or `?cursor=` history. Required repair must keep retained cursor data tied to its original cursor (do not refetch current state and label it historical), build/filter action offers for the requesting session, refuse identity overrides before cursor handling, and verify the requesting session's kernel perspective for cached reads and SSE access without bypassing kernel authority. Re-review those behaviors after a fresh builder fix.

## Fresh verification

- `GOMAXPROCS=2 GOFLAGS=-p=1 go vet ./...` — EXIT 0: `gates/01-go-vet.log`.
- `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./internal/server/... ./internal/auth/...` — EXIT 0: `gates/02-focused-go-race.log`.
- `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./...` — EXIT 0: `gates/03-go-global-race.log`.
- `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 -tags live ./internal/server -run 'TestLiveTeeth(AValidCookieWithoutCSRFReachesNoKernel|DSessionCannotActAsAnotherActor)$'` — EXIT 0 against fresh temporary kernel cells: `teeth/01-live-csrf-actor.log`.
- `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 -tags live ./internal/server -run 'TestLiveWorldPerspectiveComesFromTheSession$'` — EXIT 0 against a fresh temporary kernel cell: `teeth/02-live-session-perspective.log`.
- `git diff --check -- apps/godspeed-casework-go` — EXIT 0.

The focused and global Go suites cover the production dev-auth refusal, configured trusted-origin validation and enforcement, log escaping, session cookie/CSRF flow, rate limiting, and health/readiness behavior. The tagged live teeth prove the CSRF refusal causes no kernel request record, forged actor claims do not replace session actor attribution, and current world requests resolve distinct session perspectives. They do not cover the new F-3/F-4 integration paths; those remain blocking despite the green gates.

The earlier round-one rejection at `../confirmation.md` is preserved unchanged. Round-two interrupted gate logs under `../round2/gates/` are also preserved; this confirmation cites only commands re-run here with observed exit status.
