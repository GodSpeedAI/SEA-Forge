# Isolated primitive canonical gate result — 2026-10-07

## Outcome

Canonical verification is **blocked on formatting**. `just casework-go-check` exited 1 and reported that `internal/server/run_observation_retained_policy_test.go` would be rewritten by `gofmt`. The full-module race gate was not run, as required when canonical Gate 1 fails. This is not a source approval or full-go-verification result.

## Scope, preflight, and command

Ran only in the isolated detached worktree `/tmp/sea-casework-primitives-74f86f0-oct07` at `HEAD=74f86f0964f92c5b1715eb0703a9b880d28cccae`. The fresh preflight at `2026-10-07T18:54:31Z` recorded `MemAvailableKiB=3072068`, `SwapFreeKiB=10744696`, no competing compiler process, and exactly the six proposed untracked source paths. All six isolated hashes matched their approved values, and all six isolated files compared equal to the current primary originals. Preflight exit status was 0.

Exact command:

```text
JUST_TEMPDIR=/tmp GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 just casework-go-check
```

The actual 355-byte output contains a mise tracking-config warning about a read-only symlink, followed by:

```text
casework-go-check: gofmt would rewrite:
internal/server/run_observation_retained_policy_test.go
error: recipe `casework-go-check` failed with exit code 1
```

The command exited 1. The observed output does not report vet or test results; neither is claimed as having passed. The formatting failure must be reviewed/repaired by a fresh source builder before a new preflight and canonical attempt.

## Actual captures and comparisons

All actual captures were written in `/tmp/sea-primitives-canonical-89lY2p`. Each immutable archive below was created from the original capture and `cmp` returned 0:

| Actual original | Archive | Bytes | SHA-256 | cmp |
| --- | --- | ---: | --- | ---: |
| `/tmp/sea-primitives-canonical-89lY2p/preflight.raw` | `run-observation-primitives-canonical-preflight-oct07.raw` | 2299 | `ecd067d5834be433e96c7299aeaffe60f86bbbdb1e9b80de96477bd693a2a9ea` | 0 |
| `/tmp/sea-primitives-canonical-89lY2p/preflight.exit.raw` | `run-observation-primitives-canonical-preflight-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | 0 |
| `/tmp/sea-primitives-canonical-89lY2p/output.raw` | `run-observation-primitives-canonical-output-oct07.raw` | 355 | `71023298f18180d94641ed5522cb907c5a875c3f498567cb779e138336e16067` | 0 |
| `/tmp/sea-primitives-canonical-89lY2p/exit.raw` | `run-observation-primitives-canonical-exit-oct07.raw` | 2 | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | 0 |

No source or test file was modified by this verification. No Gate 2, other build, test, formatter, scanner, or Git mutation was run.
