# Next integration: initial lease-bookkeeping TDD unit

Date: 2026-10-08  
Scope: read-only implementation decomposition; no source or test changes.

## Proposed first unit

Limit the first TDD unit to two files:

* `apps/godspeed-casework-go/internal/server/run_observation_manager.go`
* `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go`

Add only these private lease fields:

```go
listState  contract.RunTraceListState
watermarks map[runObservationPollerKey]runObservationWatermark
```

Initialize the per-lease map in `beginPrepareOperation` (`manager.go:112-129`). Populate the list-state field under `m.mu` from the actual validated `Prepare` list outcome: `unavailable` on the existing `listErr` return path (`:394-411`) and `complete` for a successfully decoded list, including a genuinely empty list. Do not infer outcome from `len(lease.pollers)`. The existing contract type and its two values (`complete`, `unavailable`) are defined at `internal/contract/contract.go:188,440`; no public or persisted schema change is needed.

For each accepted `retainedCurrent` state, seed `lease.watermarks[key]` inside the same critical section that validates `m.pollers[key]`, `lease.pollers[key]`, phase, state key, and availability (`manager.go:516-523`). Seed before building `runObservationInitialRun(state)` (`:531`) and before the outer `boundRunObservationHydration` call (`:556-560`). Copy the complete scalar baseline represented by the existing `runObservationWatermark` (`run_observation_delta.go:40-46`): highest observed ordinal, execution/settlement/observation state, and window counts/generation. Store no frames, event IDs, retained pointers, or duplicate ledger. A small private conversion helper in `manager.go` may derive this value in O(1) from the immutable accepted state.

The seed is bookkeeping only. Keep `authorizeLeasePresent` outside the lock and the existing final `leaseCanDisclose` exact-current-pointer, exact manager entry, forward ref, reverse ref, and active-cohort checks unchanged (`manager.go:562-570,588-646`). Do not use the watermark as an authority token or weaken the final handoff.

## Focused test seams

Extend `TestRunObservationManagerPrepareReturnsExactEmptyInitialEventAndLease` (`run_observation_manager_test.go:296-343`) to assert `listState == complete` and an initialized empty watermark map. Extend `TestRunObservationManagerRefusedListReturnsUnavailableDTOAndEmptyLease` (`:1002-1050`) to assert `listState == unavailable` and no seeded watermark. These tests exercise the existing real `prepare` path and preserve its DTO assertions.

Add `TestRunObservationManagerPrepareSeedsInitialWatermarkBeforeHydration` in the same test file. Use the existing fake list/trace/manager fixture, a complete authorized run, and a deterministic accepted snapshot whose initial DTO is demonstrably pruned by `boundRunObservationHydration`. After `prepare` returns, inspect the lease map under `manager.mu`: the key's watermark must equal the captured accepted state's full scalar baseline, including its `HighestOrdinal`, even though the initial DTO omits frames. Assert the DTO actually pruned frames so the test cannot pass without crossing the boundary. Detach and drain through the normal lease API on completion.

## Follow-on units and limits

Do not add `Next`, wake channels, `nextInFlight`, operation/notifier wait groups, worker notifications, or new drain logic in this unit. The next separately scoped unit should implement Next capture/assembly/final-auth/atomic per-lease watermark commit using one captured version plus a copied ledger under `m.mu`, and its own caller context. A later worker/drain unit should add coalesced capacity-one wake notification and notifier/operation joins. This ordering follows the root decisions and keeps incomplete notification or `Add`/`Wait` scaffolding out of the initial-state change.

The unchanged private implementation signature is `prepare(ctx context.Context, caseID string, identity *requestIdentity) (contract.RunTraceObservationEvent, *runObservationLease, error)` (`manager.go:321-325`). Revision 6's conceptual API remains `Prepare(context.Context, CohortRequest) (InitialCohort, CohortLease, error)` and `CohortLease.Next(context.Context) (ObservationDelta, error)`; this first unit introduces neither API. The reviewed root decisions, their independent architecture review, the lease integration recon, revision 6/addendum/correction 3, and the separately approved private policy are inputs only; this report grants no source release or runtime claim.

No compiler, tests, formatter, scanner, build, or Git operation was run. Source identities reviewed: manager `22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57`, manager tests `cf7188c23e5bad5f7b0f093561bf85e1e273b48c721a677084429e8038f0ba86`, delta helper `5241e656d65f86a315e37df44b48d5eaae865c92cf80cc306c9bf4fc0fd7e128`.

Graft retrieval this turn saved approximately 86,071 tokens; reported value approximately $0.05 plus four calls individually reported under $0.01 each (total under $0.09).
