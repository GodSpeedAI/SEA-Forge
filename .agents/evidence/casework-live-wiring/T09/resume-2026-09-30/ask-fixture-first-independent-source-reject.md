# Ask fixture first independent source review — REJECT

Date: 2026-10-01

## Verdict

REJECT source completeness pending a fresh builder correction. The assigned fixture files are
frozen at the hashes below. No compiler or tests were run; root retains the compiler token.
Expected-RED verification remains pending that correction and explicit root transfer.

## Original assignment and inspected result

Reviewed the original `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/ask-test-first-assignment.md`,
`runtime-integration-decisions.md`, and `contract-source-preparation.md`, against the frozen
four-file diff. The assigned scope is honored: the only source additions are the semantic
application-owned Ask port, `Options.Ask`, and the two test fixtures. There is no route, cap
implementation, SFWP behavior, Rust/identity change, generated schema, golden, or dependency
change in this diff.

## Blocking finding

`apps/godspeed-casework-go/internal/server/ask_test.go:214-238` covers null, array, and string
roots, but omits numeric and boolean scalar roots. The original assignment explicitly requires
"array/scalar roots". A test that rejects a string scalar does not establish rejection for the
other JSON scalar types. Add cases for a numeric root and a boolean root to the same no-dispatch
malformed-input matrix. This is the only source completeness finding that blocks approval.

## Material deviations reviewed

- `postAskFrom` in `ask_test.go` constructs requests with `httptest` and calls the actual handler
  directly, supplying the authenticated session cookies and setting `RemoteAddr`. This is a
  justified test-harness choice: the standard HTTP client controls the outgoing socket address,
  so it cannot exercise the limiter against arbitrary actual remote addresses. The fixture still
  uses real issued cookie values and the actual handler stack.
- `ask_transport_test.go` uses `newRequest("ask")` and constructs the exact body fields directly
  instead of implementing `NewAsk`. This matches the original assignment's explicit allowance.
  It verifies the request encoder's verb, effective `actor_id`, gateway/on-behalf-of pair, and
  absence of `request_id`; it does not claim to test the not-yet-implemented adapter mapping.
- `ports/ask.go` contains semantic Go fields without JSON tags, as required by
  `runtime-integration-decisions.md`. `AskQuestion`, `AskClaim`, and `AskAnswer` are application
  models; no SFWP wire types or untyped fields were introduced.

Other assigned coverage is present by source inspection: auth/CSRF refusal before port calls;
separate Ask session/IP accounting and intent-limiter isolation; actual remote address and
forwarded-header spoof coverage; refill behavior; all nine kinds, omitted-purpose default and
explicit empty preservation; trimmed nonempty subject/case with valid supplied values preserved;
purpose UTF-8 byte boundaries; exact 8 KiB and over-limit body cases; unknown/null/wrong-type,
malformed, trailing JSON cases; verified actor and refusal; answer/partial/denied disclosure
field comparisons; and transport EOF/timeout/overflow no-retry plus explicit busy retry checks.
These remain source claims until independent expected-RED and later required runtime evidence.

## Frozen source hashes

```text
09c6af57138e52fe0e5dfe504b08a6a4517fa8d65955a7cb508e463f884de6b6  apps/godspeed-casework-go/internal/ports/ask.go
ebe23204a2390e4611c77d594e82cd6dfcb0a32343c1113e4900d11cfc5cf25c  apps/godspeed-casework-go/internal/server/http.go
e1e5c2aef5da63295fb903a0a2024efd6f04c81e85e57e952ca3c3c9a8e04879  apps/godspeed-casework-go/internal/server/ask_test.go
94764ad34ca808bfa92ed96bfd91acd23fb8767c5fa9645515e57f91395426cc  apps/godspeed-casework-go/internal/adapters/sfwp/ask_transport_test.go
```
