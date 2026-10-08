# T09 Go/UI contract mirror: independent GREEN

Date: 2026-10-01
Result: **APPROVE the bounded Go/UI contract mirror unit only.**
Reviewer/compiler owner: independent critic `t09_mirror_independent_critic`.
Compiler token: released to ROOT after all listed checks completed.

This approval follows the two immutable rejection records:
[`mirror-first-independent-source-reject.md`](mirror-first-independent-source-reject.md) and
[`mirror-second-independent-gate-reject.md`](mirror-second-independent-gate-reject.md).
The first repair supplied the missing Go field-type inventory, complete UI structural type
pins, and an exact-key hydration-budget validator with a negative unknown-key fixture test.
The first focused Go gate then exposed a golden formatting mismatch, recorded in the second
rejection.

## Fixture-only integration adjustment

After the second rejection, ROOT authorized a separate fixture-only writer to change whitespace
in only `ask-answer-answered.json`, `ask-answer-partial.json`, and `ask-answer-denied.json`.
The Go mirror implementation assignment forbids its builder from authoring goldens; this was a
separate root-authorized integration correction. A Python JSON parse comparison against commit
`b608771` returned `parsed-equal=True` for each of the three fixtures. No values, types, fields,
or field order changed. The request fixture did not change. The four fixture hashes below match
the frozen fixture repair values and remained unchanged through the passing Go gates.

## Frozen source and fixture identities

| File | SHA-256 |
| --- | --- |
| `apps/godspeed-casework-go/internal/contract/contract.go` | `f0b51bfb61698e56faf16cf4a0216010e52e9c2f447f0ff673b78980964363c6` |
| `apps/godspeed-casework-go/internal/contract/golden_test.go` | `f3d3f79d3667156042474b3152782a4bfbc9e9b11eaf959b60e4991a3d069f39` |
| `apps/godspeed-cognitive-ui/src/ports/contract.ts` | `042970baaa5dc131eb2ee2ab00a29a6fc5fcf62abffa1f11b6f5d9b253336bf4` |
| `apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts` | `ce48a051ebbd6037da828f94986a757adf25ec083fc5eddb25230f3f635036c0` |
| `ask-request.json` | `30efdb892da29fa21760db57511bac75f85d4116df0f06d4a1461e5ecf295864` |
| `ask-answer-answered.json` | `dcb6f816153996415ff6aa086eb38233eca65fed36ca4edc06bb4ae2897701dd` |
| `ask-answer-partial.json` | `b417756d06cdb37388bd0a1fc3b472494a70bb132e126647a9561c299c8732d1` |
| `ask-answer-denied.json` | `0ac5e77755b3ea2d5ab7193eb757fb8829ec3b9e2d6811dd74c823b37360d03d` |

These mirror and fixture source identities were captured before ROOT released Go-source writes
for the Ask route. The Go gates below therefore approve the **pre-Ask Go source graph** and
include the approved mirror files and fixtures at those identities. Ask implementation files
were added afterward (`internal/ports/ask.go` and an `internal/server/http.go` edit are now
present). The later T09 Go gate must cover those additions; this record does not claim that the
current, post-Ask Go graph passed the earlier commands. UI mirror files remained at the listed
hashes through UI verification.

## Verification results

All actual command exit statuses were captured to separate `.exit` files. Test stdout/stderr
was redirected directly to raw logs, without a pipeline that could mask the tested command's
status. Go commands used the task-owned `/tmp/t09-mirror-repair/go-cache` and
`/tmp/t09-mirror-repair/go-tmp`, with `GOMEMLIMIT=256MiB`, `GOGC=50`, `GOMAXPROCS=2`, and
`GOFLAGS=-p=1`. Go race commands set `GOLDEN_UPDATE=0`; the `just casework-go-check` pass ran
with `GOLDEN_UPDATE` unset (verified unset afterward). Ask fixture hashes were unchanged after
that run.

| Gate | Command | Actual exit | Result |
| --- | --- | ---: | --- |
| Canonical contract Bun | `bun test ./.agents/reports/interface-contracts/tests/contract-conformance.test.ts` | 0 | 11 pass, 0 fail, 963 assertions (final persisted-preflight rerun) |
| Canonical explicit strict TS pins | `./node_modules/.bin/tsc --noEmit --strict --target ES2023 --lib ES2023,DOM,DOM.Iterable --module ESNext --moduleResolution bundler --skipLibCheck --types bun-types --typeRoots ./node_modules ../../.agents/reports/interface-contracts/tests/contract-conformance.test.ts` from `apps/godspeed-cognitive-ui` | 0 | no diagnostics |
| Focused Go contract | `go test -race -count=1 ./internal/contract` from `apps/godspeed-casework-go` | 0 | pass |
| Prescribed Go gate | `just casework-go-check` | 0 | gofmt, vet, and all module tests green |
| Full Go module race | `go test -race -count=1 ./...` from `apps/godspeed-casework-go` | 0 | all packages green |
| UI typecheck | `bun run typecheck` from `apps/godspeed-cognitive-ui` | 0 | pass |
| Focused UI wire tests | `bun test src/ports/wireContract.test.ts` from `apps/godspeed-cognitive-ui` | 0 | 15 pass, 0 fail, 512 assertions |
| Full UI gate | `just casework-ui-check` | 0 | frozen install, typecheck, production build, and 263 tests / 1427 assertions green |

