# Independent review: direct-capture Unit 1 / retained-helper RED

Date: 2026-10-07  
Disposition: **APPROVE the bounded current-candidate semantic RED evidence only**

## Scope and identities

Independently reviewed the full direct-capture assignment and result; full manager Unit 1, retained-helper, policy-fixture, and policy-repair instructions; current manager/helper/base-test/policy source; all three preflight/output/exit triples; and all nine surviving `/tmp` originals. No tests, compiler, scanner, source edits, or Git command were performed.

| Artifact | SHA-256 |
|---|---|
| Direct-capture assignment | `f36b914632eee7a64655ff64ec9d4a1e2783cd8a3ccf31533b1c7b9b56ada51f` |
| Root result | `3a859305992c6bd7d8e3a60b35d3dbd482a02c3164ff0c949a8e5fc9088214a3` |
| Manager source/test | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` / `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| Retained helper/base test | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` / `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| Retained policy test | `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7` |

Each candidate hash matches the corresponding current file and appears identically in all three command-time preflights. The commands ran serially with the specified resource controls, race detector, one test process, and 60-second timeout. All three numeric exits are 1, the expected result for test failures against the deliberate stubs.

## Independent original-to-archive verification

I compared every `/tmp` original to its durable archive with `cmp -s`; all nine comparisons returned exit 0. SHA-256 was independently computed for both sides of every pair, and hashes match:

| Kind | Preflight | Output | Exit |
|---|---|---|---|
| manager01 | `22e93049d5d1d48a54608dde5d601c4dd7bde42249a97b230705a85f0d20c891` | `a77106c2274f13d8283ec9c4b17bb04c7eabc0371ef46afd5cae2672f41cc0e3` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |
| helper01 | `fe33ad437a735d02ead327300a1385683956f989a72102eb47e1324e02585be3` | `f246fd38ecad8df162f4dfd2e1cf2c395f46a65361b460c08fe51e5c809c2868` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |
| policy01 | `4b3c84a3f691fe5526e93e1083a3c4d1a3ccb273f48079d11f15a308e6a2afcf` | `292eae9179b9353b616d1cdc1998ae9dd4c756b8e22f4fa8acae28e5b5ccdc8b` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |

The raw preflights, outputs, and exits are under `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/go-direct-red-{manager01,helper01,policy01}-{preflight,output,exit}-oct07.raw`; originals are `/tmp/sea-go-direct-red-{manager01,helper01,policy01}-{preflight,output,exit}-oct07.raw`. This provenance is directly verified for these nine files. It does not retroactively validate the earlier transcription-only capture set.

## Preflight and command review

The three original preflights report UTC time, all five current source/test hashes, full exact command, available memory, and free swap:

| Attempt | Time UTC | Available memory | Free swap | Selector |
|---|---|---:|---:|---|
| manager01 | `2026-10-07T13:05:12.471Z` | 3,804 MiB | 11,929 MiB | `^(TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList|TestRunObservationManagerRefusedListReturnsUnavailableDTOAndEmptyLease|TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry)$` |
| helper01 | `2026-10-07T13:06:29.114Z` | 3,745 MiB | 11,930 MiB | `^TestRunObservationRetained(Candidate|Image)` |
| policy01 | `2026-10-07T13:07:05.891Z` | 3,749 MiB | 11,930 MiB | `^TestRunObservationRetainedPolicy` |

Each command is the assigned `go test -race -count=1 -parallel=1 -timeout=60s ./internal/server` with `GOMEMLIMIT=256MiB`, `GOGC=50`, `GOMAXPROCS=2`, `GOFLAGS=-p=1`, `GOLDEN_UPDATE=0`, the fixed explicit Go cache, and `CARGO_BUILD_JOBS=1`. The output package path is the expected `apps/godspeed-casework-go/internal/server`. Each output shows the selected test binary ran and the package completed with ordinary test `FAIL`; none shows compile/setup failure, race warning, timeout, panic, or cleanup failure.

## Reached semantic assertions and limits

**manager01:** The stale-claim refusal path reaches its expected assertions, then the corrected-claim retry fails at `run_observation_manager_test.go:971` on `run observation manager unit 1 is not wired`. The refused-list test fails at `:1016` on the same stub rather than producing the unavailable DTO/empty lease. The canceled-partial-Prepare test fails at `:1099` before the controlled read starts. These are the intended stub-semantic failures; the run did not verify list refusal assembly, cancellation, read join, rollback, retry, or any worker lifecycle.

**helper01:** All eight selected base helper tests reach their first behavior assertion and fail on the generic rejected/nil stub outcome: `run_observation_retained_version_test.go:95,117,148,231,254,329,435,468`. The exact selected test names and lines match the direct output. Later assertions in those tests are unreached; this run does not verify pointer copying, ordinals, refusal/recovery, terminal retention, byte accounting, window limits, or overflow implementation.

**policy01:** The output reaches all five top-level policy tests and their failing cases: ordinal nonterminal/terminal at `run_observation_retained_policy_test.go:97`, generation overflow at `:116`, all five execution/settlement rows at `:158`, both final-marker rows at `:188`, accepted source-count case at `:203`, and read-unavailable recovery at `:241`. Each fails against the helper stub as intended. `t.Fatalf` means the policy test's later key mismatch and invalid-count cases after recovery are not reached. No policy behavior beyond the first expected stub assertion in each case is proven by this run.

The result accurately distinguishes semantic failures from setup/compiler failures and avoids claiming later assertions or lifecycle behavior. These current direct captures supersede only their three assigned selectors; they do not establish full package GREEN, helper correctness, lifecycle correctness, production integration, `Next`, SSE, or T09 settlement. The prior manually transcribed six-file evidence remains unapproved and is not relied upon here.

## Verdict and release boundary

Approve this verified current-candidate evidence for the narrow claim that the three selected manager tests, eight base helper tests, and five policy tests compile and reach their intended semantic failures against the unchanged unwired manager and rejected helper stubs. This satisfies the source-reviewed focused RED prerequisite for the next separately authorized private algorithm step. It does not approve the algorithm, lifecycle implementation, any unreached assertion, runtime behavior, manager integration, public/API changes, or settlement. Root retains the implementation release and next compiler authorization.

No new Go run or test was performed in this review. Graft retrieval saved ~47,911 tokens (~$0.04) this turn.
