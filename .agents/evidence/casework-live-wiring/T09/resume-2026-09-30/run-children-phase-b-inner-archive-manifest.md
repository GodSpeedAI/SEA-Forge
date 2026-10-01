# Phase B narrow Go evidence archive manifest

Recorded 2026-10-01. This manifest indexes immutable copies of the original
`/tmp/t09-phaseb-inner-*` preflight and focused race output files. Every copied raw
file's byte count and SHA-256 was checked against its `/tmp` original after archive.
The original files remain untouched. Preflight wrapper exits were observed as 0, but
the wrapper did not save separate preflight `.exit` files; no exit artifact is inferred.

## Preflights

All five commands performed read-only host RAM, cgroup memory (when readable), Go/compiler,
Cargo/Rust, and Bun process-name scans; verified the existing `target/debug/sea-forge-server`
executable; and saved stdout to the listed original path. The wrapper reported exit 0 for each.
Preflights 01–02 ran from the repository root; 03–05 ran from `apps/godspeed-casework-go`.

| Original | Archive | Bytes | SHA-256 |
|---|---|---:|---|
| `/tmp/t09-phaseb-inner-preflight-01.raw` | `run-children-phase-b-inner-preflight-01.raw` | 189 | `4c29e1c4aeb72c39e8b01919001f769ecef3426812c791bdbdb1c70fa49ec373` |
| `/tmp/t09-phaseb-inner-preflight-02.raw` | `run-children-phase-b-inner-preflight-02.raw` | 189 | `8f11675cd23c786d1660a71fab1e15bec4dacabae9a2c88c3f3e59d47566d5c3` |
| `/tmp/t09-phaseb-inner-preflight-03.raw` | `run-children-phase-b-inner-preflight-03.raw` | 195 | `8f24df731dda007c653c9def9f3121814feac032213c2d5e28f5fb0051ba0832` |
| `/tmp/t09-phaseb-inner-preflight-04.raw` | `run-children-phase-b-inner-preflight-04.raw` | 195 | `0b3a389b02f2904b8cdf480dbba5d8c10e8f4819712a56860203e6d3cb4977e9` |
| `/tmp/t09-phaseb-inner-preflight-05.raw` | `run-children-phase-b-inner-preflight-05.raw` | 195 | `367b155c28ce2771f6dae7e96b6d3be055f50eceae28ebd5be17d93c1aa9e8a9` |

## Focused race attempts

Attempts 01–04 were intended to run this focused command with the environment below:

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 TMPDIR=/tmp GOCACHE=/tmp/t09-runchildren-phaseb-gocache go test -race -count=1 -parallel=1 -run '^(TestScopedRunsList.*|TestLegacyRunListRequestRemainsUnscoped|TestLiveSource.*|TestStoreDeepCopiesUnreadableRunIDs|TestBuildAddsCaseOwnedRunChildrenForEveryStandingPair|TestBuildOmitsInvalidRunChildrenWithoutDroppingValidSiblings|TestBuildRunChildrenDoNotManufactureParentAttention|TestStoreKeepsRunChildStandingAtCapturedCursor)$' ./internal/adapters/sfwp ./internal/projection
```

Attempt 01 ran from `/home/sprime01/projects/sea-rs` instead of the Go module and stopped
before compilation because the repository root has no `go.mod`; exit 1. Attempts 02–04 ran
from `/home/sprime01/projects/sea-rs/apps/godspeed-casework-go`.

| Original stdout | Archive | Bytes | SHA-256 | Exit | Result |
|---|---|---:|---|---:|---|
| `/tmp/t09-phaseb-inner-race-01.raw` | `run-children-phase-b-inner-race-01.raw` | 130 | `2519f2c38b650409e69899ca1953b1a64afba491ea0cf8c729e13eac62c0d6d1` | 1 | Wrong working directory; no test compiled. |
| `/tmp/t09-phaseb-inner-race-02.raw` | `run-children-phase-b-inner-race-02.raw` | 697 | `23b6abd7c1c5b1b5f76aea460dab1b0434bfefcdb2513843ab75403960ca0544` | 1 | Scoped refusal classification and record-ref fixture assertions failed. |
| `/tmp/t09-phaseb-inner-race-03.raw` | `run-children-phase-b-inner-race-03.raw` | 632 | `d308cda4e61309843f33aec310fbf9bbefcc0d8d03d48ae038d39ef3d2eb4e00` | 1 | Scoped refusal classification and scoped-call-count assertion failed. |
| `/tmp/t09-phaseb-inner-race-04.raw` | `run-children-phase-b-inner-race-04.raw` | 183 | `9c86a66ee1de8ed785b3b45dfdbfc27936ced58ee7d1f15bf0d8280f5adfb9b6` | 0 | Tests passed against a source snapshot containing a rejected missing-record semantic assumption; this is not source approval or valid GREEN evidence. |

Separate original exit files and archive copies:

| Original | Archive | Bytes | SHA-256 | Exit content |
|---|---|---:|---|---:|
| `/tmp/t09-phaseb-inner-race-01.exit` | `run-children-phase-b-inner-race-01.exit` | 2 | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | `1\n` |
| `/tmp/t09-phaseb-inner-race-02.exit` | `run-children-phase-b-inner-race-02.exit` | 2 | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | `1\n` |
| `/tmp/t09-phaseb-inner-race-03.exit` | `run-children-phase-b-inner-race-03.exit` | 2 | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | `1\n` |
| `/tmp/t09-phaseb-inner-race-04.exit` | `run-children-phase-b-inner-race-04.exit` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `0\n` |

The Phase B source snapshot remains rejected and frozen pending root direction. This archive
does not approve that source or the passing fourth attempt. No pre-existing evidence file was
overwritten or deleted.
