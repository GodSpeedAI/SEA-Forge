# fakeauthority

## Overview
Provider-side adapter used by boundary and preflight tests. Its wire shape is deliberately
provider-flavoured (a foreign envelope with foreign field names) so the adapter boundary is real
rather than asserted: everything this adapter returns is translated into `internal/ports` types,
and `internal/boundary` fails the build if a core package imports this package or mentions its
vocabulary. It is a test-support adapter, not a production integration.

## Interface
- `Authority` — fake `ports.Health`/`ports.CaseAuthorityPort`-shaped provider over a foreign wire envelope; methods `Health`, `Settlement`, `ListCases`, `History` [authority.go:Authority]
- `ArtifactProber` — fake `ports.ArtifactStore`-shaped prober; methods `Health`, `Put`, `Get` [authority.go:ArtifactProber]
- `Probe` — minimal prober that reports a fixed `Err` through `Health` [authority.go:Probe]
- `standing` — maps a provider code onto a `ports.Standing` [authority.go:standing]

## Invariants
- All returned values are `internal/ports` types; the provider envelope stays inside this package [authority.go:wireEnvelope]

## Side Effects
- None (in-memory fake; no I/O)

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
