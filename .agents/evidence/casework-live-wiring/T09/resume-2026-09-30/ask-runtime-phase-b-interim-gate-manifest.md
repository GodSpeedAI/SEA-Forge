# Ask phase-B interim gate manifest

This manifest records the frozen production/test source identities and the independently run phase-B gates so far. It is not a final approval; the live test name is being shortened by a fresh builder to make the required owned-temp cell socket path portable, then the live proof and remaining broader gates must be rerun.

## Frozen source identities at this review

- `internal/server/ask.go` `68a588dfe741ae2e5a13b58229c78da4016a94f0355bf54e08952cccb102b699`
- `internal/adapters/sfwp/ask.go` `75702c8577327b84d7cafeb23171c400e914d1a60f575e9e47588f701e8749eb`
- `internal/adapters/sfwp/client.go` `8cdfc52a68c07df84e1f88506163da1a8dab212004d45689a4c8ab8007b3b61a`
- `internal/adapters/sfwp/frame.go` `ab7d05604574029f684874d9da415f9c3c1695778dc45ea12e3bbbb462167185`
- `internal/server/server.go` `31d5d38e8f4f080d472179ce0da7f63ac853a2ed57021d99eea2708c8cd7a023`
- `internal/server/http.go` `ebe23204a2390e4611c77d594e82cd6dfcb0a32343c1113e4900d11cfc5cf25c`
- `internal/server/ratelimit.go` `53e91b67580f79fa002073cb8eef2b43e0d63a1d21548d4f4cbb4b32686edd7b`
- `internal/ports/ask.go` `09c6af57138e52fe0e5dfe504b08a6a4517fa8d65955a7cb508e463f884de6b6`
- `internal/livestack/stack.go` `6c1021b2204cd1f4327a527b3feebd480e1e8007b0bc6bd31f2d7564b8629af5`
- `cmd/godspeed-casework/main.go` `2a9b5c6620aef9d8a7b8cd9f597f245a8a69eae17c84ff172bf55f9c773771a6`
- `internal/adapters/sfwp/ask_adapter_test.go` `7564a7ef04c2b0af242b6a7f270b4a9aef701221ec8ce39ec3f6c51cb4678200`
- `internal/server/ask_live_test.go` `8f3613930cea8052671611549646419c6e6f6e02007a7d28631417f1d3d7eb0f` (the original long-name live fixture; pending an authorized test-name-only edit)
- frozen `internal/server/ask_test.go` `90612d084a10fce6ec4871faae563790c7de5267d2ef60369bc1fc23bd538cbe`
- frozen `internal/adapters/sfwp/ask_transport_test.go` `94764ad34ca808bfa92ed96bfd91acd23fb8767c5fa9645515e57f91395426cc`
- frozen `internal/server/ask_validation_test.go` `a1888b765e7219513bf0ca9b2bd083631f52050f25c230c8e0ead7d1d82de861`

## Gates and boundaries

- Focused full server and SFWP adapter race run passed: `go test -race -count=1 -parallel=1 ./internal/server ./internal/adapters/sfwp`; raw log, status and actual-host preflight are in `ask-runtime-phase-b-focused-race.*`.
- Required actual-kernel live Ask proof passed once with actual-host escalation and `TMPDIR=/tmp/a`. The successful run is in `ask-runtime-phase-b-live-pass.*`. The assertion suite exercises a fresh disposable cell, real denial and granted disclosure, HTTP session/CSRF/invalid refusal non-writes, four linked records per answer, effective writer/delegation attribution, exact eleven-field top-level comparison after canonical ClaimView projection, and `ledger verify`.
- `just casework-go-check` passed format, vet and ordinary full-module tests; raw evidence is in `ask-runtime-phase-b-just-go.*`.
- No full-module race command has run yet. No Ask unit is approved yet.

## Live harness portability correction and retained failed attempts

The test name `TestLiveProtectedAskUsesSessionAndMatchesCommittedDisclosure` plus `t.TempDir` yielded a socket path rejected by the actual kernel. The original default phase-B `TMPDIR` attempt's harness removed its cell `server.log` during cleanup; only that test output and exit/preflight remain, so no child-log diagnosis is claimed for that attempt. An actual captured child log from a later `/tmp/asktmp` attempt reported a 98-byte Unix socket path against the kernel's explicit 95-byte maximum. A default-sandbox attempt under `/tmp/a` used a 93-byte path but got `Operation not permitted`; this is preserved as a sandbox failure, not code or kernel behavior evidence. The successful escalated `/tmp/a` run demonstrates the proof itself passes, but root requests a short stable test name so the live proof works under the normal owned phase-B TMPDIR and does not depend on an unusually short path. Root measured the original phase-B TMPDIR-derived socket at 128 bytes and estimates the approved `TestLiveAskLedger` name path at 85 bytes.

All failed attempts, available child logs, and actual-host preflights are retained in separate sibling raw files. Missing original child logs remain missing; none were reconstructed. Full T09 settlement and this unit's final verdict remain pending the authorized name-only test repair, re-review, live rerun, and full-module race gate.