The focused UI run confirms that the negative test for an unknown hydration-budget property
passes without leaking a nested failed `expect`. The full UI production build emitted its
existing chunk-size advisory (largest JS chunk about 907 kB); build and gate exited successfully.

## Host preflight and execution deviations

Before each compiler/test command, the actual-host scan used `exec_command` with
`sandbox_permissions=require_escalated`, `free -h`, and a process scan restricted to Go, Cargo,
Rust, Bun, Node/Vite, and TypeScript compiler commands. Captured preflights are preserved under
`mirror-verification-v2-preflight-*.txt`.

- The full Go gate's first sandboxed attempt could not create Just's temp directory under
  `/run/user/1000/just`; it exited 1 before compilation. The retry used task-owned
  `TMPDIR=/tmp/t09-mirror-repair-v2/tmp` and
  `XDG_RUNTIME_DIR=/tmp/t09-mirror-repair-v2/xdg-runtime`.
- The sandboxed Go gate retry and sandboxed full UI gate each reached tests but failed only when
  tests tried to open Unix/TCP localhost sockets (`operation not permitted` / `EPERM`). Those
  outputs and statuses are preserved, not counted as passing evidence. Their escalated reruns
  passed the full gates.
- The first actual-host preflight before focused UI tests found unrelated PID `1167718`, command
  `bun test tests/architecture/`. It was left untouched. A process check confirmed it exited;
  the subsequent persisted preflight found no competing compiler. The original busy scan and the
  clean retry scan are both retained.
- The first actual-host preflight immediately before the first canonical Bun run was observed at
  `2026-10-01T04:20:39Z` (2.6 GiB available, no matching build process) but was not saved to a
  `/tmp` file at that moment. A final identical canonical Bun run was performed later with a
  persisted preflight and passed. All other final compiler/test invocations have a captured
  actual-host preflight.

## Immutable raw evidence

Raw command output, separate actual-exit files, and captured actual-host preflights are stored as
`mirror-verification-v2-<original-name>` siblings in this directory. They were added by native
`apply_patch` from their complete `/tmp/t09-mirror-repair-v2/` files, then byte-compared with
`cmp`; all copies matched. The first canonical TypeScript pin log is intentionally empty
(SHA-256 of the empty file `e3b0c442...b855`) and the compiler's captured actual exit is zero.

Selected raw-log hashes:

| Raw log | SHA-256 |
| --- | --- |
| `mirror-verification-v2-canonical-bun-final.log` | `d8013deae0bb2033df626f0fc8b669291ab6ab8624dcc60cf82b7bdb25f35495` |
| `mirror-verification-v2-canonical-tsc.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `mirror-verification-v2-go-focused-race-count1.log` | `e64ff1a3db2a3cb40afc09168baa6c040bdb5bcb5851dc525e8e119a1c624387` |
| `mirror-verification-v2-just-casework-go-check-escalated.log` | `ea375aca9d4f4e8f3ffc38e7014ad8945f1fee08628485635a67fef2f09047b4` |
| `mirror-verification-v2-go-module-race-count1.log` | `aba89ea367b0265639f6d995572d2eb2a2c6462d4f45137dd42bd57b8e0aa1d9` |
| `mirror-verification-v2-ui-typecheck.log` | `8366207267355d3e3d5bf3bf6e8c94c5f93f6078c34f08973fa2b38cdda6cc92` |
| `mirror-verification-v2-ui-wire-focused.log` | `67689161860ba8b0cad64f983af3dd2a714062d149590d62230bdac4dcabe204` |
| `mirror-verification-v2-just-casework-ui-check-escalated.log` | `eea1f5e2e0ebdb195d974bf95a16e3f2dbb00aae800d5f6f77eb94a3d63e68c3` |

Final `git diff --check` across the four mirror files and the three whitespace-adjusted Ask
answer fixtures passed with no output. This is a bounded mirror-unit approval only. Ask route
behavior, observation runtime enforcement, the post-Ask Go source graph, and full T09 settlement
remain outside this approval.
