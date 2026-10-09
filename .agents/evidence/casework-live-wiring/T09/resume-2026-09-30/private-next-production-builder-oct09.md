# Private Next production builder record

Date: 2026-10-09  
Status: source-only implementation, frozen for root review. No runtime GREEN,
source approval, T09 settlement, or public readiness is claimed.

## Governing assignment and evidence reviewed

Implemented against the full bounded source assignment and its binding
clarifications:

- `run-observation-private-next-assignment-oct09.md`
- `run-observation-private-next-assignment-addendum-oct09.md`
- `run-observation-private-next-notifier-proof-addendum-oct09.md`
- `run-observation-private-next-projection-failure-addendum-oct09.md`
- `private-next-initial-unavailable-fixture-correction-oct09.md`
- `run-observation-next-integration-root-decisions-oct08.md`
- `live-cursor-v4-complete-candidate-revision6-oct08.md`
- `run-observation-manager-revision6-retention-initializer-correction-revision3-oct06.md`
- `run-observation-manager-revision6-retention-initializer-correction-revision3-independent-review-oct06.md`
- `run-observation-private-policy-operator-approval-oct08.md`
- `run-observation-private-next-assignment-independent-review-oct09.md`
- `private-next-tdd-repair3-independent-source-review-oct09.md`
- `private-next-projection-failure-independent-review-oct09.md`

The frozen Next fixture hash is
`36dab5455a24521a6715f5ca8f9597a2cad28a070f7c5f09df56598c28386780`, matching
the source-approved identity. The behavioral RED remains root's recorded
`next-private-red01`: two unimplemented-Next failures and the passing initial
unavailable cleanup fixture. This builder did not edit tests or alter that
hash.

## Source result

The implementation provides a bounded single-call transaction: it admits an
exact non-zero-size token and operation before callbacks, validates before and
after waiting, captures the full cohort/immutable states/copied ledgers/prior
watermarks under the manager mutex, projects only those captures, and commits
all candidate watermarks together after final authorization and exact
attachment checks. Current-pointer advancement during projection is allowed;
the next publication remains coalesced in the capacity-one wake. Run and gap
collections are nonnil and ordered by run ID. Explicit transient read and
nonterminal retention markers return unavailable without advancing watermarks;
unexpected projection errors and terminal failures schedule the existing
asynchronous drain owner.

Prepare now sets immutable lease case identity and seeds the never-closed wake
for successful complete and unavailable-list leases. Worker publication uses
the production notifier registration/send helpers, with notifier references
registered under the manager mutex and nonblocking sends/releases outside it.
Drain joins creator, admitted operations, registered notifications, and actual
workers before releasing lease capacity. The initial nil-current read failure
path remains unchanged and does not emit a worker wake.

## Exact source identities

SHA-256 after source edits:

| File | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `eefe4dda8a81e48041e290c438512197cc47d5453a8991f0ce207921511c6dc8` |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `fdd84ca4d5ef891baee20a88992f34b95bee1e0d28abe2242c309a4e399bb8f7` |
| `apps/godspeed-casework-go/internal/server/run_observation_next.go` | `1340ee2d4c05e4f0756730b2e86e652a67caecfec19298828658c98752dce550` |
| Frozen `run_observation_next_test.go` | `36dab5455a24521a6715f5ca8f9597a2cad28a070f7c5f09df56598c28386780` |

## Limits and next verification

No compiler, test, formatter, repository gate, status/debt edit, commit, or push
was run here. `git diff --check` was the only source check. Root owns the sole
compiler and must inspect this exact source, run the authorized focused manager
and Next race gate, and preserve its actual captures before any later Go gates.
Independent critic review and runtime GREEN remain outstanding. There are no
known intentional deviations from the assignment; source correctness and all
race behavior remain unverified until those reviews and gates complete.
