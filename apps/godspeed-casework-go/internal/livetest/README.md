# livetest

## Overview
Shared harness for `-tags live` integration tests against the REAL `sea-forge-server` kernel.
Nothing here talks to a fake kernel: every helper boots the real binary on a fresh temporary cell
seeded the way the live stack expects — E2E plan templates, the E2E authority policy, and a
`server.yaml` binding the invoking OS user to the gateway principal with `operator_local` (operator)
and `rso_local` (R-SO) in the delegation allowlist. Test-support package; never part of the serving
binary.

## Interface
- `Cell` — one booted kernel process on a fresh temp cell: `NewCell(t)`, `NewCellAt`, `Root`, `PID`, `SocketPath`, `Stop`, `StopGraceful`, `Kill9`, `Restart`, `Client` [harness.go:Cell]
- `ServerBinary(t)`, `EnsureServerBinary()`, `RepoRoot(t)` — locate/build the real kernel binary [harness.go:ServerBinary]
- Observation helpers — `CaseEvents`, `CountTraceKinds`, `RequestRecord`, `CaseDirs`, `DelegationAuditRecords`, `FindDelegationRecord` [harness.go:CaseEvents] [harness.go:DelegationAuditRecords]
- Governance helpers — `GatewayGov`, `DelegatedPortsGovernance`, `DelegateOnBehalfOf`, `DelegatedOpts` [harness.go:GatewayGov]
- `WaitUntil(t, timeout, what, cond)` — bounded polling helper [harness.go:WaitUntil]

## Invariants
- Every helper targets the real kernel binary on a real temp cell; no fake kernel anywhere [harness.go:package doc]
- The seeded cell carries the T02 gateway principal and two delegable end-user actors [harness.go:package doc]

## Side Effects
- Builds the kernel binary and writes temp cell directories on disk [harness.go:EnsureServerBinary] [harness.go:seed]
- Spawns and signals the real `sea-forge-server` process (graceful stop, SIGKILL, restart) [harness.go:Stop] [harness.go:Kill9]
- Reads kernel cell files (events ledgers, request records, delegation audits) [harness.go:CaseEvents]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/); live-tagged tests additionally run with `-tags live` and need a built kernel
