# Initial lease bookkeeping TDD source review

## Verdict

**SOURCE READY for the separately authorized focused RED gate only.** This is a
static review of TDD scaffolding. It establishes no compile, test, or runtime
result and does not approve implementation of lease initialization/seeding.

## Inputs and identities

The full assignment is
`run-observation-initial-lease-bookkeeping-assignment-oct08.md`
(SHA-256 `2be0bcd6c5f7f043e482716095df32409b0147dac3f920162cd8dd76de659e03`).
I reviewed the original source result
`run-observation-initial-lease-bookkeeping-tdd-result-oct08.md`
(`852306415c9e178ed56ab34def1e3bfc314633169cf834a1ce64642df6d6c1af`), its
identity correction (`run-observation-initial-lease-bookkeeping-tdd-result-correction-oct08.md`,
`8e6ee2e7f54c7e5c8a64c61cdb2cd3c0cec0d24f42ea3896047e6d8defcf05d9`), and
the fresh two-fix result
(`run-observation-initial-lease-bookkeeping-two-compile-fix-result-oct08.md`,
`6368af856ca6e47566fdeacfec36b0b0fb95c36d93a64e723d2d033ff7405c22`).

Independently recomputed current hashes:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `e2b7709410ff4766503a52580e0cdec8b9eaf189f695bbb75ae87b2dbbdf3285` |
| `run_observation_manager_test.go` | `08852d598e91264a81a745ac9ba08b31e7eeb5cdb60dd01324e578967f260f50` |
| frozen `run_observation_delta.go` | `5241e656d65f86a315e37df44b48d5eaae865c92cf80cc306c9bf4fc0fd7e128` |
| frozen `run_observation_delta_test.go` | `48c6d28e658b8bd4e8fd3c6650ff26c73b0c791f35feff83d056ff8f501c89ed` |
| frozen `run_observation_poller_worker.go` | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` |

The correction matters: the initial result's manager-test hash was superseded
after trace-call assertions were added, and its “six frozen” statement was
inaccurate; it lists three frozen delta/worker files. The fresh repair changes
only the test: `strings.Repeat` is assigned to a runtime variable, and one
unused local key is removed. This review uses the final identities above. No
other source change is claimed.

## Source findings

`run_observation_manager.go:81-92` contains exactly the two approved new
declarations: `listState contract.RunTraceListState` and
`watermarks map[runObservationPollerKey]runObservationWatermark`. Searching the
manager file finds no other access to these fields; neither constructors nor
`prepare` initializes or seeds them. Thus the builder has not included
implementation behavior in the compile scaffold.

The complete-empty test reads both fields under `manager.mu` and requires
`complete` plus a nonnil empty map (`run_observation_manager_test.go:319-328`).
The refused-list test similarly requires `unavailable` plus a nonnil empty map
(`:1188-1197`). Their prior DTO, call-count, and detach assertions remain.

The new test drives two real `manager.prepare` calls. Its first list contains
eight distinct runs; each fake port response is an accepted terminal snapshot
with 1,024 frames (`run_observation_manager_test.go:358-413`). The count is at
the valid per-run limit (`run_observation_retained_version.go:14-17`), and the
candidate builder accepts terminal accepted standings and checks key/frame
validity (`:70-115`). The resulting DTO must contain fewer than the original
8,192 frames (`run_observation_manager_test.go:431-437`), so the test exercises
actual aggregate hydration pruning. It then copies the lease map under lock and
checks all scalar fields against the expected generation-1 accepted capture
(`:439-473`). A second actual Prepare uses a different key; the first map must
remain eight entries, and the second must contain only its own key
(`:475-499`). Trace counts assert eight initial reads and exactly one
additional read (`:425-429,482-483`). There is no manual map population,
sleep-based scheduling, or test hook.

Cleanup is registered before Prepare (`:419-420`). The existing cleanup helper
snapshots worker ownership under the mutex, cancels outside it, performs
bounded stop/drain, and waits for actual worker completion
(`:178-259`). Successful paths explicitly detach both leases (`:501-506`).
The equality assertion is statically valid because the watermark and nested
window contain only scalar/enum fields (`run_observation_delta.go:9-15,40-46`).
The two prior compile defects are exactly the ones repaired in the fresh
result; no compiler was run to independently establish compilation.

No material deviation from the full TDD assignment remains. The fields are
intentionally uninitialized, so their assertions remain expected RED until a
later implementation grant. This verdict authorizes only the actual focused
RED gate, not GREEN or T09 completion.
