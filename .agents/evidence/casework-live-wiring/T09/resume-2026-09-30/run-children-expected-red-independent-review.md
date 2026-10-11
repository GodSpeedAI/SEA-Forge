# Independent run-child fixture source review and expected-RED result

Review date: 2026-10-01

## Source identity and scope

Reviewed the original test-first assignment, root clarifications, prior immutable rejection, and the repaired frozen fixture:

- Test contract: `run-children-test-first-assignment.md`
- Clarifications: `run-children-root-fixture-clarifications.md`
- Prior source rejection: `run-children-fixture-independent-reject.md`
- Test source: `apps/godspeed-casework-go/internal/projection/run_children_test.go`
- Frozen source SHA-256: `4ddb2445a7ff6726c3b6fd93832ac4122b53da4d26e5b54180f9a1cafb802741`

The repaired fixture addresses the rejection: the 24-standing matrix tracks seen IDs, fails duplicate children, and asserts each expected ID occurs exactly once (test lines 40–53 and 91–98). It covers the six-by-four execution/settlement matrix, actual parent IDs, empty child actions, absence of observation/poller/frame JSON keys, more than eight children, separately labelled standings, progress stability, invalid/foreign/duplicate/colliding/missing-parent/unknown-standing rows, valid sibling and parent preservation, clarified parent focus stability, and immutable retained Store facts and snapshots. It adds only the assigned new test file; no production/API changes are part of this review.

## Focused expected-RED gate

Command (run from `apps/godspeed-casework-go`):

```sh
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 \
  GOCACHE=/tmp/t09-run-children-gocache \
  GOTMPDIR=/tmp/t09-run-children-gotmp TMPDIR=/tmp/t09-run-children-tmp \
  go test -race -count=1 -parallel=1 ./internal/projection \
  -run 'TestBuildAddsCaseOwnedRunChildrenForEveryStandingPair|TestBuildOmitsInvalidRunChildrenWithoutDroppingValidSiblings|TestBuildRunChildrenDoNotManufactureParentAttention|TestStoreKeepsRunChildStandingAtCapturedCursor'
```

The first invocation exited 1 before compilation because `/tmp/t09-run-children-gotmp` did not exist. Its exact output and exit are preserved as `run-children-expected-red-first.log` and `run-children-expected-red-first.exit`. After creating the task-owned cache/temp directories, a fresh actual-host preflight showed 2.5 GiB available and no competing compiler in the scan; its output is preserved as `run-children-expected-red-preflight-2.txt`. The foreign Bun server noted by root was not modified or stopped.

The initial attempt to create the archive patch was rejected by `apply_patch` as malformed because the same first-attempt exit path appeared twice. That was an evidence-archive write error only; it did not run a command, change source, or affect either test result. The corrected archive patch created distinct first-attempt and retry exit files.

The retry compiled and ran the tests, then exited 1 with expected missing-child assertions: all 24 expected IDs had zero children; the invalid-row test's required valid sibling and the Store test's historical child were absent. No compiler/build error occurred. Exact full stdout and actual exit are preserved in `run-children-expected-red-retry.log` and `run-children-expected-red-retry.exit`.

The intentional RED proves the central matrix and required valid-child/history positives fail because the pre-child Build emits no run children. The invalid-row test terminates at its `objectByID` fatal for missing `run_valid`, so its later negative guards and sibling-preservation checks do not execute in this RED run. The pure attention-preservation test has no child-existence assertion and can pass on pre-child code; it is a regression guard, not independent RED evidence. This approves the test-first fixture only, not implementation, and does not claim every assertion executed in the RED run.

## Raw artifact identities

The stdout, exit-status, and host-preflight artifacts are separate from this narrative. Hashes are recorded in the accompanying root review/checkpoint after checking the durable copies against their `/tmp` originals.

Disposition: fixture source approved; focused expected-RED behavior confirmed with the execution limitations stated above. No production code or existing tests were changed. Compiler token is released to root after this focused gate; broader Go gates are not part of this unit's claim.
