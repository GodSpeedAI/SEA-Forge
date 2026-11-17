# artifactstore

## Overview
FIXTURE-SCOPED artifact persistence for the in-process Northstar fixture stack. Durability here is
the running process's memory: a persisted artifact survives for the process lifetime and is served
with leveled content, but this is never governed persistence — a restart reseeds from the fixture.
Nothing in this package may be presented as a governed artifact store.

## Interface
- `Store` — in-memory leveled artifact store seeded from `[]projection.Artifact` [store.go:Store]
- `New(seed []projection.Artifact) *Store` [store.go:New]
- `(*Store).Get(ref string) (projection.Artifact, bool)` [store.go:Get]
- `(*Store).Persist(ref, boundObject, title string) projection.Artifact` [store.go:Persist]
- `(*Store).Count() int` / `(*Store).List() []projection.Artifact` [store.go:Count] [store.go:List]

## Invariants
- In-memory durability only: a restart reseeds from the fixture [store.go:package doc]

## Side Effects
- Mutates in-process memory on `Persist` (no disk or network access) [store.go:Persist]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
