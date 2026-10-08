# Run-children Phase 1 independent review and expected RED

Review date: 2026-10-01. This is a bounded Phase 1 source/test review and assertion-RED record against the rejected Phase B production. It does not approve production behavior, Phase B, the real-kernel proof, broader Go verification, or T09 settlement. No source/test edits were made by the critic.

## Governing scope

Reviewed the original `run-children-runtime-assignment.md`, `run-children-runtime-test-phases.md`, `run-children-root-fixture-clarifications.md`, `run-children-real-kernel-proof-supplement.md`, immutable `run-children-phase-b-independent-semantic-reject.md`, and the fresh builder's Phase 1 tests. The Phase 1 semantic-repair assignment requires only the two new scoped tests to change before this RED run; the rejected production graph must remain frozen. See `run-children-phase-b-semantic-repair-assignment.md`, especially lines 14–34.

## Frozen source identities at review

SHA-256 values measured immediately before the focused tests:

- `internal/ports/ports.go`: `e4343e489927c553c789ea3dab6f191326ea31e57b765e295fd184a9eddcd686`
- `internal/projection/builder.go`: `3edf1db574480e3d5ce619f4fa9df4c3998127dbb13f6cf3bb84cd1df36d10b2`
- `internal/projection/live.go` (still rejected Phase B): `d4d517dd68325353e093b3c24ba9e9fb27ec4df4339026b25f94d55288abc706`
- `internal/projection/scoped_live_test.go` (Phase 1 repaired fixture): `a9e0deb03f69b12ab379e547de2fb2aaf20c13dd0a51929cdeeda0bea14e7116`
- `internal/projection/scoped_runs_live_test.go`: `8e485c64cdc0776d46a8fda6678d22f3bfe98ec837bd8eaaeb94fecced8a9134`
- `internal/projection/unreadable_runs_test.go`: `de85a2edb9bd05dc5542c7c9850a9cf9caea69dbb491eb21f95fe93913bfcebb`
- `internal/adapters/sfwp/frame.go`: `be8800c437c9f2f699772371a28f14dc068e7c8bc8a543955f6874c0b6259750`
- `internal/adapters/sfwp/authority.go` (still rejected Phase B): `6a047a3a56192eacbb0f8bb816516d872eca944425c796e531b79859d2e21b15`
- `internal/adapters/sfwp/scoped_runs_test.go` (Phase 1 repaired fixture): `57bfc686da998303655bd99fdc5bffcbf386ddbcb931ea8bc290762d848fad5e`
- Preserved pure-child fixture `internal/projection/run_children_test.go`: `4ddb2445a7ff6726c3b6fd93832ac4122b53da4d26e5b54180f9a1cafb802741`

## Source/test assessment

The Phase 1 changes are within the authorized two-test scope. `scoped_live_test.go` now tests both an empty case list and a nonempty foreign-only list for a missing requested case, with exactly one ListCases call and no overview, horizon, approvals, scoped-list, or legacy-list calls. It also selects `case_1` after a preceding foreign record and asserts the exact selected record and exact scoped ref. Existing ownership/malformed/duplicate guards remain in the fixture.

`scoped_runs_test.go` retains the explicit unavailable-class/error-chain test and adds input_error → Invalid, identity_required → AuthorityDenied, unsupported_version → Unavailable, and unknown_refusal → Internal, all asserting `errors.As` and preservation of the wire refusal class. These are direct, scope-matched assertions for the required typed mapping behavior.

The pre-implementation source remains known defective, intentionally: `projection/live.go:60–82` performs downstream reads before checking `record == nil` and returns Unavailable when another case exists; `authority.go:261–274` wraps Refusal as Unavailable. The RED tests directly distinguish those semantics.

