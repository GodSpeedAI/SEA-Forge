# Ask fixture independent review

Date: 2026-10-01

## Verdict and scope

APPROVED for the bounded Ask fixture assignment after the fresh builder added number and boolean
root cases. This approval covers fixture and minimal declaration completeness only. It does not
approve runtime implementation or settle T09. The original assignment expressly excludes route,
cap implementation, and transport behavior from this unit.

Reviewed the original
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/ask-test-first-assignment.md`,
`runtime-integration-decisions.md`, and `contract-source-preparation.md`, plus the complete frozen
four-file source set and its diff. The earlier source rejection is preserved in
`ask-fixture-first-independent-source-reject.md`; this review does not overwrite it.

## Source coverage

The initial rejection was specifically the missing numeric and boolean scalar roots. The fresh
builder added `7` and `true` in `apps/godspeed-casework-go/internal/server/ask_test.go:223-224`;
the updated source hash is recorded below. No other changes were made by the fresh builder.

Source review confirmed the assigned cases: unauthenticated request and CSRF refusals before
AskPort calls; separate Ask session/IP limits and intent limiter isolation; actual `RemoteAddr`
with spoofed forwarded headers; refill; all nine kinds; omitted purpose default versus explicit
empty preservation; preserved valid subject/case values and blank-value rejection; UTF-8 purpose
limits; exact 8 KiB and over-limit bodies; unknown fields, null/wrong types, all JSON root kinds,
malformed/trailing input, and zero dispatch on invalid input; verified session actor and refused
perspective; answered/partial/denied output preserving claim refs and disclosure metadata; and
uncertain transport outcomes versus explicit busy retry.

## Deviations and limits

- The server fixture uses `httptest` request/response objects and calls the actual handler
  directly, with real issued session cookies, rather than an HTTP client. It sets `RemoteAddr`
  directly because an HTTP client cannot choose the peer address used by the rate limiter. This
  is a justified way to verify actual-IP accounting and the forwarded-header spoof case.
- The transport fixture constructs `newRequest("ask")` and its exact fields directly instead of
  implementing `NewAsk`. The original assignment explicitly permits this. It proves request
  encoding of the actual `ask` verb, no `request_id`, `actor_id`, and gateway/on-behalf-of pair.
  **It does not prove a future configured Ask adapter maps the verified actor and semantic
  question into that request.** Runtime integration must add or run evidence against the real
  adapter mapping before any broader approval.
- The application port models use semantic Go fields and have no JSON tags, matching the runtime
  integration clarification. No SFWP type or untyped application field was introduced.
- The server refill fixture intentionally waits roughly ten seconds to prove the six-per-minute
  refill boundary. This is slower than a fake-clock unit but tests the real limiter wired into the
  harness; the assignment specifically requires refill evidence.

No other material deviation from the assigned scope was found. No dependency, route, transport
implementation, Rust/identity behavior, schema, or golden file changed.

## Independent expected-RED evidence

Before each Go command, an escalated actual-host preflight ran `free -b` and scanned active
`go test`, `cgo`, `gcc`, `golang`, Bun test, Cargo test/check/build, and `rustc` processes. The
first preflight reported 2,752,765,376 bytes available; the second reported 2,459,144,192 bytes
available. Neither scan found another compiler/test process.

The server and SFWP transport commands ran separately, serialized, with host access and:
`GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0`, owned `/tmp` Go cache
and temp directories, `-race -count=1 -parallel=1`.

1. `go test -race -count=1 -parallel=1 ./internal/server -run '^TestAsk'` exited 1 only after
   the Ask cases reached the absent-route `405 Method Not Allowed` assertions. There were no
   compile/import failures or harness failures in the escalated run. Since the route is absent,
   these `405`s do not prove authentication or later handler behavior. Full captured output is
   `ask-fixture-server-expected-red.log` (5,968 bytes), exact SHA-256
   `17dcc84e7175540f519767e1f4ddb0227dc4df0feb2a0dbba6c96de6d38e87f2`; actual exit status is
   in `ask-fixture-server-expected-red.exit`.
2. `go test -race -count=1 -parallel=1 ./internal/adapters/sfwp -run
   '^(TestAskWireHasProtectedActorPairAndNoCorrelationID|TestUncertainAskEOFIsUnavailableWithoutResendOrStatusLookup|TestUncertainAskTimeoutIsUnavailableWithoutResendOrStatusLookup|TestUncertainAskResponseOverflowIsUnavailableWithoutResendOrStatusLookup|TestExplicitServerBusyAskGetsOneSeparateSafeRetry)$'`
   exited 1 on the intended unsafe current-client behavior: EOF and response overflow each sent
   Ask twice instead of once. The timeout test observed one send on this path in the current
   client, but this single baseline case does not prove a general Ask safety guarantee. The frame
   check passes only for its manually constructed `Request`; it does not exercise adapter
   translation. Explicit busy retry also passed in this baseline. Full captured output is
   `ask-fixture-transport-expected-red.log` (434 bytes),
   exact SHA-256 `4b39ace6ddb5746ac7bb498b66b058dd7dbe6ae897984bfff0cb1e0bedac49d1`; actual exit
   status is in `ask-fixture-transport-expected-red.exit`.

The earlier non-escalated server attempt failed before tests at shared `httptest.NewServer` setup
because sandbox socket permission was denied. It is retained separately as
`ask-fixture-server-sandbox-socket-failure.log` and is not counted as expected-RED evidence.

## Frozen source hashes

```text
09c6af57138e52fe0e5dfe504b08a6a4517fa8d65955a7cb508e463f884de6b6  apps/godspeed-casework-go/internal/ports/ask.go
ebe23204a2390e4611c77d594e82cd6dfcb0a32343c1113e4900d11cfc5cf25c  apps/godspeed-casework-go/internal/server/http.go
90612d084a10fce6ec4871faae563790c7de5267d2ef60369bc1fc23bd538cbe  apps/godspeed-casework-go/internal/server/ask_test.go
94764ad34ca808bfa92ed96bfd91acd23fb8767c5fa9645515e57f91395426cc  apps/godspeed-casework-go/internal/adapters/sfwp/ask_transport_test.go
```
