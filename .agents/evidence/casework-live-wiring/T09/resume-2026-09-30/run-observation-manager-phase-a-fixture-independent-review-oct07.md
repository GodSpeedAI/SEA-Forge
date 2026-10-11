# Independent Phase A fixture repair review — 2026-10-07

**Verdict: APPROVE this fixture repair for source-review readiness only.** It
resolves the prior three fixture findings within the authorized test file. It
does not claim compilation, expected RED, lifecycle behavior, or T09 completion.

## Preimage, diff, and final identities

I read the complete original Phase A release and its full repair assignment,
the prior independent rejection, the repair assignment, and the repair result.
The repair assignment SHA-256 is
`dc37aa4b06d995431fadaa581ce9a87e85fc62db7514534523bf1d6d81722d3e`; the
repair result SHA-256 is
`68fb3350d6fcacaedefd5be026cad59a6e8f5bad6929b8a8d558c046318bfaa5`. The
previous review's assignment and rejection remain preserved unchanged.

The exact source preimage is archived as
`run-observation-manager-failure-test-preimage-oct07.json`. I decoded its
gzip/base64 `base64` field directly to a hash stream; the result is
`3b0f0653892cb8bcd9823a91779ffbd03e37d015c41242762091bdfad8233990`, matching
the assignment's preimage identity. Read-only unified diff against the current
fixture shows only the three authorized changes. The final fixture is 27,708
bytes with SHA-256
`4ff2fe1c922d6ae7e232d5a58f201298e232807437967fe22a9316dcb45aa0f3`.

The other authorized source and frozen boundaries match:

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

The Phase A source boundary remains exactly manager, worker, and new failure
fixture. Manager and worker remain intentionally unwired stubs; no algorithm
claim follows from this fixture approval.

## Finding-by-finding review

1. **Undefined caller fixed.** The held-actual-read case now binds
   `manager, caller, _, _` from the fixture helper at
   `run_observation_manager_failure_test.go:261`; its later `prepare` call uses
   that declared caller at line 273.
2. **Cancel-field mutation and spy removed.** The held-read test at
   `run_observation_manager_failure_test.go:253-314` now uses the real
   cancellation path and retains the actual held port signal, registered-entry
   check, Stop-not-returned check, release, Stop completion, worker completion,
   post-JOIN removal check, and call-count check. There is no assignment to
   `entry.cancel`, `TryLock` spy, or spy assertion.
3. **Negative eligibility checks added at the deterministic barriers.** The
   Stop-wins test at lines 144-157 calls the real
   `manager.claimPollerReadStart(entry)` after observing `manager.stopping` and
   fails if it accepts the claim, before the real launch batch. The detach-wins
   test at lines 190-203 performs the corresponding negative claim after
   observing `lease.draining` and before launch. The existing positive
   pre-Stop claim remains at lines 231-233 and continues through the mandatory
   worker continuation after Stop.

## Matrix and constraints

All nine original cases remain in the same order, as confirmed by the test
function inventory and diff:

1. Stop between reserve and launch.
2. Detach before first read.
3. Positive read claim followed by Stop and the worker continuation.
4. Stop while an actual trace call is held, then JOIN.
5. Global Stop across distinct pending leases.
6. Canceled Prepare while a distinct authorized lease survives.
7. Detach versus exact selected-A ref release.
8. Shared first-read failure with successful A semantics.
9. Oversized nil-current image with one allowed list and zero trace reads.

The shared first-read case retains owner/waiter attempted-read counts 1/0,
non-nil distinct leases, `R=1,S=1,V=0,U=0,A=1,O=0`, one trace start, exact-ref
cleanup, and JOIN. The oversized-key case keeps the valid bounded case identity,
tests a long `planItemID`, allows exactly its list call, and requires zero
trace reads with no entry/ref/worker and a nonnil successful-A lease. The
existing manager fixture with all-eight-start-before-wait coverage is still
frozen at `af2df...`.

The diff introduces no sleep, callback, generic hook, fake pre-invocation port
barrier, direct lifecycle-field assignment, or new test-only state. Its
`runtime.Gosched` bounded state poll observes actual state under the manager
mutex; channel waits have bounded timeouts. No assertion or case was removed.

## Material differences and limits

Compared with the original Phase A release, the repair changes only the fixture
to implement the three cited review corrections. It does not alter the
compile-safe stubs, implement lifecycle behavior, change the nine-case matrix,
modify the frozen manager fixture or six primitives, or broaden the three-path
release. No other material instruction difference was found.

No compiler, focused RED, formatter, scanner, Gitleaks, Graft build, or Git
mutation was run. Root must separately authorize the focused expected-RED
execution after this source review. Phase B still requires its own source
release and independent review.
