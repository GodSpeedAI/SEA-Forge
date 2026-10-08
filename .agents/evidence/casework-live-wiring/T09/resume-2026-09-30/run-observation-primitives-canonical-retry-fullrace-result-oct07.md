# Isolated primitives canonical retry result

Date: 2026-10-07  
Status: canonical `just casework-go-check` failed on sandbox socket permission errors; full-module race gate was not run.

## Authorized command and preflight

The full execution assignment is archived at
`run-observation-primitives-canonical-retry-fullrace-assignment-oct07.md`
(SHA-256 `f5007a0551fb08605c488696ee706799165e9b12f4fedd670c385626d5f66b83`).
The isolated root was `/tmp/sea-casework-primitives-74f86f0-oct07`, HEAD
`74f86f0964f92c5b1715eb0703a9b880d28cccae`, with exactly the six authorized
untracked source/test paths. Its status contained no other paths. All six
isolated hashes matched the authorized identities, and each isolated file
compared byte-for-byte with its primary counterpart.

The fresh gate preflight at
`/tmp/sea-primitives-canonical-gate1-critic-20261007/preflight.raw` recorded
UTC `2026-10-07T22:13:26Z`, `MemAvailable=2821796 kB`, `SwapFree=10280092 kB`,
zero competing Go processes, and zero competing Rust compiler processes. The
preflight exit was `0`.

The actual command, from the isolated root, was:

```sh
env JUST_TEMPDIR=/tmp GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 just casework-go-check
```

Its stdout and stderr were redirected together to
`/tmp/sea-primitives-canonical-gate1-critic-20261007/canonical.stdout.raw`;
its exit status was captured separately. The process exited `1`. The output
ends with `error: recipe \`casework-go-check\` failed with exit code 1`.

## Failure observed

The captured package tests report socket creation failures with `operation not
permitted`. Examples include Unix socket listener setup in
`internal/adapters/sfwp` and TCP6 loopback listener setup by `httptest` in
`internal/auth` and `internal/server`. The `internal/server` example is
`TestArtifactGetReturnsVerifiedCanonicalPayloadAndSessionPerspective`, whose
`httptest.NewServer` panics after `listen tcp6 [::1]:0: socket: operation not
permitted`. Thus the canonical recipe did not pass in this sandbox. This
result does not attribute the failure to the six primitive files.

As required after canonical failure, the full-module `go test -race` gate was
not run. No retry, permission escalation, source edit, formatter, extra
verification gate, or Git mutation was performed.

## Captured evidence

All four actual preflight/output/exit originals were archived immediately
after the gate joined, before any further gate, and compared byte-for-byte
with their archives (`cmp=0`):

| Capture | Actual original | Archive | Bytes | SHA-256 |
|---|---|---|---:|---|
| Preflight stdout | `/tmp/sea-primitives-canonical-gate1-critic-20261007/preflight.raw` | `run-observation-primitives-canonical-gate1-preflight-oct07.raw` | 1,841 | `aa3d1e745c95a60e6bc5ab5fb7ffd203225f492c1f03c25ed0f73adf5deccc83` |
| Preflight exit | `/tmp/sea-primitives-canonical-gate1-critic-20261007/preflight.exit` | `run-observation-primitives-canonical-gate1-preflight-exit-oct07.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| Canonical stdout+stderr | `/tmp/sea-primitives-canonical-gate1-critic-20261007/canonical.stdout.raw` | `run-observation-primitives-canonical-gate1-output-oct07.raw` | 54,758 | `55a548c0ffe58d3a4d135a5850845b3791eca1cedff3920243c9fdb3e7334ed2` |
| Canonical exit | `/tmp/sea-primitives-canonical-gate1-critic-20261007/canonical.exit` | `run-observation-primitives-canonical-gate1-exit-oct07.raw` | 2 | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |

The two exit captures contain `0\n` and `1\n`, respectively. Archive
filenames are relative to this evidence directory. This is an actual failed
canonical gate with preserved output, not a pass and not evidence for the
unrun full-module race gate.
