# Run observation hydration cap: final runtime review

Date: 2026-10-06

## Verdict

The approved Go implementation passed the four assigned runtime gates. Each actual test process was joined, returned captured test exit 0, and used the frozen source and fixture. This establishes the focused cap behavior, the complete Go server package, the Go module package set, and the repository's `casework-go-check` recipe under the recorded limits. It does not establish Rust/SFWP integration, a manager call site, live service behavior, or the wider T09 workflow.

Frozen production source SHA-256: `3dfa2993004676a13b1dd9c449965c3b531a199885db325a6fd14941a90f501a` (`apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go`).

Frozen fixture SHA-256: `0e3b1774303fa6b155269e45adb0703262b789611a81993d4c2b63bd9e8140f9` (`apps/godspeed-casework-go/internal/server/run_observation_hydration_cap_test.go`). Each preflight captured both hashes. Gates 2–4 also have direct post-run hash checks with the same values; the focused gate's immediately following gate-2 preflight records both unchanged hashes.

All Go commands used `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1`. The canonical recipe additionally used `JUST_TEMPDIR=/tmp`. Resources below are the command-namespace `/proc/meminfo` and `/tmp` filesystem readings in each fresh preflight, not a claim about global host capacity.

## Joined gates and immutable captures

| Gate | Actual command / session | Result | Preflight resources and source hashes | Captures (`/tmp` original ↔ repository archive; all `cmp` exact) |
|---|---|---|---|---|
| Focused cap | `go test -race -v -count=1 -parallel=1 -timeout=90s ./internal/server -run '^TestBoundRunObservationHydration'`; session `72410` joined | Inner test exit `0`. All seven top-level tests and their nested cases passed, including oldest-frame eviction, timestamp/offset and full-ID tie ordering, metadata/count preservation, caller-data non-aliasing, and all invalid-input cases. The positive post-success assertions were reached. | Fresh preflight: available RAM `2812 MiB`, free swap `2999 MiB`; cache present. Captured source/fixture hashes match above. | `/tmp/sea-casework-20261006-hydration-cap-green-01-{preflight,run,exit}.raw` ↔ `run-observation-hydration-cap-green-01-{preflight,run,exit}.raw`. SHA-256: preflight `76385d74d570bbd73b35b37617aaf02a285c09add46c89b70b6bdd1cf4fd39ab`; run `1f1f335202e17ea3bd4c1416eacd642d48800f31adbd39b86c3184c0e99ad5c9`; exit `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`. |
| Full server package | `go test -race -count=1 -parallel=1 -timeout=180s ./internal/server`; session `29681` joined | Inner test exit `0`; `internal/server` passed in `19.049s`. | Fresh preflight `2026-10-06T15:58:52Z`: `MemAvailable=2806384 kB`, `SwapFree=3098924 kB`, `/tmp` available `1346949120` bytes, cache present. Direct post-run hashes match frozen source and fixture. | `/tmp/sea-casework-20261006-hydration-cap-server-01-{preflight,run,exit}.raw` ↔ `run-observation-hydration-cap-server-01-{preflight,run,exit}.raw`. SHA-256: preflight `f8acbcdd0d84b71995b7911124951a5b65d022fe2c0032f7a6ba22bcc0451ecc`; run `32d23b60e00ed835d79ca08cc846921b90a5f9eac18bd2e892822372b507afcf`; exit `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`. |
| Full Go module | `go test -race -count=1 -parallel=1 -timeout=180s ./...`; session `42743` joined | Inner test exit `0`; every module package passed, including `internal/server` in `14.830s`. | Fresh preflight `2026-10-06T15:59:58Z`: `MemAvailable=2834056 kB`, `SwapFree=3088448 kB`, `/tmp` available `1346928640` bytes, cache present. Direct post-run hashes match frozen source and fixture. | `/tmp/sea-casework-20261006-hydration-cap-module-01-{preflight,run,exit}.raw` ↔ `run-observation-hydration-cap-module-01-{preflight,run,exit}.raw`. SHA-256: preflight `ebd5894120d1a7bfc041049a7ed35b39170dcb30b6a33fe22be355bf3acd708e`; run `2ade2dbc1c3ec0f923759f91afedf6114e2e3d0eaa0d8905b590cddc2917ec0c`; exit `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`. |
| Canonical recipe | `just casework-go-check`; session `54587` joined | Inner recipe exit `0`; format, vet, and tests green. The output reports Go packages, including `internal/server`, and ends `casework-go-check: format, vet and tests green`. | Fresh preflight `2026-10-06T16:02:00Z`: `MemAvailable=2846832 kB`, `SwapFree=3092284 kB`, `/tmp` available `1344335872` bytes, cache present. Direct post-run hashes match frozen source and fixture. | `/tmp/sea-casework-20261006-hydration-cap-canonical-01-{preflight,run,exit}.raw` ↔ `run-observation-hydration-cap-canonical-01-{preflight,run,exit}.raw`. SHA-256: preflight `b3df4afed700df6ff93e2f746d9ce6b03547fe859fd4f29b22a0cc527edab03a`; run `a9b119508d00f369796553295f48cd22881c2a172c333100618757c2a993d903`; exit `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`. |

The full server and module gates used the root-authorized local-only elevated sandbox after a prior observed local-listener sandbox denial. The canonical recipe used `JUST_TEMPDIR=/tmp` to avoid the previously observed scratch-directory issue. These gates did not use external network access.

## Capture correction

The first native archive patch for the focused run was rejected because it omitted one recorded subtest line. The archive was then corrected from the actual joined `/tmp` capture. A subsequent `cmp` passed for the full output and exit; the immutable hashes above are for the exact matching files. No source or fixture was modified.

## Limits

These checks do not prove integration with Rust/SFWP or a live subscription/manager path. The helper remains a private sizing/trimming boundary over a well-formed DTO, with non-nil arrays and valid enum values supplied by its future caller; no production manager call site was approved or exercised here. No full T09, CI, publication, or deployment claim is made.
