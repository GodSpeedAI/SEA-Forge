# Full-module race execution assignment

Date: 2026-10-08

Executor: independent Luna renderer_resume_critic, sole compiler owner for
exactly one full-module race. The full instructions are in
`run-observation-fullmodule-race-assignment-oct08.md`; the original canonical,
focused, source, fixture, and formatter instructions govern as cited there.
Canonical listener retry03 passed under normal permission escalation and is
archived in `run-observation-canonical-go-listener-retry03-result-oct08.md`.

Before running, capture fresh UTC, HEAD, all eleven current source identities,
MemAvailable, SwapFree, and namespace-visible competing compiler/gate
processes. Require source hashes to equal canonical retry03, MemAvailable >=
1200 MiB, SwapFree >=512 MiB, and no other visible compiler. Stop if any
condition fails.

Run once from `apps/godspeed-casework-go` using normal listener permission
escalation, justified by existing tests requiring local Unix/TCP listeners
denied by the sandbox:

```sh
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp go test -race -count=1 -parallel=1 -timeout=120s ./...
```

Capture exact command, preflight, preflight exit, stdout, stderr, and exit in a
new unique `/tmp` directory. Immediately after JOIN, archive all six captures
via native `apply_patch` with original paths and raw bytes (Base64 JSON is
acceptable), decode and compare each archive to its untouched original before
any further command or gate. Derive counts, identities, and hashes from the
actual data. Write one immutable result receipt and return compiler ownership.

No source/test/formatter/golden edits, Git/Graft build, or second gate. If the
result is an established coordinator-only baseline flake, preserve it and
report root before any isolation retry. No Next/public C2/live-wiring/T09
completion claim. After this bounded gate, provide an independent lifecycle
readiness verdict against the complete original assignments and frozen
implementation, with material deviations and evidence; this is not a runtime
or lifecycle completion claim.
