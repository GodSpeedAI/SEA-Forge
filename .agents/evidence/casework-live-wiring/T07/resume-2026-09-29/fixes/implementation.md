# T07 critic fixes

Implementation record for independent rejection F-1/F-2.

## Changes

- Added `serve.trusted_origins` as an exact allowlist. Validation rejects wildcard hosts, userinfo,
  paths, queries, fragments, malformed values, duplicates, non-loopback HTTP in development, and
  all HTTP origins in production.
- Wired configured origins through the live server. Empty config keeps the existing loopback-only
  development allowlist. Configured origins receive credentialed CORS; supplied untrusted or
  duplicate Origin headers on POSTs are refused before route handling. Existing authentication,
  session identity, CSRF token, actor mapping, and authority checks remain in the request path.
- Escaped every string-valued request log field as a quoted logfmt value. Correlation IDs still use
  the original intent ID sent as SFWP `request_id`.
- Documented exact-origin setup and the `SameSite=Strict` constraint: same-site subdomains can use
  browser sessions, while cross-site cookies may be withheld by the browser.

## Verification

- Focused T07 regression/auth gate: `gates/01-focused-test-race.log` — EXIT 0.
- Go vet: `gates/02-go-vet.log` — EXIT 0.
- Full Go workspace race gate: `gates/03-global-go-race.log` — EXIT 0.
- `gofmt -w` was applied to the T07 files and shared server/main/artifact files; `git diff --check`
  passed. The final `gofmt -l` audit found only the T08 trajectory owner's
  `internal/projection/store_test.go` still unformatted; its owner was notified and will format it
  after this compile slot is released.
- Initial sandboxed focused test attempt could not access the read-only Go build cache and could not
  bind `httptest` sockets. The same focused command passed through the parent-authorized local
  execution path. No sandbox was weakened.

The original independent rejection remains preserved at the sibling `confirmation.md` path; this
record addresses only its F-1 and F-2 findings. Independent re-confirmation remains outstanding.