The test covers the ordinary `Client.Do` refusal route. `client.go:376–397` returns decoded response refusals as `resp.Err.appErr(...)` before returning a `Response`, so the `authority.go:270–274` `Response.Into` refusal branch cannot be reached with the concrete production Client for an ordinary refusal. The adapter's second defensive branch is therefore not independently runtime-exercised by this focused test; its source remains included in Phase 2 review. No claim of coverage for that unreachable branch is made here. The test did not use `server_busy`, so no retry semantics were changed or exercised.

## Focused expected-RED commands and actual results

Each command had its own immediately preceding actual-host preflight. Both used:

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0
go test -race -count=1 -parallel=1
```

1. From `apps/godspeed-casework-go`:

   `go test -race -count=1 -parallel=1 ./internal/projection -run 'TestLiveSource(SelectsRequestedCaseAfterForeignRecord|RejectsUnknownRequestedCaseBeforeDownstreamReads)$'`

   Actual exit status: `1`. The positive foreign-before-requested selection test passed. Expected RED occurred in both unknown-case variants: the empty-list case showed downstream call counts `1/1/1/1/1/0` rather than `1/0/0/0/0/0`; the foreign-only case returned Unavailable rather than Invalid. The package compiled and both failures reached their intended assertions.

2. From `apps/godspeed-casework-go`:

   `go test -race -count=1 -parallel=1 ./internal/adapters/sfwp -run 'TestScopedRunsListPreservesOtherRefusalKinds$'`

   Actual exit status: `1`. `unsupported_version` passed as Unavailable. Expected RED occurred for `input_error` (observed Unavailable, expected Invalid), `identity_required` (observed Unavailable, expected AuthorityDenied), and `unknown_refusal` (observed Unavailable, expected Internal). The package compiled; the failures reached the typed-kind assertions, and the test also checked that each Refusal remained in the error chain with its original class.

## Raw artifacts and capture identity

All original command outputs, exit files, and preflights were captured separately under `/tmp`, then archived with native `apply_patch` into new paths. The following original-to-archive pairs were SHA-256 identical:

| Original `/tmp` artifact | New evidence artifact | SHA-256 |
|---|---|---|
| `/tmp/t09-unit4-phase1-projection.raw` | `unit4-phase1-projection.raw` | `7b3287427020d11feeb94cbf583891c7cba869104e62f32ee3e021f247c91757` |
| `/tmp/t09-unit4-phase1-projection.exit` | `unit4-phase1-projection.exit` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |
| `/tmp/t09-unit4-phase1-preflight.raw` | `unit4-phase1-projection-preflight.raw` | `d5a450be65246c5bc624a30c94c3a299344a8878a88aa653dd08958786e7d35c` |
| `/tmp/t09-unit4-phase1-adapter.raw` | `unit4-phase1-adapter.raw` | `d3cea23bed7bceb1ae1bbb6bf7b4dad52093694890b0ee0cb9beb3665651da6b` |
| `/tmp/t09-unit4-phase1-adapter.exit` | `unit4-phase1-adapter.exit` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |
| `/tmp/t09-unit4-phase1-adapter-preflight.raw` | `unit4-phase1-adapter-preflight.raw` | `59b260d4ad2d72c3f2059d32847f403c551b56d359a09326123e4024ad1012fc` |

Both actual-host preflights recorded RAM and compiler-process scans. The only matching process was PID 2109808, a Bun `server.ts` service under the Cognate reconstruction; it was left untouched and was not a Go/Cargo compiler. No competing compiler was listed. Available memory was 2.2 GiB at both preflights (free memory was 434 MiB then 267 MiB); this is actual-host evidence, not sandbox PID-namespace output.

## Disposition

Phase 1 expected RED is accepted as evidence that the new assertions compile and expose the two known semantic defects. It does not approve the rejected Phase B implementation. The compiler token is released to root after this bounded gate. Fresh Phase 2 source repair and independent source review are still required, followed by focused GREEN, the required actual-kernel two-case scoped-run proof, fresh canonical/full Go gates, and later T09 units/global settlement.
