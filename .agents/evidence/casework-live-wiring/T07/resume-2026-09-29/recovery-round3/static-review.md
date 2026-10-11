# T07 recovery-round3 static review (provisional)

This is a static review of the fresh `session-read-fix` implementation against the original bounded repair specification and T07 plan. It is not an approval or settlement. Per compile-slot coordination, no Go test, vet, build, or fresh-cell probe has been run in this review. Fresh-cell and gate evidence remains required before a verdict.

## Material implementation differences from the repair specification

- `projection.Revision` carries captured `CaseFacts` behind `json:"-"`; `Store.Append`, `At`, `Live`, `Trajectory`, replay, and live subscriber delivery deep-copy the captured facts and snapshot while retaining the existing count-based revision bound.
- `Relay.accept` uses the source's optional `Facts` method and builds the retained snapshot from that same capture; legacy sources continue through `Snapshot` without retained facts. `LiveSource.Facts` fixes its timestamp at capture so historical re-render keeps the original time.
- `handleWorld` rejects the presence of `actor` or `role` query keys before verification or cursor lookup, verifies the authenticated session through `PerspectiveVerifier`, binds an optionally supplied `case_id` to the cursor's case, and re-renders retained facts for that actor. A legacy revision is served only when its stored actor and role exactly match.
- `handleEvents` rejects identity query keys and verifies the session before stream setup. Each replayed/live revision is re-rendered from its captured facts for the authenticated actor; a legacy revision with a different perspective ends the stream without sending the mismatched snapshot.
- No wire-contract field, dependency, persisted schema, kernel policy, or identity-model change was found in the T07 repair hunks. `server.go` also contains concurrent T08 artifact/trajectory wiring; that is outside this T07 review.

## Static coverage observed

- The new `session_read_test.go` covers a single facts capture for snapshot plus retention; a later enabled revision while reading a prior pending cursor; R-SO and operator historical/SSE perspectives and offers; query-override refusal; cross-case cursor refusal; mocked revoked-delegation refusal on cached reads and SSE; and mismatched-perspective fallback refusal.
- `store_test.go` covers deep copies of nested authority DTO slices across append/read.
- Static inspection of the current DTO definitions found the clone helpers cover all present nested slice/pointer fields in `CaseFacts` and `CognitiveWorldSnapshot`.
- These tests are source-only and explicitly unrun; they do not satisfy the required fresh-cell probes or gates.

## Documentation omissions

- `internal/server/http.go`'s `Options.Perspective` comment still says SSE snapshots stay at one configured relay perspective and directs clients to refetch `/api/world`; the new code instead re-renders SSE for each verified session. Update this comment to match behavior.
- `internal/server/server.go`'s `PerspectiveVerifier` comment still describes verification of an explicit `?actor=&role=` override; the verifier now serves authenticated session perspectives and the override is refused. Update this comment to describe its actual contract.

The core repair appears statically aligned with the spec; the two comments above are documentation drift, not an observed runtime defect. Final verdict waits on a fresh-cell operator/R-SO stream/history exercise (including genuine old state after mutation), security teeth, focused/global gates, and the root's explicit compile-slot grant after Go source is stable.
