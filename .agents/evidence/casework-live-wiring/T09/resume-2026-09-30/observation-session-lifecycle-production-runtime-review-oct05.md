# Independent runtime review: SessionStore lifecycle prerequisite

Date: 2026-10-05. Verdict: **focused and full auth race gates passed** for the
independently source-approved bounded auth-store change. This does not approve
Unit5C SSE integration, poller ownership, or pending read cancellation/drain.
All compiler sessions completed and the exclusive compiler token is returned.

## Frozen inputs and source verdict

The source review is recorded in
`observation-session-lifecycle-production-independent-review-oct05.md`.
Production `session.go` SHA-256:
`d1c9daabfb758de94ea20ea0b7db7fbce1fad3a4ec43f6dee461d428b9972aa5`.
Frozen fixture `session_observation_test.go` SHA-256:
`8de408be7efc31d6e19d73870bee541d82f127272b5c2ab048dbee6ba3212e3f`.
Both match the reviewed builder result. Source review found the bounded
`Current` snapshot and all five removal routes consistent with the approved
proposal; no source was edited during independent review.

## Runtime gates

Environment for both commands: `GOCACHE=/tmp/sea-casework-go-build-cache-auth-prod-oct05`,
`GOMEMLIMIT=256MiB`, `GOGC=50`, `GOMAXPROCS=2`, `GOFLAGS=-p=1`,
`GOLDEN_UPDATE=0`, `-race -count=1 -parallel=1`.

1. Focused lifecycle race command:
   `go test -race -count=1 -parallel=1 ./internal/auth -run
   '^TestSessionStore(Current|ResolveSlides|Removal|Capacity)'`
   Result: **exit 0**, `ok .../internal/auth 1.702s`.
2. Full auth package race command:
   `go test -race -count=1 -parallel=1 ./internal/auth`
   Result: **exit 0**, `ok .../internal/auth 41.794s`. This run used the
   approved loopback escalation because existing OIDC coverage binds an
   `httptest` listener. No test was changed or skipped.

Each command had its own immediately preceding resource preflight. The
namespace-visible `ps` contained only the Codex process, its shell and `ps`;
these are namespace observations, not claims about host-wide process absence.
Focused preflight: 2.5 GiB available memory, 7.3 GiB swap used. Full preflight:
2.4 GiB available, 7.4 GiB swap used. Exact preflight, raw stdout and actual
exit files are saved as `auth-session-focused-{preflight,stdout,exit}-oct05.txt`
and `auth-session-full-{preflight,stdout,exit}-oct05.txt`. All six immutable
copies byte-compare equal to their `/tmp` captures (`cmp` exit 0). SHA-256:

- Focused preflight `2905202ed5f0261b9a683d6a936f47c3731933e00545a366d4d8385974ac1eec`;
  stdout `cf3155b86b991c2392d50aa29634758206892e4d44fc71844afc236f50a07f21`;
  exit `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.
- Full preflight `4ab5e98e391d15250c33ea7e30f7d52c08cd953a12362ae00b12fee71d6dd2bd`;
  stdout `c6d442ec8b328e1deddda07ac411f00c02e0acd21733da14e2398a882968f330`;
  exit `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.

The earlier non-escalated Phase 1 full-auth attempt failed at the existing
OIDC loopback listener (`socket: operation not permitted`) and was preserved in
the separate Phase 1 runtime record. It is not counted as a production gate;
the current full auth gate completed successfully with loopback access.

## Boundary

These passes verify the focused lifecycle test set and all existing tests in
`internal/auth` under the race detector. They do not establish server-side
session watcher behavior, exact timer re-arm scheduling, subscriber/poller
sharing, stale-buffer suppression, or cancellation/drain of `RunTracePort`
reads. Those remain separately authorized and unverified work.
