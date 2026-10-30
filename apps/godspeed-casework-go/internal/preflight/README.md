# preflight

## Overview
Validates configured capabilities BEFORE the application accepts consequential work, and reports
each capability's state so a failure disables only what it must. REQ-CONFIG-010: required
ports/adapters and configuration are validated before consequential work. REQ-CONFIG-012: a
preflight failure disables only the capability whose dependency is unavailable, unless that
capability is required for application startup — in which case the whole application is blocked
(exit 2 at the command layer), and the unrelated capabilities are still reported with their own
honest states.

## Interface
- `State` — per-capability state: `ready`, `degraded` (optional capability unavailable), `blocking` (required capability unusable) [preflight.go:State]
- `Capability` — name, kind, required flag submitted to `Check` [preflight.go:Capability]
- `Result` — one capability's verdict; `Summary` — the aggregate verdict [preflight.go:Result] [preflight.go:Summary]
- `Check(ctx, caps, probers, configProblems) []Result` — probes every capability and folds capability-scoped config faults into per-capability states [preflight.go:Check]
- `Summarise(results) Summary` [preflight.go:Summarise]
- `Failed(results, capability) *apperr.Error` — typed error when a capability is unusable [preflight.go:Failed]

## Invariants
- A required capability that is unusable blocks the application; an optional one degrades only itself [preflight.go:stateFor]
- Capability-scoped configuration faults disable ONLY that capability; unrelated capabilities keep their own honest states [preflight.go:package doc]
- Preflight runs before any consequential work is accepted [preflight.go:package doc]

## Side Effects
- Calls each registered prober's `Health` (for the live authority this probes the real kernel: `system.hello`, `readiness.get`, `identity.get`) [preflight.go:Check]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
