# Independent review: pure run observation projection

Date: 2026-10-08. Verdict: **APPROVE the private per-run projection unit as
source-ready**, bounded to the pure assembler, its direct tests, and the Go
verification recorded below. This does not approve manager/lease integration,
public wiring, retention policy, full Next behavior, or T09 completion.

## Authority and reviewed identities

Reviewed the full original unit assignment
(`run-observation-next-projection-assignment-oct08.md`, SHA-256
`c6bd03f372140f0542f92c3c15a7e6b3f2471d311ef9bd705f0a2acc3c832c9d`), full
root clarification (`run-observation-next-projection-root-clarification-oct08.md`,
`1a3685eb42c287c54ecbe09830827fbe6edce85aa96349b0f9f724269c526fee`),
revision 6 proposal (`60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`),
revision 6 addendum (`9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`),
and retention initializer correction revision 3
(`b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`). I also
read the complete fixture repair and over-cap repair grants/results, algorithm
builder result, guard result, formatting result and its identity erratum, prior
independent fixture/format reviews, and capture-provenance erratum.

Current source identities:

| File | Bytes | SHA-256 |
|---|---:|---|
| `apps/godspeed-casework-go/internal/server/run_observation_delta.go` | 5,599 | `5241e656d65f86a315e37df44b48d5eaae865c92cf80cc306c9bf4fc0fd7e128` |
| `apps/godspeed-casework-go/internal/server/run_observation_delta_test.go` | 15,263 | `48c6d28e658b8bd4e8fd3c6650ff26c73b0c791f35feff83d056ff8f501c89ed` |

All eleven frozen neighboring source/test files match the TDD preparation
identities: manager `22005e5c…`, poller worker `b0fde3fe…`, authority terminal
test `889f9648…`, manager test `cf7188c2…`, manager failure test `8c71197a…`,
key `a6f0114d…`, retained version `2157583f…`, retained version test
`34df05f1…`, retained policy test `e156c9cf…`, poller image `3dba418f…`, and
manager retained image test `cf501bf7…`. The preflight for full-race02 records
the complete hashes.

## Requirement review

The private seam takes only the captured retained state, copied ledger, and
prior scalar watermark (`run_observation_delta.go:48-52`). Its body reads only
those arguments. It does not read manager maps, recapture, retry, or mutate
caller state.

| Grant requirement | Source and test evidence |
|---|---|
| Reject non-current, malformed, over-cap, or `P > H` input with typed unavailable and zero outputs | Guard and shared zero-output constructor at `run_observation_delta.go:53-57,170-172`; malformed oracle at `run_observation_delta_test.go:235-292`; over-cap oracle at `294-321`. |
| Emit retained frames only when first ordinal exceeds P, in captured source order; preserve opaque IDs | Iteration and P filter at `run_observation_delta.go:92-110`; source-order/P/H and opaque IDs at `run_observation_delta_test.go:24-81`. No identifier parsing or delivery ordering occurs. |
| Deep-copy optional pointers and leave inputs unchanged | Pointer value copies at `run_observation_delta.go:111-123`; mutation/immutability oracle at `run_observation_delta_test.go:181-211`. |
| Derive exact window from captured generation, total, retained length, omitted difference, and truncation | `run_observation_delta.go:82-88`; exact field assertions in the first and window-change tests at `run_observation_delta_test.go:52-57,119-179`. |
| Preserve captured identity, timestamp, and standings; advance candidate watermark to captured H | Delta and candidate construction at `run_observation_delta.go:125-142`; assertions at `run_observation_delta_test.go:43-81,144-179`. |
| Build gaps only from the copied ledger for P < ordinal <= H, absent from captured window; deterministic exact-ID sorting; flags true | Filter, captured-window membership, sorting, count, ordinals/window, and flags at `run_observation_delta.go:144-167`; exact gap oracle at `run_observation_delta_test.go:63-70`. Later IDs above H are intentionally ignored at `run_observation_delta.go:67-70,145-147`, tested at `run_observation_delta_test.go:83-117`. |
| Permit standing/window-only updates, reappearance with immutable first ordinal, and successful empty terminal state | Tests at `run_observation_delta_test.go:119-179,213-233,323-341`. |

