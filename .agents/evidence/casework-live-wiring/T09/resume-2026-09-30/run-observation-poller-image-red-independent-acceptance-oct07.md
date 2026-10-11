# Independent acceptance: poller image focused RED

Date: 2026-10-07  
Verdict: **ACCEPT the focused RED prerequisite.** This records only the expected throwing-stub RED; it is not a passing test or implementation approval.

## Evidence reviewed

- Execution assignment: `run-observation-poller-image-red-execution-assignment-oct07.md`.
- Pre-run source approval: `run-observation-poller-image-exact-oracle-source-independent-review-oct07.md`.
- Run receipt: `run-observation-poller-image-red-oct07-result.md`.
- Actual preflight, preflight exit, combined output, and command exit archives listed in the receipt.
- Root independently read the actual preflight and full actual output and re-compared all four original/archive capture pairs; root reported all comparisons 0.

## Acceptance basis

The only executed command was the explicitly released focused command from
`apps/godspeed-casework-go`:

```sh
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 go test -v -race -count=1 -parallel=1 -timeout=60s ./internal/server -run '^TestRunObservationPollerImage'
```

The actual run exited 1 after successfully compiling and executing the suite.
Exactly five positive encoder assertions failed on the intended fixed throwing
stub: nil-current encoding, current encoding, exact-cap setup, marker-width
setup, and valid-input alias setup. The two refusal-only top-level tests passed
(oversized nil-current and invalid codes/current-key mismatch); all three
nested validation cases passed. No unrelated compile failure occurred. This is
the expected RED shape for the stub and seven-case fixture.

The fresh preflight satisfied the memory/swap thresholds, reported no
competing compiler, and recorded all seven expected source hashes. The source
result preserves the throwing seam and all five frozen files. Actual capture
archives are present and `cmp` equal to their originals; no output was
transcribed. The run is accepted only as the focused RED prerequisite.

## Scope limits and handoff

No encoder algorithm/GREEN, manager lifecycle behavior, admission/read
suppression, or broader package result is established here. The heavy token is
returned to root. Any further compiler work requires a separate root grant;
this acceptance grants none.
