# Poller image focused RED result — 2026-10-07

Status: focused encoder RED completed under explicit sole-heavy-token grant.
All actual capture artifacts were archived immediately after process join and
compared byte-for-byte with their `/tmp` originals before another gate.

## Authorization and reviewed source identity

The run followed the immutable execution assignment
`run-observation-poller-image-red-execution-assignment-oct07.md` and the
independent source review
`run-observation-poller-image-exact-oracle-source-independent-review-oct07.md`.
The review found the exact two-file stub/fixture state ready for focused RED.
No algorithm, source, fixture, or frozen-file edits occurred during this run.

Preflight source hashes were:

- Throwing seam `run_observation_poller_image.go`: `32f57bd8aa6edb9c20b645e3138b4b9e9fc02076518a34587253a3af31af328e`.
- Exact-oracle fixture `run_observation_manager_retained_image_test.go`: `8e741f7604f49bf28674e669f820ec9e696abd84ad4c60bb2a145b36123496f8`.
- Frozen manager source: `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`.
- Frozen manager fixture: `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.
- Frozen pure helper: `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`.
- Frozen base helper fixture: `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`.
- Frozen policy fixture: `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.

## Actual preflight and command

Fresh preflight at `2026-10-07T18:17:38Z` passed. `/proc/meminfo` reported
`MemAvailable=3281160 KiB` (about 3,204 MiB) and `SwapFree=11346088 KiB` (about
11,080 MiB). The recorded process check found no competing `go`, `compile`,
`link`, `rustc`, or `cargo` process. Go version was `go1.27.1 linux/amd64`.
Preflight exit was 0.

From `apps/godspeed-casework-go`, ran exactly:

```sh
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 go test -v -race -count=1 -parallel=1 -timeout=60s ./internal/server -run '^TestRunObservationPollerImage'
```

The command compiled and executed the focused suite. Exit was 1 because the
throwing stub returned its fixed generic error for the five positive encoder
assertions. The five expected failures were nil-current encoding, current
encoding, exact-cap setup, marker-width setup, and valid-input alias setup.
The two refusal-only top-level tests passed: oversized nil-current and invalid
codes/current-key mismatch. Its three validation subtests also passed. There
was no unrelated compile failure, package test, or extra gate.

This is expected RED evidence only. It does not establish production encoder
behavior or a passing test result. Fixed generic errors disclosed no key or
candidate data. No source edits, tests beyond this command, formatter,
scanner, Git operation, or build/index command was run.

## Actual capture archive and comparison

The process ran in `/tmp/poller-image-red-k5tjVa`. The following actual files
were archived in BASE using native apply_patch immediately after the command
joined; each archive was then compared with its `/tmp` original. Every `cmp`
returned 0:

| Capture | BASE archive | Bytes | SHA-256 | cmp |
|---|---|---:|---|---:|
| Preflight | `run-observation-poller-image-red-preflight-oct07.raw` | 2000 | `47e49ffebc0f54a6e70a42d4dd054a9dab28135c741f2f28657cf40a2ebe5647` | 0 |
| Preflight exit | `run-observation-poller-image-red-preflight-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | 0 |
| Combined command output | `run-observation-poller-image-red-output-oct07.raw` | 2429 | `84fae8578513ee4730d204eabb78f358ab3b99a25a3e2ad723794cce77ec2600` | 0 |
| Command exit | `run-observation-poller-image-red-exit-oct07.raw` | 2 | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | 0 |

The exit files contain the actual decimal exit code followed by LF (`0\n` for
preflight and `1\n` for the test process). The reported byte counts and hashes
are from both original and archived copies. No output was manually
transcribed.

## Handoff boundary

The focused RED prerequisite is recorded for root review. The sole-heavy token
is returned to root. Any encoder implementation/GREEN, manager lifecycle
fixture, or broader gate requires a separate explicit grant and independent
source review.
