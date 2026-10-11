# Private Next production source independent review

Date: 2026-10-09  
Verdict: **REJECT pending three bounded lifecycle fixes and one formatting correction**

## Scope and frozen identities

I reviewed the complete original private Next assignment and its sequencing,
notifier-ownership, projection-failure, and approved initial-unavailable
correction addenda, plus the integration root decisions. The frozen builder
packet is `private-next-production-builder-oct09.md` (SHA-256
`b5f2ba1a67191b97eb5e0fa88d6d75774dd0f6074da480e91f6a7ee50d6055bf`). The
approved fixture identity remains
`run_observation_next_test.go` SHA-256
`36dab5455a24521a6715f5ca8f9597a2cad28a070f7c5f09df56598c28386780`.

Reviewed production source identities:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `eefe4dda8a81e48041e290c438512197cc47d5453a8991f0ce207921511c6dc8` |
| `run_observation_poller_worker.go` | `fdd84ca4d5ef891baee20a88992f34b95bee1e0d28abe2242c309a4e399bb8f7` |
| `run_observation_next.go` | `1340ee2d4c05e4f0756730b2e86e652a67caecfec19298828658c98752dce550` |

I inspected the complete changed source and diff. No production implementation
or tests were changed in this review.

## Blocking findings

### 1. Prepare sends the seeded wake while holding `manager.mu`

In `run_observation_manager.go`, both the unavailable-list path (lines 410–421)
and complete-list path (lines 444–455) hold `m.mu` while doing a nonblocking
send to `lease.wake` (lines 418 and 452). The assignment explicitly prohibits
channel sends under this mutex. Nonblocking capacity does not remove the lock
ordering violation. Preserve the list-state/cohort updates under the lock,
then seed the wake after unlocking, with the successful-Prepare behavior and
drain ordering still proved.

### 2. Final-commit failures return without consistently scheduling terminal drain

In `run_observation_next.go` lines 198–201, the final locked validation
includes `ctx.Err() != nil`, but this branch unlocks and returns unavailable
directly. If cancellation becomes visible after the preceding outside-lock
context/auth checks and before this commit check, the call returns without
`startLeaseDrain`. The following final-commit branches (lines 203–214) also
return directly for inactive cohort, exact captured-entry mismatch, or missing
reverse reference. The assignment classifies caller cancellation and failed
final exact ownership validation as terminal. Preserve the no-commit result,
unlock, then schedule the drain before returning; `startLeaseDrain` is
idempotent when Stop/detach has already begun, and operation release remains
deferred.

### 3. A visible cancellation can be mistaken for recoverable marker state

In `run_observation_next.go` lines 148–155, the capture loop returns typed
unavailable for `retainedReadUnavailable` and `retainedRetentionUnavailable`
without checking the caller context. Cancellation can occur after the
post-wake authorization/context check and before capture acquires `m.mu`; if
the captured current value is one of those markers, this branch returns as a
recoverable marker result and leaves the lease active. The assignment requires
cancellation to enter terminal drain. Check caller cancellation before
treating a captured marker as recoverable; when cancellation is visible,
unlock and schedule the single drain owner before returning zero/no commit.

### 4. New lease wake initializer has a formatting discrepancy

The `wake: make(chan struct{}, 1)` initializer in
`run_observation_manager.go` line 134 is the sole newly added field line in
the lease literal and is not aligned with its surrounding multiline fields.
Root flags this native-patch formatting as needing correction. Normalize only
the exact initializer block with a native patch; no whole-file formatter write
is requested here. I did not run a formatter probe.

## Other lifecycle checks

The transaction otherwise reflects the specified ownership shape: operation
token and `operations.Add` are admitted under the same mutex as drain state;
authorization/context checks and projection are outside the lock; capture
copies exact entry/current/ledger/watermark values; commit checks exact token,
cohort, manager entry, and forward/reverse references while allowing current
to advance; marker recovery is distinct from projector/terminal failure; and
deferred operation release avoids waiting on its own drain reference.

The notifier registration helper validates the exact current manager entry
and both attachment directions, rejects draining leases, and registers
`notifyWG` under the manager lock; the actual worker calls it and the shared
send/release helper outside the lock. Stop and drain transition lease
`draining` under that same lock, so I found no additional notifier ownership
race in the frozen lifecycle. Successful terminal current remains available
for Next after worker completion. The initial nil-current read failure and
Prepare release contract remain intact. No separate change to selection,
retention limits, or public behavior was observed.

## Runtime evidence and limits

Root reports the authorized focused manager/Next race gate passed with exit 0
in 18.562 seconds and archived/compared all six actual captures under
`next-private-focused01`. That runtime result does not resolve the source
deviations above. This review ran no compiler, tests, formatter,
or gates and makes no runtime-green claim of its own. The implementation
cannot be approved until the bounded fixes are frozen and independently
reviewed; root remains the sole compiler owner.
