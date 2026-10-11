# Independent unit 4 run-scope review and verification

Date: 2026-10-01. Scope: the bounded Go run-child projection unit only. This does not settle full T09.

## Governing requirements and review history

Reviewed against `run-children-test-first-assignment.md`, `run-children-root-fixture-clarifications.md`, `run-children-runtime-assignment.md`, `run-children-runtime-test-phases.md`, `run-children-phase-b-semantic-repair-assignment.md`, and `run-children-real-kernel-proof-supplement.md`. The earlier independent package-cycle rejection is preserved in `run-children-phase2-independent-review-reject.md`; its authorized repair scope is `run-children-live-package-repair-assignment.md`. The semantic repair also addressed the preserved findings in `run-children-phase-b-independent-semantic-reject.md`.

The final implementation fixes the two material semantic defects from that rejection: an absent requested case is rejected as unknown immediately after `ListCases`, before reads for other cases; and refusal normalization to unavailable is limited to the explicit unavailable refusal class while preserving the original error chain for other classes. The corrected live test now uses an external test package to avoid the projection/livestack import cycle. The new package-local helper was renamed and qualified; a mechanical comparison found its body unchanged from the original helper. No proof assertion was removed or weakened.

## Frozen source identities

SHA-256 identities at final verification:

| File | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/ports/ports.go` | `e4343e489927c553c789ea3dab6f191326ea31e57b765e295fd184a9eddcd686` |
| `apps/godspeed-casework-go/internal/projection/live.go` | `0f17e374a3dc9df44dd1598188c277e97746ea68f8db9682d0536de68b5349b2` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/authority.go` | `a9c49702fe1b353a1de9814e6559284ffe2eb38806f04b48872e4b366700453b` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/frame.go` | `be8800c437c9f2f699772371a28f14dc068e7c8bc8a543955f6874c0b6259750` |
| `apps/godspeed-casework-go/internal/projection/builder.go` | `3edf1db574480e3d5ce619f4fa9df4c3998127dbb13f6cf3bb84cd1df36d10b2` |
| `apps/godspeed-casework-go/internal/projection/store.go` | `64da50272f3ffee84d0f461c8043eab0420159cda13833ec7a7a27af2d29e05e` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/scoped_runs_test.go` | `57bfc686da998303655bd99fdc5bffcbf386ddbcb931ea8bc290762d848fad5e` |
| `apps/godspeed-casework-go/internal/projection/scoped_live_test.go` | `a9e0deb03f69b12ab379e547de2fb2aaf20c13dd0a51929cdeeda0bea14e7116` |
| `apps/godspeed-casework-go/internal/projection/unreadable_runs_test.go` | `de85a2edb9bd05dc5542c7c9850a9cf9caea69dbb491eb21f95fe93913bfcebb` |
| `apps/godspeed-casework-go/internal/projection/run_children_test.go` | `4ddb2445a7ff6726c3b6fd93832ac4122b53da4d26e5b54180f9a1cafb802741` |
| `apps/godspeed-casework-go/internal/projection/scoped_runs_live_test.go` | `ffc979eee57a02199567395b7c667c3fd0c878dfa4ca8485f9b729ed8b460bcf` |

The corrected original test snapshot is `run-children-live-import-cycle-original.corrected.go.snapshot`, SHA-256 `8e485c64cdc0776d46a8fda6678d22f3bfe98ec837bd8eaaeb94fecced8a9134`. Its differences are limited to externalizing the test package, qualifying `projection.Build`, importing `strings` and the projection package, and copying/renaming the local helper to avoid import-cycle ownership. The exact proof body remains intact.

## Correctness and limits

The pure tests exercise execution/settlement result pairing, unique child identifiers, labels, non-acceptance paths, no actions, bounded child collection, parent/focus/rank/progress behavior, malformed or duplicate/colliding child omission, sibling preservation, and Store cloning of historical run-child state. Adapter tests cover the scoped request mapping and malformed responses. Existing unscoped request behavior remains covered.

The live test uses the existing executable `target/debug/sea-forge-server` (51,456,400 bytes; SHA-256 `c17b4c1458c019a7527305cd6313638c3143e1c06e95af37c13659b0cf4603fc`) and does not invoke Cargo. It commits two actual cases, executes `task_prepare` for each, observes accepted settlement, lists the requested case's runs through the scoped owner path, and verifies the production projection contains the actual child IDs/parent links and separate child standings while excluding the foreign case's summary/child. It makes no action request.

One interface limitation remains explicit: ordinary refusal from the concrete `Client.Do` is returned as an error before a response exists, so the defensive `Response.Into` refusal branch cannot be reached through that concrete client. The branch is reviewed at source level; tests cover the reachable refusal mapping and its error classes. This is a bounded testability limitation, not a claim of runtime coverage for the defensive branch.

## Verification

Every command had an actual-host memory/process preflight immediately before execution. The preflights show 2.0–2.3 GiB available and only unrelated Bun PID 2109808 (`server.ts`), with no competing Go compiler/test process. Go commands were serialized with `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0`; race tests used `-race -count=1 -parallel=1`.

