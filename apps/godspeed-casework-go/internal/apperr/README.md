# apperr

## Overview
The application's stable, provider-independent error model (REQ-ARCH-002). Provider and transport
errors are translated into these kinds at the adapter boundary; no provider error type, code, or
payload may be referenced from this package. Callers switch on `Kind`, never on a provider code.

## Interface
- `Kind` — failure classifier; closed set `config`, `unavailable`, `authority_denied`, `invalid`, `internal` [apperr.go:Kind]
- `Error` — typed error carrying `Kind`, `Capability`, `Op`, `Message`, wrapped `Err` [apperr.go:Error]
- `New`, `Wrap` — construct typed errors with/without a wrapped cause [apperr.go:New] [apperr.go:Wrap]
- `(*Error).With`, `(*Error).Unwrap` — attach cause; support `errors.Is/As` [apperr.go:With] [apperr.go:Unwrap]
- `KindOf` — reports the `Kind` of any error (empty for nil, `KindInternal` for untyped) [apperr.go:KindOf]
- `CapabilityOf` — reports the capability scope of an error [apperr.go:CapabilityOf]

## Invariants
- The kind set is deliberately small and stable; a caller never switches on a provider code [apperr.go:Kind]
- No provider error type, code, or payload may be referenced here — `internal/boundary` fails the build if one appears [apperr.go:package doc]

## Side Effects
- None (pure error construction and classification)

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
