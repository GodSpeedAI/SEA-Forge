# Run observation hydration cap — focused assertion RED runtime review

Date: 2026-10-06  
Verdict: **EXPECTED STUB RED CONFIRMED, WITH ASSERTION BOUNDARY LIMITED AS BELOW.** This verifies only the fixture’s positive behavior is distinguishable from the frozen throw-only stub. It is not an implementation or green runtime approval.

## Frozen source identities before and after the gate

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go` | `1ebfa2e2a002b3c86ca4fd8b599e8b572929feb8ebaa192326658b3c3b8227d7` |
| `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap_test.go` | `0e3b1774303fa6b155269e45adb0703262b789611a81993d4c2b63bd9e8140f9` |
| `run-observation-hydration-cap-phase1-final-source-review-oct06.md` | `bdffeafb52090441091461f67268cadccd81843010b002cb134c0bf499fe0b5b` |
| `run-observation-hydration-cap-original-assignment-oct06.md` | `ed68ba884fac5673d6b8f6093612a3f3e2c342176beaf03a8cf88cd8dccce37d` |
| `run-observation-hydration-cap-fixture-repair-original-assignment-oct06.md` | `3ce3a79b6d38c0695ed85482d577460fb0bf0842a13703e6f7de145ae4afe827` |

Source/test hashes match the accepted source review and fresh preflight; no source or test was changed during this gate.

## Preflight and exact command

Fresh preflight at `2026-10-06T15:42:35Z` reported `MemAvailableMiB 2340`, `SwapFreeMiB 3359`; `/tmp` had 1,354,391,552 bytes free and the assigned GOCACHE directory existed. Both frozen source hashes were included. The three raw files below were archived by native `apply_patch` and compared byte-for-byte with their original `/tmp` captures; all `cmp` commands succeeded.

Working directory: `apps/godspeed-casework-go`. Exact test command and environment:

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 \
  go test -race -count=1 -parallel=1 ./internal/server -run '^TestBoundRunObservationHydration' -v
```

The process ran as exec session `31028` and was joined. The wrapper’s tool exit was zero after capture; the actual `go test` exit written to the exit file is **1**.

## Observed assertion boundary

The raw output enters all eight top-level hydration test functions and all five negative-input subtests. Each of the five negative-input subtests passes against the stub because it expects a typed `KindUnavailable` and empty result; these passes do not prove those individual validation rules.

The six successful-output cases fail immediately at `requireHydrationSuccess` with the exact typed-unavailable stub error: empty/short preservation; exact cap; cap-plus-one trimming; each of the three deterministic tie/order subtests; natural multi-run trimming; ring-omission preservation; and no-alias short success. (The function count includes grouped subtests; raw output is authoritative.) Therefore the tests distinguish the positive behavior from the stub as intended, but no post-success output, count, deterministic-retention, or immutability assertion was reached. In particular, this run does not establish those behaviors, nor does it establish a runtime implementation pass.

There was no setup error, compile/type error, race report, panic, or timeout. Failure is the expected assertion-level failure against the explicit unavailable stub.

## Immutable captures

Each archived repository capture is byte-identical to the original `/tmp` file:

- Preflight: `run-observation-hydration-cap-focused-red-01-preflight.raw`, SHA-256 `f8a245124d200cea2d548dcede58354cbdd72bb687e04e45d23ad239d869bf4a`.
- Joined actual stdout/stderr: `run-observation-hydration-cap-focused-red-01-run.raw`, SHA-256 `bf83d3026f53205af6547d2b6491a25554c9408d67f05d7f938181c75ffe63cd`.
- Actual command exit: `run-observation-hydration-cap-focused-red-01-exit.raw`, contains `1\n`, SHA-256 `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865`.

Original captures are at `/tmp/sea-casework-20261006-hydration-cap-focused-red-01-preflight.raw`, `/tmp/sea-casework-20261006-hydration-cap-focused-red-01-run.raw`, and `/tmp/sea-casework-20261006-hydration-cap-focused-red-01-exit.raw`. Repository archive paths are in `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/` with the same basenames. No broader server/module/canonical gate, production implementation, Graft build, or integration claim is included. Root retains implementation and further runtime gate ownership.
