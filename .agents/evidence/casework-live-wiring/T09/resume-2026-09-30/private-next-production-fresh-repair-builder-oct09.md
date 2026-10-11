# Private Next production fresh-repair builder record

Date: 2026-10-09  
Status: bounded source-only repair frozen for independent review. No new
runtime-green, source-approval, T09 settlement, or public-readiness claim.

## Governing grant and rejection

This repair follows the complete bounded grant and its binding clarifications:

- `run-observation-private-next-assignment-oct09.md`
- `run-observation-private-next-assignment-addendum-oct09.md`
- `run-observation-private-next-notifier-proof-addendum-oct09.md`
- `run-observation-private-next-projection-failure-addendum-oct09.md`
- `private-next-initial-unavailable-fixture-correction-oct09.md`
- `run-observation-next-integration-root-decisions-oct08.md`

The frozen production source was rejected by
`private-next-production-independent-review-oct09.md`, SHA-256
`c9bf15996eb1aacb5881e4b6953afee714e47f93a3f19c0034a8c4278434a445`.
The review preserved the frozen test identity
`36dab5455a24521a6715f5ca8f9597a2cad28a070f7c5f09df56598c28386780` and
worker identity `fdd84ca4d5ef891baee20a88992f34b95bee1e0d28abe2242c309a4e399bb8f7`.

## Bounded findings and repairs

1. Both Prepare list-outcome paths updated `lease.listState` and seeded the
   capacity-one wake while holding `manager.mu`. Each path now stores list
   state under the mutex, unlocks, and then performs the nonblocking seed send.
   Prepare still owns the registered creator operation; the existing drain
   joins that creator, so no extra wait-group accounting or test hook was added.
2. Final transaction validation failures for context/token/cohort state,
   manager/lease entry identity, and reverse references now unlock and call the
   existing idempotent `terminalFailure`. Candidate watermarks and aggregate
   remain uncommitted, and deferred operation release remains in place.
3. Before returning a captured read-unavailable or nonterminal
   retention-unavailable marker, capture checks `ctx.Err()` while the capture
   mutex is held. A visible cancellation unlocks and enters `terminalFailure`;
   the recoverable marker result is used only when cancellation is not visible.
4. The `wake` initializer line alone was aligned to its neighboring fields with
   a native patch; no formatter rewrite was performed.

## Exact source identities

| File | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `941851e03da13a8781eb3dd3324ba331be67254c939d0c8fd274531459ff60ee` |
| `apps/godspeed-casework-go/internal/server/run_observation_next.go` | `6ed10dec3ea33e660dad5ea59f92764210592f56c7a9cb9dc7034a2238a9b3f5` |
| Frozen `apps/godspeed-casework-go/internal/server/run_observation_next_test.go` | `36dab5455a24521a6715f5ca8f9597a2cad28a070f7c5f09df56598c28386780` |
| Unchanged `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `fdd84ca4d5ef891baee20a88992f34b95bee1e0d28abe2242c309a4e399bb8f7` |

## Checks and limits

Read-only checks for this repair were direct inspection of the changed source,
the scoped read-only diff, and exact SHA-256 identity capture. The frozen Next
test and worker hashes match their grant identities. There are no deviations
from the bounded source grant. No compiler, test, formatter, repository gate,
or Git mutation was run. Root's previously reported focused result (50 checks
passing) is historical and does not resolve the source deviations or approve
this repair. Independent source review and actual runtime confirmation remain
pending. No fixture, worker, public contract, authority, policy, dependency,
or other source file was changed.
