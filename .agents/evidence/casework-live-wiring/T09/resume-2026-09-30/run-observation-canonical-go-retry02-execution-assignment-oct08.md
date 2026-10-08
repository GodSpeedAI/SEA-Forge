# Canonical Go retry02 execution assignment

Date: 2026-10-08

Executor: independent Luna renderer_resume_critic. Root grants sole compiler
ownership for exactly one canonical retry. The full retry assignment is
`run-observation-canonical-go-retry02-assignment-oct08.md` (SHA-256
`2d921f8e18f5cb7056623bd981088b234cda592efc6075bb6d67a49294f24e16`). The
original canonical command and constraints are in
`run-observation-canonical-go-current-source-assignment-oct08.md` (SHA-256
`8ec3e01783dca962b1ba0ff22b11b07a80b4752dcafebf87c8f911661df50172`). Prior
canonical failure result, focused retry02 result, and independent formatter
review were read. The formatter review approved only source formatting.

Only source identity changed from the original canonical preflight is
`apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go`,
now 47,658 bytes with SHA-256
`8c71197adc61bf0f0a3fb0a0fdf413ff41f1fdfa54e8a4af0c4a3abf63cb8df8`. All
other ten source identities must match the focused retry02 preflight.

Before running the gate, record a fresh UTC timestamp, repository HEAD, eleven
source paths/byte lengths/SHA-256 values, MemAvailable, SwapFree, and visible
competing compiler/gate processes. Require MemAvailable >=1200 MiB,
SwapFree >=512 MiB, and no other visible compiler/gate process. Use one job.
If any condition fails, do not execute the gate.

Run exactly once from repository root:

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp just casework-go-check
```

Capture untouched actual command, preflight, preflight exit, stdout, stderr,
and gate exit in a unique `/tmp` directory. After actual JOIN, immediately
archive all six captures as new immutable artifacts using native
`apply_patch`; decode each archive and compare it byte-for-byte with its
original before any further command or gate. Preserve actual listener
permission failures. Do not run a second gate, edit source/tests/formatting,
update goldens, run scanners, mutate Git, or build Graft. Do not claim
lifecycle/T09 completion.

Write one new concise result receipt with actual paths, command, preflight,
source identities derived from capture data, exit status, output summary,
capture hashes and byte comparisons. Root must independently verify the
captures before any further gate. Return compiler ownership after JOIN,
archival, and receipt.
