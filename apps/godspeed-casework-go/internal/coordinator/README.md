# coordinator

## Overview
Intent handling for the casework boundary's fixture stack: validation, idempotency, the FIXTURE
authority decision, lease lifecycle, and the staged world revisions an accepted consequential intent
produces. FIXTURE-LABELED: the authority here is an allowlist over the Northstar fixture, not a
governed authority. It refuses everything it does not explicitly permit (decide-approval is refused
outright because the fixture has no review capability), and a refusal has ZERO effects: no lease,
no revision. Real governed authority wiring must replace this decision point without changing the
refusal contract.

## Interface
- `Coordinator` — owns the fixture intent pipeline: `New(proj, arts, opts)`, `Submit(Intent) Outcome` [coordinator.go:Coordinator]
- `Intent` — the incoming intent (id, kind, target, parameters, cursor) [coordinator.go:Intent]
- `Outcome` — the typed result: accepted-with-lease or refused [coordinator.go:Outcome]
- `LeaseView` — lease state projection; `Leases() []LeaseView`, `SubscribeLeases() (<-chan LeaseView, func())` [coordinator.go:LeaseView]
- `Options` — tunable stage timing (stages 1/2, sleep hook) [coordinator.go:Options]
- `validateStructure`, `freshnessRefusal`, `bodyHash` — structural validation, stale-cursor refusal, idempotency hashing [coordinator.go:validateStructure]
- `record`, `replay` (via `record`) — idempotent outcome replay keyed on intent id + body hash [coordinator.go:record]

## Invariants
- The fixture authority is an allowlist: `implement` only for `ns-migration`/`ns-secondary`; `decide-approval` refused `authority_denied`; refusals have ZERO effects [coordinator.go:package doc]
- Intent ids are idempotent: same id + identical body replays the recorded outcome; same id + different body is refused `invalid` [coordinator.go:record]
- A stale intent cursor is refused before any effect [coordinator.go:freshnessRefusal]
- Accepted intents run staged lease transitions (claimed → active → revisions appended → released) that append fixture-store revisions and stream to subscribers [coordinator.go:runStages]

## Side Effects
- Appends revisions to the `projection.FixtureStore` (mutating the fixture world) [coordinator.go:appendStage]
- Emits lease events to subscribers on `SubscribeLeases` channels [coordinator.go:emitLease]
- Spawns goroutines for staged lease runs [coordinator.go:runStages]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
