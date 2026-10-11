# Canonical Go listener retry03 execution assignment

Date: 2026-10-08

Executor: independent Luna renderer_resume_critic. Root grants sole compiler
ownership for exactly one normal permission retry. Full instructions are in
`run-observation-canonical-go-listener-retry03-assignment-oct08.md`; the
original canonical grant and retry02 assignment/result also govern. The
retry02 canonical command ran once and failed because its test listeners were
denied by the sandbox. That failure and its six capture archives remain
immutable. Root accepted retry02 as an environment failure, not GREEN.

Use the same exact root command, toolchain, environment, tests, and eleven
source identities as the original canonical grant. The only source variant is
already-frozen formatter output in
`run_observation_manager_failure_test.go` (47,658 bytes,
`8c71197adc61bf0f0a3fb0a0fdf413ff41f1fdfa54e8a4af0c4a3abf63cb8df8`). Do not
edit sources, tests, formatting, or goldens.

Before the gate, capture a fresh UTC timestamp, HEAD, all eleven source
identities and byte lengths, MemAvailable, SwapFree, and visible competing
compiler/gate processes. Require MemAvailable >=1200 MiB, SwapFree >=512 MiB,
jobs1, and no other visible team compiler. If any requirement fails, stop.

Run exactly once from repository root using
`tools.exec_command` with `sandbox_permissions="require_escalated"` and the
justification that the existing canonical Go tests require local Unix/TCP
loopback listeners denied by the sandbox. This is a normal permission
escalation, not a gate bypass. If automatic approval review rejects it,
preserve and report that response; do not try a workaround or another attempt.

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp just casework-go-check
```

Capture the exact command, preflight, preflight exit, stdout, stderr, and gate
exit in a new unique `/tmp` directory. Immediately after actual JOIN, archive
all six captures through native `apply_patch`, preserving original paths and
raw bytes (raw Base64 is acceptable). Decode and compare every archive against
its original before another command or gate. Derive source hashes and outcome
details from the captures. Write one new immutable result receipt, then return
compiler ownership. Do not run a second gate, alter Git, build Graft, or claim
lifecycle/T09 completion. Root must independently verify captures before any
full-module race or further gate.
