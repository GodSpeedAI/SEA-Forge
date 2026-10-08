# Independent Phase A lifecycle source review — 2026-10-07

**Verdict: REJECT focused expected-RED readiness for these frozen sources.**
This verdict is limited to Phase A compile-safe test-first readiness. It is not
an algorithm review, runtime result, lifecycle approval, or T09 completion
claim. I made no source changes and ran no compiler or test.

## Reviewed evidence and final identities

I read the full Phase A release instruction
`run-observation-manager-phase-a-testfirst-assignment-oct07.md` (SHA-256
`7211787b2f4a07db3d8cddb031ee7f644f38ac5b5331f12ab0de54d5fec66565`), its
source result (SHA-256
`e06d30b178b7efe58d22a959824b5cd34181d348748f23b751d526f5d0ff5ffd`), and
the identity/finding erratum (SHA-256
`7068c5400aad449a12978f7f6fe610941b855db9d145573ba8f24b57cac23492`). The
erratum supersedes the result's preliminary manager hash; review below uses
only the final frozen identities it supplies:

| Path | Preimage | Final SHA-256 |
| --- | --- | --- |
| `run_observation_manager.go` | `ab9f1c35757aecd8c8c02de300a95205a1bd07aa5e39ebe8ae056dbc8878fe62` | `f9321a620ad64546e735b936d0b58b9f514eab0d7c6d0325f1e7a82f37c5e314` |
| `run_observation_poller_worker.go` | absent | `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f` |
| `run_observation_manager_failure_test.go` | absent | `3b0f0653892cb8bcd9823a91779ffbd03e37d015c41242762091bdfad8233990` |

The original source result's manager hash `e2b0...` is preliminary and is not
used here. The erratum explains that the final source was edited after that
result was written. It also independently identifies the undefined `caller`
finding. No compiler was run, so this review makes a direct source-level
finding, not a reproduced compile result.

The protected existing manager fixture remains `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`. The six committed primitive files were rehashed and match the frozen boundary:

| Path | SHA-256 |
| --- | --- |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

The actual source paths are exactly the three authorized paths. Manager and
worker methods are clearly labeled unwired stubs; I do not treat that deliberate
RED setup as a lifecycle implementation or as proof of compilation.

## Material blockers

1. **The fixture has an unresolved identifier.** In
   `run_observation_manager_failure_test.go:255`,
   `manager, _, _, _ := newRunObservationFailureFixture(...)` discards the
   helper's caller return. At line 267, the goroutine passes `caller` to
   `manager.prepare`, but this function has no declaration for `caller`. This
   is the compile-blocking source issue already recorded in the identity
   erratum. The focused RED therefore cannot be approved as assertion-only.

2. **The held-read case mutates a production lifecycle field and installs a
   test spy.** At lines 275–285 it reads `entry.cancel`, replaces it directly
   with a closure that probes `manager.mu.TryLock()`, and invokes the original
   cancel. The corrected proposal explicitly says the new test file must not
   set lifecycle fields directly or add a callback/test hook
   (`manager-read-start-testability-corrected-proposal-oct07.md:120-154`;
   correction supplement §“Stop lock discipline”). The Phase A result also
   claims no generic hook or callback exists, which conflicts with this source.
   The probe is not conclusive: `TryLock` failure can mean another goroutine
   owns the mutex even if cancellation is correctly outside it. The unsynchronized
   replacement also bypasses the manager's ownership discipline for `cancel`.
   Keep the real held port call, cancellation, still-registered assertion,
   Stop-not-returned assertion, return, and worker JOIN checks; remove the field
   replacement and leave lock-boundary proof to independent source inspection
   of the later production implementation.

3. **The two pre-read stop cases do not assert refusal of the read-eligibility
   claim.** `TestRunObservationManagerFailureStopWinsBetweenReserveAndLaunch`
   (lines 121–167) and
   `TestRunObservationManagerFailureDetachWinsBeforeFirstReadUsesDrainingCode4`
   (lines 169–207) observe Stop/draining before calling the real launch batch,
   then assert zero port calls and code 4. Neither asserts that
   `claimPollerReadStart` is refused after the stop/drain transition. The
   approved contract says no later claim may pass after Stop wins the mutex
   (`manager-read-start-testability-corrected-proposal-oct07.md:138-149`), and
   the root decision requires both pre-read branches to prove the existing code
   and phases. As written, a claim that incorrectly succeeds could still pass
   these assertions if the continuation separately suppresses the actual call.
   Add direct negative assertions through the real eligibility method after
   each transition; do not add a hook or field.

## Matrix review and bounded positive findings

The nine named cases correspond to the released matrix. Several important
parts are sound in shape: reservation precedes the observed Stop and the owned
launch batch follows it (lines 126–165); the positive claim precedes real Stop
and the test then calls the mandatory continuation (lines 213–243); the held
read blocks at the actual trace-port call and checks capacity remains until
release/JOIN (lines 255–320); the global-stop and surviving-lease cases use
distinct lease refs (lines 324–478); the shared first-read A case asserts
per-Prepare owner/waiter counts 1/0, successful nonnil leases, exact-ref
cleanup, one list each, one read, and JOIN (lines 546–628); and the oversized
nil-current case permits one selected list while asserting zero trace reads,
non-nil successful-A lease, no poller/ref, and retained cohort membership
(lines 630–660). The helper's count order and values match the required
`R=1,S=1,V=0,U=0,A=1,O=0` case. Existing all-eight-before-wait coverage is in
the frozen `af2df...` fixture and remains unchanged.

`waitForRunObservationFailureState` uses bounded polling while reading the
actual condition under `manager.mu` and `runtime.Gosched`; the other waits are
bounded channel waits. I found no sleep used to manufacture the target race.
The direct reserve/launch and positive claim/continuation ordering are explicit.

The detach-versus-release case (lines 479–544) exercises the production
`releasePollerRef` boundary against lease detach and verifies both membership
maps are clear after the held call joins. Because `prepare` is intentionally
unwired in Phase A, this test does not by itself prove that Phase B routes an
actual selected-A outcome through that boundary; Phase B review must verify
that production wiring. The shared-first-failure test separately covers
successful A cleanup and the lease/count semantics.

## Original-contract differences and limits

I read the full original Unit 1 assignment, lifecycle preregistration,
revision 6 and addendum, initializer/retention corrections, corrected
read-start proposal and its approved three-finding/cardinality supplements,
plus root worker-launch, initial-failure, read-eligibility, and signal/ref-owner
decisions. The three-file source boundary and deliberately unwired Phase A
stubs are consistent with the release; they do not implement the broader
revision 6 `Next`/delta, retention, SSE, or public behavior. The frozen
all-eight coverage remains outside the new fixture as directed. The three
findings above are fixture/readiness defects, not authorization to broaden the
source unit.

No compile, focused RED, typecheck, formatter, scanner, Gitleaks command, Git
mutation, or runtime gate was run. Readiness must be reconsidered only after a
fresh builder repairs the cited defects, publishes exact new source identities
and result bookkeeping, and a different critic reviews that final source.
