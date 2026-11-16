# projection

## Overview
The cognitive projection: pure builders and stores that turn governed authority facts into canonical
spec-04 `contract.CognitiveWorldSnapshot` revisions, plus the fixture-side Northstar dataset types.
Two sources feed it: `LiveSource` (network side, fetches bounded views through
`ports.CaseAuthorityPort` — nothing decides, every fact is what the authority reported, and a fetch
failure surfaces as a typed error rather than a degraded guess) and the fixture dataset (build-tag
and fixture-scoped). The `Store` is the revision history behind the live `/api/world` history and
SSE replay.

## Interface
### Live path
- `LiveSource` — fetches case facts through `ports.CaseAuthorityPort` (case.list, case.get_overview, case.get_horizon, approval.list, run.list): `NewLiveSource`, `Facts`, `Snapshot`, `NewestCaseID`, `EntryOptions`, `Preflight`, `VerifyPerspective` [live.go:LiveSource]
- `Build(facts CaseFacts) contract.CognitiveWorldSnapshot` — the pure builder (items, runs, evidence children, salience, actions, summary, attention) [builder.go:Build]
- `ProvenanceLabelLive` — `"go:live:sfwp"`, the served provenance of live snapshots [builder.go:ProvenanceLabelLive]
- Role gates — `MayDecideApproval`, `MayExecute`, `MayPropose`, `MayDriveLifecycle`, `RoleOffersConsequentialWork` [roles.go:MayDecideApproval]
- `EmptyWorld`, `Templates`, `PreflightResult` — snapshot/contract mapping helpers [live.go:EmptyWorld]

### Revision store
- `Store` — bounded, cursor-keyed revision history: `NewStore`, `NewStoreWithRetention`, `Live`, `Head`, `Oldest`, `At`, `Trajectory`, `Append`, `Subscribe`, `CountAfter`, `Len`, `SubscriberCount` [store.go:Store]
- `Revision`, `CaseFacts`, `CloneFacts` — stored revision shape and deep-clone helpers [store.go:Revision]
- `ErrUnknownCursor` — one error for evicted and never-stored cursors (rendered as 404; SSE maps "older than oldest retained" to `resync_required`) [store.go:ErrUnknownCursor]
- `DefaultRetention` = 512 [store.go:DefaultRetention]

### Fixture path (fixture-scoped)
- `FixtureStore` — cursor-keyed fixture revision store: `NewFixtureStore`, `Live`, `At`, `Window`, `Append`, `Subscribe`, `LiveCursor`, `SubscriberCount`; `ErrUnknownFixtureCursor` [fixture_store.go:FixtureStore]
- `Fixture()`, `FixtureBytes()` — embedded Northstar dataset [fixture.go:Fixture]
- Fixture types: `Dataset`, `Object`, `Surface`, `Relationship`, `Snapshot`, `FixtureRevision`, `WindowEntry`, `Artifact` (leveled content via `Artifact.Content`), `Position`; `ProvenanceLabel` = `"go:fixture:northstar"` [fixture_types.go:Dataset] [fixture_types.go:ProvenanceLabel]

## Invariants
- Revisions are keyed by the KERNEL's event cursor — the gateway never mints cursors of its own, so `/api/world?cursor=...` serves true kernel history and an SSE event's id IS a kernel cursor [store.go:package doc]
- Retention is bounded (default 512): the oldest revisions are evicted; an evicted or unknown cursor answers `ErrUnknownCursor` [store.go:package doc]
- Rebuild failure never stalls the relay: the observed cursor still advances while mutation waiters advance only after a revision is retained (see `internal/server` Relay) [store.go:Subscribe]
- Nothing in `LiveSource` decides: every fact is what the authority reported; fetch failures surface as typed errors, never degraded guesses [live.go:package doc]
- Role gates (`May*`) are closed role sets; consequential work is offered to non-machine roles only [roles.go:RoleOffersConsequentialWork]
- The fixture dataset must stay byte-equal to the canonical dataset in the cognitive-ui app (test-enforced) [fixture.go:Fixture]

## Side Effects
- `LiveSource` makes live kernel reads over the authority port and caches run artifact listings [live.go:Facts] [live.go:runArtifacts]
- `Store`/`FixtureStore` maintain in-memory revision windows and broadcast to subscriber channels [store.go:Append] [fixture_store.go:Append]
- `Fixture()` decodes the embedded base64 fixture JSON [fixture.go:decodeBase64]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
