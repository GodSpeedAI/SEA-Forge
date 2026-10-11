# Initial lease bookkeeping: final independent evidence review

## Verdict

**APPROVE the bounded initial lease bookkeeping unit.** The approved scope is the initial Prepare lease state and its scalar watermark snapshot. The result does not establish Next/wake/drain behavior, C2 reader runtime, public live wiring, settlement, or completion of T09.

## Governing instructions and reviewed source

I reviewed the full original assignment `run-observation-initial-lease-bookkeeping-assignment-oct08.md` (SHA-256 `2be0bcd6c5f7f043e482716095df32409b0147dac3f920162cd8dd76de659e03`), the implementation grant/result `run-observation-initial-lease-bookkeeping-implementation-result-oct08.md` (`38a1ee193299cf8be841234423ddb655ed51154f7eff4d9dff2d46a4852f18fc`), and both independent source reviews (`run-observation-initial-lease-bookkeeping-independent-source-review-oct08.md`, `ceea832caafc792bb26f4d7f6930dd6a549a3151d85290385dde8440a2340f6f`; `run-observation-initial-lease-bookkeeping-implementation-independent-source-review-oct08.md`, `0e1c2222c941ea0cb2c9f365722ff5f61532d456a9037bdb1c26c6132780474f`). I also reviewed the full formatter result (`initial-lease-gofmt-only-result-oct09.md`, `3db2d5ca61c4dcb063336aa3caad0e352ef68bfc1fca25860d248600381481e2`) and independent formatter review (`initial-lease-gofmt-only-independent-review-oct09.md`, `36ce0910a6643238dddf01cb4da8deec17e1ba0c0cfd8d338d1e464270dca2ab`).

The source implements the specified behavior in [run_observation_manager.go](apps/godspeed-casework-go/internal/server/run_observation_manager.go): each lease initializes its own watermark map (lines 123–130); successful and refused list outcomes set `listState` under the manager mutex (397–436); accepted immutable current states are seeded only after active-cohort, manager-entry, forward-reference, reverse-reference, phase, key and availability checks in the same critical section (537–553); and the conversion copies only scalar standing and window fields (678–694). Existing final authorization/disclosure checks remain in place. No frame, pointer, event ID, duplicate identity ledger, public field, dependency, budget, or unrelated lifecycle behavior was added.

The focused fixture in [run_observation_manager_test.go](apps/godspeed-casework-go/internal/server/run_observation_manager_test.go) exercises the actual Prepare path. It verifies accepted complete-empty and refused-list lease state (1166–1198), then uses eight fitting terminal captures to cross the aggregate hydration budget and proves DTO pruning while all pre-hydration scalar watermark fields remain exact; a second Prepare proves lease maps remain separate (358–507). The existing assertions and cleanup remain present. I found no material implementation or fixture deviation from the full assignment.

## Independent gate evidence

I independently ran and parsed these three gates after the final source identities were frozen. Each preflight recorded HEAD `e33bf30ee33d882c29595c31190dc899efbadc2a`, the same eleven source hashes below, adequate RAM/swap, and no competing heavy process. Root acknowledged lossless comparisons for all six captured files per gate; I independently decoded each archive and compared it to the original temporary file (18/18 comparisons equal).

| Gate | Actual command | Result |
|---|---|---|
| Focused race | `go test -race -count=1 -json -run '^TestRunObservationManager' ./internal/server` | Exit 0; 33 top-level and 6 nested tests passed, 0 failed. The complete-empty Prepare, hydration/watermark, and refused-list tests all passed. |
| Canonical Go | `just casework-go-check` | Exit 0; canonical format, vet, and Go tests passed. |
| Full module race | `go test -race -count=1 -json ./...` | Exit 0; 729 test cases passed (308 top-level, 421 nested), 0 failed, across 10 tested packages; 5 packages reported no test files. |

The corresponding immutable raw captures are:
- `initial-lease-critic-green03-{command,preflight,preflight-exit,stdout,stderr,exit}-oct08.raw.json`
- `initial-lease-critic-canonical06-{command,preflight,preflight-exit,stdout,stderr,exit}-oct08.raw.json`
- `initial-lease-critic-fullrace06-{command,preflight,preflight-exit,stdout,stderr,exit}-oct08.raw.json`

The originals were `/tmp/initial-lease-critic-green03-583a34kn`, `/tmp/initial-lease-critic-canonical06-yjxapiy3`, and `/tmp/initial-lease-critic-fullrace06-mbirn66f`. Each gate used the recorded bounded Go environment (256 MiB Go memory limit, GOGC 50, two Go scheduler threads, package parallelism 1, count 1). Earlier resource/setup holds and failed attempts remain in their own evidence; they are not counted as gate results. Full-module race is the app-wide regression evidence for this unit, not a claim that the larger T09 or cross-application workflow is complete.

## Source identities

These identities are machine-derived from the final full-race preflight's `source_sha256` object and match the focused and canonical preflights as well.

| Source path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_delta.go` | `5241e656d65f86a315e37df44b48d5eaae865c92cf80cc306c9bf4fc0fd7e128` |
| `apps/godspeed-casework-go/internal/server/run_observation_delta_test.go` | `48c6d28e658b8bd4e8fd3c6650ff26c73b0c791f35feff83d056ff8f501c89ed` |
| `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go` | `3dfa2993004676a13b1dd9c449965c3b531a199885db325a6fd14941a90f501a` |
| `apps/godspeed-casework-go/internal/server/run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `76e0dcc20e87007248cec3bf5ffe9a56a997b75c4ab2d2fb960cbbbaf0978d52` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_authority_terminal_test.go` | `889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go` | `6e3a3315e4f92e277ff302a6bc17c06cdbcb5b128d4c234922b9f2320ca249cb` |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |

## Formatting provenance and limits

The two current Go files were independently verified to equal standard `gofmt` output for the exact reconstructed preimages; the canonical gate also passed its formatting step. The formatter builder's historical raw formatter stdout/exit captures are absent, so this review does not authenticate that historical process record. That provenance limit does not change the independently reproduced byte-equivalence result.

No additional source edits, format writes, tests, or Git changes were made for this review. This is a bounded approval of initial lease bookkeeping and its specified regression gates only.
