# Phase A failure fixture repair result — 2026-10-07

## Exact source identity and preimage

Only `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go`
was edited. Its exact preimage was captured before editing in
`run-observation-manager-failure-test-preimage-oct07.json` (SHA-256
`e68b31f018f33fd43553fda4ad839f8df486605641da7d0f7b592b4b8479a02a`). The
JSON records the source path, byte count, source SHA-256, and exact source bytes
encoded as gzip plus base64. Before the edit, the decoded bytes were 27,806
bytes and SHA-256
`3b0f0653892cb8bcd9823a91779ffbd03e37d015c41242762091bdfad8233990`; decoded
archive bytes were checked against that identity.

The edited fixture is 27,708 bytes with SHA-256
`4ff2fe1c922d6ae7e232d5a58f201298e232807437967fe22a9316dcb45aa0f3`.
The complete bounded instruction was archived before the source edit at
`run-observation-manager-phase-a-fixture-repair-assignment-oct07.md` (SHA-256
`dc37aa4b06d995431fadaa581ce9a87e85fc62db7514534523bf1d6d81722d3e`). It
includes the current root instruction, the authorized Phase A release identity,
and the full independent rejection review basis by immutable path and hash:
`run-observation-manager-phase-a-independent-source-review-oct07.md`, SHA-256
`63ce065c46739005279fcd631e066ce12639c1fb67ac3d8c31791979825f7324`. The
reviewed Phase A release is
`run-observation-manager-phase-a-testfirst-assignment-oct07.md`, SHA-256
`7211787b2f4a07db3d8cddb031ee7f644f38ac5b5331f12ab0de54d5fec66565`.

## Finding-to-change map

1. In `TestRunObservationManagerFailureStopJoinsHeldActualRead`, retain the
   fixture's returned caller (`manager, caller, _, _`). This resolves the
   review's undefined identifier at the `manager.prepare` call.
2. In that same held-actual-read case, remove replacement of `entry.cancel`,
   the `TryLock` spy, and the spy assertion. Preserve the real held port call,
   manager cancellation signal, registered-entry assertion before return,
   assertion that Stop has not returned, release, Stop completion, worker
   completion, post-JOIN removal assertion, initial successful lease, and
   actual-call count check. Lock discipline remains a production source-review
   responsibility; the removed spy was not conclusive evidence.
3. In `TestRunObservationManagerFailureStopWinsBetweenReserveAndLaunch`, call
   the real `manager.claimPollerReadStart(entry)` after the test observes
   `manager.stopping` and before launch; fail if the claim succeeds. In
   `TestRunObservationManagerFailureDetachWinsBeforeFirstReadUsesDrainingCode4`,
   do the same after observing `lease.draining` and before launch. The already
   existing positive claim-before-Stop case is unchanged.

The fixture still contains the same nine cases, in the same order:

1. Stop wins between reservation and launch.
2. Detach wins before first read and uses draining code 4.
3. A successful read claim precedes Stop and uses worker continuation.
4. Stop joins a held actual read.
5. Global Stop releases distinct pending leases.
6. A canceled Prepare leaves an authorized lease.
7. Detach and selected-A release have one owner.
8. Shared first-read error remains successful A.
9. Oversized nil key remains A without admission.

No other assertions, count values, authorization/ownership expectations,
helpers, or cases were changed. The patch adds two negative read-claim
assertions, changes one fixture result binding, and removes only the rejected
cancel-field spy and its assertion.

## Frozen source identity checks

Read-only SHA-256 checks after the edit confirmed these protected files still
match their assigned identities:

| Path | SHA-256 |
| --- | --- |
| `run_observation_manager.go` | `f9321a620ad64546e735b936d0b58b9f514eab0d7c6d0325f1e7a82f37c5e314` |
| `run_observation_poller_worker.go` | `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

## Limits and deviations

This is source-only fixture repair evidence. No compiler, test, formatter,
scanner, Graft build, or Git mutation was run. It makes no compile, focused
RED, lifecycle behavior, or runtime claim. No deviation from the released
source boundary or the three authorized findings was identified. The fixture
is ready for a different independent source critic; expected-RED execution
still requires root's separate authorization.
