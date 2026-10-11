# T07 session-read test setup repair

## Original specification

Implemented against the bounded repair direction in
[`session-read-builder-spec.md`](session-read-builder-spec.md): preserve the session-bound
historical/SSE assertions and fail-closed legacy refusal; repair only test setup that prevents
those behaviors from being exercised. Production code and security assertions were not changed.

## Root causes and repair

- `testUser.get` constructs requests as `u.base + path`. The three session-view requests had
  already included `h.ts.URL`, producing malformed double-host URLs. The refusal and legacy
  requests used the same invalid form. All five calls now pass relative paths.
- The retention-resync fixture configured relay actor `op`, while the authenticated test user
  resolves to `operator_local`. `fakeWorld.Snapshot` preserves the actor it receives, and
  `renderRevision` correctly refuses a mismatched legacy snapshot. The resync fixture now uses
  the harness operator actor `operator_local`; hello, `resync_required`, and replay assertions for
  `01BBB` and `01CCC` remain intact.
- Added explicit HTTP 200 checks, including response bodies on failure, before decoding the
  positive historical/current reads and consuming the positive SSE response.

## Result and deviations

Only `internal/server/session_read_test.go` and `internal/server/server_test.go` were edited among
Go sources. The historical actor/action checks, stream actor/action checks, override and
cross-case refusals, revoked-perspective failures, and negative legacy perspective test remain
present. No production behavior was changed. No material deviation from the original spec.

## Verification and pending gates

No tests, vet, build, or compilation were run for this repair. The released compile token remains
reserved for the independent T07 critic, who must rerun the focused/global race gates and gateway
build after these setup changes.
