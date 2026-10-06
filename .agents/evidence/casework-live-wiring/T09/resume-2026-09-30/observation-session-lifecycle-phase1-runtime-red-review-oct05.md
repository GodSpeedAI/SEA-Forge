# Independent runtime RED review: session lifecycle Phase 1

Date: 2026-10-05. Verdict: **intended focused assertion RED confirmed** for the
frozen auth fixture. Full auth-package execution separately hit a sandbox
loopback-listener failure before reaching the intended assertion. This review
does not approve implementation. No source, tests, or Git state were changed.

## Fixture and command identity

Frozen source identity at execution: `session.go`
`f8f0df88d283ef0c28057f08fcdd6b23c7111c89abd68e23b52f754eb9d17b48` and
`session_observation_test.go`
`8de408be7efc31d6e19d73870bee541d82f127272b5c2ab048dbee6ba3212e3f`. These
match the independently approved source-ready Phase 1 review and follow-up
builder record. The `e8859aa2` suffix in temporary evidence filenames is only
the command/cache-run label; it does not identify the frozen source. The source
hashes above identify the fixture.

Go settings were exactly: `GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1`,
`GOMEMLIMIT=256MiB`, `GOGC=50`, `GOMAXPROCS=2`, `GOFLAGS=-p=1`,
`GOLDEN_UPDATE=0`, with `-race -count=1 -parallel=1`.

First, full package command `go test -race -count=1 -parallel=1 ./internal/auth`
completed with actual exit 1. Existing
`TestOIDCDiscoveryAndFlowConstruction` panicked because
`httptest.NewServer` could not bind `[::1]:0` (`socket: operation not
permitted`, `internal/auth/oidc_test.go:89,152`). This is an environment/setup
failure, not the expected RED. Its exact preflight, raw stdout and exit are
preserved in the `auth-phase1-package-{preflight,stdout,exit}-oct05.txt`
evidence copies.

Then the bounded focused command
`go test -race -count=1 -parallel=1 ./internal/auth -run
'^TestSessionStore(Current|ResolveSlides|Removal|Capacity)'` completed with
actual exit 1. It compiled and ran without setup errors or hangs. Its exact
preflight, raw stdout and exit are preserved in
`auth-phase1-focused-{preflight,stdout,exit}-oct05.txt`. Each copy was compared
byte-for-byte with its `/tmp` capture (`cmp` exit 0); the raw hashes are:

- Package preflight `f3dd720393676283bb760cda6325e3b045c59de328ffd229a65f12a46a51c372`;
  stdout `a407302926e207efb41d059882ca0c7e529023cc122aa9ee0f5663a0e9a1c716`;
  exit `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865`.
- Focused preflight `022a629e601f5689a09461cdbe8f01beea9d1d4453612f8fa3caeb2e588657aa`;
  stdout `e1c7f8ed6c13a1021290aad4cc16d843b139a4ebee1c1afe70c07599766f5324`;
  exit `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865`.

## RED analysis

All focused failures are behaviorally consistent with the frozen stub at
`session.go:143-146`, which always returns zero state and false. Failure sites
are `requireCurrentSession` before each test proceeds: detached/non-sliding
`:81`; idle and absolute boundary subtests `:113`; Resolve sliding/fixed
absolute `:132`; every deletion subtest `:211`; capacity `:224`; race setup
`:258`. No compilation, setup, timeout, or unrelated test failure occurred in
the focused invocation. Thus this is the intended Current-live-state RED.

Important limit: because Current returns false, each grouped test stops at its
initial live-state assertion. The run proves that the declaration/stub is
observable and the suite reaches the assertion; it does not dynamically
exercise channel closure, deadline calculations, or race outcomes yet. Those
remain fixture coverage for the later implementation GREEN.

The initial and second preflights used namespace-visible process inventory,
not host-wide absence claims. Both showed only the Codex process, its shell,
and the preflight `ps`; memory/swap snapshots are in the exact preflight
captures. No concurrent compiler/scanner was visible in that namespace.

The only running command sessions completed: full-package session exit 1 and
focused session exit 1. No process was left running. The source RED token is
returned to root; no further compiler command was run. Full auth package
coverage remains unverified until an explicitly approved environment can run
its loopback-dependent OIDC test; do not weaken or skip that test to claim a
package pass.