The base assignment's complete expected behaviors are represented in these
tests. The clarifying malformed-input cases are present: nil state, negative
and impossible totals, `P > H`, missing/disagreeing ledger entries, frames
above H, invalid/unavailable markers, and zero outputs. Clarification also
allows later copied-ledger entries above H; the implementation and test honor
that boundary. Gap flags follow the clarification: both are true for every
reported gap, and the count is only the count of exact observed IDs.

## Explicit material deviations and limits

1. **Additional copied-ledger structure validation.** The algorithm checks
   that captured ledger ordinals are nonzero, unique, and have cardinality H,
   which implies the full `1..H` range (`run_observation_delta.go:60-80`). This
   is stronger than the initial algorithm description. It is consistent with
   the accepted retained-state invariant: the producer copies the exact
   entry-lifetime ledger, assigns the next ordinal only to an unseen ID, and
   retains all prior IDs (`run_observation_retained_version.go:108-142`).
   Entries above H remain allowed. The malformed test cases at
   `run_observation_delta_test.go:250-276` exercise missing/inconsistent
   ordinal input and the implementation rejects it with zero outputs. They do
   not isolate the duplicate-ordinal branch with a complete `1..H` ledger; that
   narrower branch-coverage gap is noted, but the invariant and rejection are
   directly verifiable from source.

2. **Retained-frame cap guard.** The initial pure-projection assignment did
   not state the 1,024-frame limit as a separate bullet. The producer rejects
   windows above this limit (`run_observation_retained_version.go:14-16,82-84`),
   and the separately reviewed over-cap grant required the assembler to fail
   closed when given such a malformed captured state. The sole guard is at
   `run_observation_delta.go:53-56`. Its coherent 1,025-frame oracle requires a
   typed unavailable error and all-zero outputs at
   `run_observation_delta_test.go:294-321`. The fixture's pre-guard expected
   failure was archived as a separate actual RED before the guard grant.

3. **Fixture and formatting repairs.** The six settlement literal fixes from
   `settled` to allowed `accepted` are the complete fixture-only change
   authorized by the fixture repair grant. The later test format change is
   whitespace-only; its result plus identity erratum establish the decoded
   preimage and exact formatter output. These changes introduce no additional
   algorithm semantics.

The 1,024-frame cap is an existing source invariant. This review does not
approve proposed private lease/cohort limits or the proposed 1 MiB retained
image policy. No integration or public interfaces were changed or reviewed as
part of this unit.

## Runtime evidence and capture provenance

The focused race gate passed:
`GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 -json -run '^TestRunObservationDelta' ./internal/server`.
Its six-file lossless packet is
`next-projection-race01-critic-capture-oct08.json`, SHA-256
`2e2d833050a1203980833fcb8b3038b0489a0ad1d0d0f0e4c6e29a4cb22c534f`; root
independently decoded and compared the originals.

Canonical `JUST_TEMPDIR=/tmp GOMAXPROCS=2 GOFLAGS=-p=1 just casework-go-check`
passed format, vet, and tests. Its packet is
`next-projection-canonical02-critic-capture-oct08.json`, SHA-256
`4fb8b0f434c06d21f84d9f4d4ef7388943e9ef91ed39f03688342ceaeacf1458`; root
independently decoded and compared all six entries.

The first full-module race attempt's `exit.raw` is not child-status evidence
because its shell wrapper negated the command before reading `$?`. The exact
originals remain preserved in `next-projection-full-race01-critic-capture-oct08.json`
(`891037c4233bf0745ec5eb820fded3c403dba94f59499db65acfdb610d7948ad`), and
`run-observation-full-race-capture-erratum-oct08.md` (`a11f3845…`) records the
CW26 capture-provenance recurrence. That attempt is not counted as a passing
gate.

The corrected full-module race run used Python `subprocess.run` and preserved
its actual child return code. `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1
-json ./...` exited 0; the stream contained 738 pass, 5 skip, and 0 fail events,
with 10 tested packages passing and five packages skipped. The exact six-file
packet is `next-projection-full-race02-critic-capture-oct08.json`, SHA-256
`d1a6fce0bafbdbc6e770cbd308ac2efee56269484a7859857819807f52c68bd3`. I decoded
and byte-compared every entry; root independently repeated that comparison.

These checks support the pure per-run projection unit only. No test or review
here proves atomic manager capture/commit, per-lease concurrency, cancellation,
authorization, public response assembly, Next lifecycle, or T09 completion.