1. Focused race tests, exit 0: `go test -race -count=1 -parallel=1 ./internal/adapters/sfwp ./internal/projection -run 'TestScopedRunsList|TestLegacyRunListRequestRemainsUnscoped|TestLiveSource|TestBuild.*RunChildren|TestStoreKeepsRunChildStandingAtCapturedCursor|TestStoreDeepCopiesUnreadableRunIDs$'`.
2. Kernel-backed live test, exit 0: `TMPDIR=/tmp/t09u4 go test -race -count=1 -parallel=1 -tags live ./internal/projection -run '^TestLiveRunScope$' -v`.
3. Canonical module check, exit 0: `just casework-go-check` (format, vet, regular tests).
4. Full Go module race suite, exit 0: `go test -race -count=1 -parallel=1 ./...`.
5. Supplemental live log capture rerun, exit 0: same `TestLiveRunScope` command with `TMPDIR=/tmp/t09u4capture`. Both actual case executions again report accepted settlement. The captured temporary root was 48 bytes and socket path 60 bytes, well within the test's socket-path constraint.

The live test's cleanup can emit a relay rebuild EOF/client-closed warning after the test has printed PASS. This is teardown only: stack cleanup closes the client before canceling the relay context. The warning occurred in both successful live runs and does not change the process exit (0) or assertions. The supplementary capture confirms the server started and listened; the raw watcher capture also includes `tail` diagnostics when `t.TempDir` removed the watched directory. Those observer lines are retained rather than presented as server log content.

## Raw evidence manifest

Each raw output, exit, and preflight listed below is archived separately and remains unedited. The captured sources and hashes are:

| Gate | Original `/tmp` output → evidence path | Output SHA-256 | Exit SHA-256 | Preflight SHA-256 |
|---|---|---|---|---|
| Focused | `/tmp/t09-unit4-fix-focused.raw` → `unit4-fix-focused.raw` | `ff69a61872a2d5b2bfab8c0604db8b169106a6796e5f7b03d04f9eedfa9f3469` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `60de62b32583346c8e2914778fb3c72f8b42268e5ce0434ee9a260d4034157fc` |
| Live | `/tmp/t09-unit4-fix-live.raw` → `unit4-fix-live.raw` | `6f668591a51d6a93bec64693a941e8a72b9b40a09c4d599acc8fd1166d331715` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `3c69c3245ba2e110580b600d30178e2566fead2d615a536439dbd1074bb50f66` |
| Canonical | `/tmp/t09-unit4-fix-canonical.raw` → `unit4-fix-canonical.exact.raw` | `dedaeb7a98ed782df1301e5e946f39356afb0d808e3e0f7b87bf6900b1a0f0b8` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `57017d5a2114c9504a89201250c1a05b190c9c380be8e92ce3044db3cbe271ee` |
| Full race | `/tmp/t09-unit4-fix-fullrace.raw` → `unit4-fix-fullrace.exact.raw` | `88b92e8a01571e72029ebc8641da9f6915b1becb1d9fae3f3bc522d39a00f508` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `5c02178547eaf3fe6ddf47ed572bf00fa252609009b4f83c5f435d1a2fb293c5` |
| Supplemental live capture | `/tmp/t09-unit4-fix-live-capture.raw` → `unit4-fix-live-capture.raw` | `9b055898df3b13a80069f74dfe0d99bc94738b84863757850cf30ce5eaa16925` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `41058c1e39b1e8b1ae986a4f73a54344b928f739d31978038786c3482b690b15` |

Supplemental sidecars are byte-identical to their originals: `/tmp/t09-unit4-fix-live-capture-path.raw` → `unit4-fix-live-capture-path.raw`, SHA-256 `cb13c02ab038e93f19ad31cddad837c6f6cb4d1106eb03c81271612a659043c1`; `/tmp/t09-unit4-fix-live-server.raw` → `unit4-fix-live-server-watcher.raw`, SHA-256 `1567fdf73375bf7820bf4d07c510fb0b2ca7729e47131a6b4ee832d51c20e7e2`. Each was checked with `cmp` against its original. The watcher file contains two server JSON lines followed by three `tail` cleanup diagnostics.

The first native archive attempt for canonical and full-race output omitted one alignment space on five `?` rows. The initial artifacts were retained unchanged as `unit4-fix-canonical.raw` (SHA-256 `78f07ca3f9baac668a6b42034e07abbbb5691bc9032b8807252bda4d61256fbf`) and `unit4-fix-fullrace.raw` (SHA-256 `f865bb46919882cfab1e57c08c06332166e421e1e20251f46b8171ea52ef95f5`). The corrected exact copies use new names and match the originals byte-for-byte. `unit4-fix-archive-transcription-erratum.md` records the mismatch; no raw evidence was overwritten.

## Verdict

Approve this bounded unit 4 Go run-scope/run-child change based on the reviewed frozen source identities, focused race tests, real-kernel live proof, canonical Go check, and full Go race suite above. This verdict does not approve or settle other T09 units, the UI mirror, or the full T09 milestone.
