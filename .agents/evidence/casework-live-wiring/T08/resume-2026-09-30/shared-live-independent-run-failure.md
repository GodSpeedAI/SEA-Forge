# Independent shared live conformance run

**Verdict: REJECT T08 pending investigation.** The independently run shared live
conformance command exited 1 after starting its own gateway and kernel.

## Command and host preflight

From `apps/godspeed-cognitive-ui`:

```sh
TMPDIR=/tmp/t08-live-independent-20260930 GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOCACHE=/home/sprime01/.cache/go-build CASEWORK_LIVE=1 bun run e2e/live-conformance.ts
```

Actual-host preflight at 2026-09-30 21:25:06 UTC showed 2,100,564 kB available RAM,
no Go/Cargo/Vite compiler process, and no listener on TCP 4179. The only matching
foreign runtime in the bounded check was Bun PID 13814; it was left untouched.
The run started a fresh kernel cell and gateway under
`/tmp/t08-live-independent-20260930/t08-live-run-fk6AmA/` and retained that cell on
failure. A post-run actual-host check at 21:26:21 UTC showed TCP 4179 free, no Go
compiler, and the same unrelated Bun PID still present.

## Result

The run passed health, unauthenticated world refusal, login/session identity,
template/preflight, PROPOSE acceptance and case identity, initial snapshot/action
offers, then failed the resumed-stream cursor assertion:

```text
CaseworkPort conformance failed: resumed stream replays the accepted mutation cursor
got 01M3T3AH846066KYV4X73TFCT9, expected at least 01M3T3AH8PKM99Y651GMBZZ37E
```

The durable trajectory assertion that follows this check was not reached. This run
therefore does not establish retained history or old-snapshot immutability for the
shared adapter.

The assertion at `src/adapters/conformance/caseworkPortConformance.ts:120-126` waits
for the first resumed event after the original snapshot cursor and requires that
first event to be at or beyond the accepted receipt cursor. The failure may be an
over-strong first-frame expectation if an accepted action emits an earlier
intermediate frame; this runtime result alone does not establish which layer is
wrong. The exact accepted receipt point should be observed in the resumed stream
before classifying the discrepancy. No source was changed by this critic.

## Artifacts

- Raw command output and explicit `exit=1`:
  `/tmp/t08-live-independent-20260930/shared-live-conformance.log`
- Failed run cell retained at:
  `/tmp/t08-live-independent-20260930/t08-live-run-fk6AmA/cell`
- Raw log SHA-256: `8153d0c2465ee2fe2a044d07594f0cc195e3efdb0f92cce3734be363d7c32e45`
- Shared runner source SHA-256:
  `52d881816346d3a71ba1e457efd72f65c46c021513837ea42fd1a0c93f4db6bf`
- Conformance assertion source SHA-256:
  `85e118edc991d175e422069c93a5f1baecdd3b2cb5427bf8695fdb2155abf295`

The runner's failure path retained its owned cell and its process cleanup completed;
the post-run port check was clear. The prepared plan asked to preserve cell/output.
The failure cell is present. On success, current source removes its temporary run
root, so a successful rerun would need to capture required artifacts before that
cleanup if the plan's preservation requirement is still in force.
