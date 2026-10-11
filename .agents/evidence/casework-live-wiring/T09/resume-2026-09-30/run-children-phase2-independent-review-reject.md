# Run-children Phase 2 independent review: REJECT

Review date: 2026-10-01. Scope is unit 4 only. This review read the original runtime assignment, test-phase supplement, root fixture clarifications, real-kernel proof supplement, Phase B semantic repair assignment, both immutable prior rejection records, the actual Phase 2 production, and every frozen unit 4 fixture. No production, test, or golden source was edited by the critic.

## Frozen identities reviewed

Measured before the focused gates:

- `internal/ports/ports.go`: `e4343e489927c553c789ea3dab6f191326ea31e57b765e295fd184a9eddcd686`
- `internal/adapters/sfwp/frame.go`: `be8800c437c9f2f699772371a28f14dc068e7c8bc8a543955f6874c0b6259750`
- `internal/adapters/sfwp/authority.go`: `a9c49702fe1b353a1de9814e6559284ffe2eb38806f04b48872e4b366700453b`
- `internal/projection/live.go`: `0f17e374a3dc9df44dd1598188c277e97746ea68f8db9682d0536de68b5349b2`
- `internal/projection/builder.go`: `3edf1db574480e3d5ce619f4fa9df4c3998127dbb13f6cf3bb84cd1df36d10b2`
- `internal/projection/store.go`: `64da50272f3ffee84d0f461c8043eab0420159cda13833ec7a7a27af2d29e05e`
- `internal/adapters/sfwp/scoped_runs_test.go`: `57bfc686da998303655bd99fdc5bffcbf386ddbcb931ea8bc290762d848fad5e`
- `internal/projection/scoped_live_test.go`: `a9e0deb03f69b12ab379e547de2fb2aaf20c13dd0a51929cdeeda0bea14e7116`
- `internal/projection/unreadable_runs_test.go`: `de85a2edb9bd05dc5542c7c9850a9cf9caea69dbb491eb21f95fe93913bfcebb`
- Preserved pure child fixture `internal/projection/run_children_test.go`: `4ddb2445a7ff6726c3b6fd93832ac4122b53da4d26e5b54180f9a1cafb802741`
- New live fixture `internal/projection/scoped_runs_live_test.go`: `8e485c64cdc0776d46a8fda6678d22f3bfe98ec837bd8eaaeb94fecced8a9134`

The production diff is restricted to `authority.go` and `live.go`, addressing the two defects in `run-children-phase-b-independent-semantic-reject.md`. No extra production behavior or API change was found in this phase.

## Source review

The source repair addresses the previously rejected semantics:

1. `LiveSource.Facts` now locates exactly the requested case record and returns typed Invalid immediately if absent, before overview, horizon, approvals, or scoped-run calls (`live.go:43-79`). Repeated matching case records fail unavailable. This matches the Phase 2 requirement regardless of whether unrelated cases are present.
2. Scoped run refusals are normalized only when `errors.As` finds a `Refusal` whose exact `RefusalClass()` is `unavailable` (`authority.go:309-315`). Other classes retain the original `Refusal.appErr` mappings and wrapped refusal, including Invalid, AuthorityDenied, Unavailable, and Internal. Both adapter error sites use the same helper (`authority.go:261-274`). The ordinary concrete-client route is tested; `Response.Into`’s defensive `resp.Err` branch is source-reviewed but cannot be reached for ordinary refusal frames through `Client.Do`, which returns the refusal error before returning the response (`client.go:376-397`). No runtime coverage claim is made for that defensive branch.
3. The remaining scoped projection path validates requested case, selected refs, exact scoped list call, readable ownership, actual horizon parent, distinct readable/unreadable IDs, accepted standings, and nonnegative evidence count (`live.go:43-139`). Adapter conversion validates required arrays, row fields, ownership, standings, timestamps, evidence count, duplicates, overlap, and preserves all returned unreadable IDs (`authority.go:276-364`).
4. `runChildren` emits one child per valid unique captured run, uses the factual run ID and actual horizon item as ParentID, omits duplicate/malformed/foreign/missing-parent rows and ID collisions, preserves siblings, has no child-count cap, and creates no actions or live observation fields (`builder.go:131-175`). `Build` calculates the original attention focus before appending children (`builder.go:93-108`).
5. The preserved pure fixture covers all 24 execution/settlement pairs, distinct labeled statuses, no invented completed→accepted settlement, no child actions, no hydration truncation, parent actions/focus/narration/relative item rank/progress preservation, invalid and duplicate child omission, preservation of stage/item objects on ID collisions, and immutable retained historical run standing (`run_children_test.go:13-249`). The separate unreadable-ID Store test checks cloning on append and on returned reads (`unreadable_runs_test.go:9-35`).
6. The new live fixture follows the approved real-kernel supplement: it checks the existing server binary before constructing a cell, commits two actual cases, executes `task_prepare` with distinct IDs, captures actual run IDs, checks each run occurs exactly once in its owner list and never in the foreign list, compares source facts to actual list summaries, and verifies actual child parent, separate standings, no actions, and no foreign child (`scoped_runs_live_test.go:17-125`). It does not write synthetic run records or case indexes.

## Blocking finding: live fixture cannot compile in its current package

`scoped_runs_live_test.go` declares `package projection` and imports `internal/livestack` (`scoped_runs_live_test.go:3,12-13`). `internal/livestack/stack.go` imports `internal/projection`. Building tests for package `projection` therefore creates the cycle:

```text
internal/projection test → internal/livestack → internal/projection
```

The authorized command compiled far enough to report the Go test package setup error before executing the test body. As a result, no disposable cell, socket, case commit, item execution, or real-kernel assertion ran. This is a fixture defect, not evidence about kernel run scoping. A fresh fixture builder should remove the import cycle, for example by making the live fixture an external `projection_test` package and providing a test-local equivalent of the currently unexported `hasLabelledStanding` helper. That repair should keep every assertion and the test name intact, then receive a fresh independent source review and live run.

Because the required direct real-kernel proof cannot currently run, the bounded unit 4 evidence is insufficient for approval. The focused package race pass below does not waive this required proof.

## Commands, preflights, and results

All test commands used `GOMEMLIMIT=256MiB`, `GOGC=50`, `GOMAXPROCS=2`, `GOFLAGS=-p=1`, `GOLDEN_UPDATE=0`, `-race -count=1 -parallel=1` and actual command exit files.

Focused unit 4 tests and regressions:

```text
go test -race -count=1 -parallel=1 ./internal/adapters/sfwp ./internal/projection -run 'TestScopedRunsList|TestLegacyRunListRequestRemainsUnscoped|TestLiveSource|TestBuild.*RunChildren|TestStoreKeepsRunChildStandingAtCapturedCursor|TestStoreDeepCopiesUnreadableRunIDs$'
```

Actual exit status: `0`; both packages passed. This includes the Phase 1 expected-RED assertions, all scoped adapter malformed/refusal tests, scoped live-source validation, pure run-child regressions, and unreadable-ID clone proof.

Required live proof attempt:

```text
TMPDIR=/tmp/t09u4 go test -race -count=1 -parallel=1 -tags live ./internal/projection -run '^TestLiveRunScope$' -v
```

Actual exit status: `1`, package setup failed with the import-cycle diagnostic above. The live preflight verified the existing executable `target/debug/sea-forge-server` was regular/executable, 51,456,400 bytes, SHA-256 `c17b4c1458c019a7527305cd6313638c3143e1c06e95af37c13659b0cf4603fc`; no hidden Cargo build was attempted. Because the test did not enter its body, there is no observed socket path or kernel proof result.

Immediately before each command an actual-host preflight captured UTC time, RAM, and a compiler-process scan. Both scans found no Go/Cargo/Rust/Bun compiler process. The only match was PID 2109808, a Bun `server.ts` service, left untouched. The preflight captured 2.3 GiB available memory. Each preflight/output/exit was separately redirected under `/tmp` and archived to new evidence paths below.

## Raw artifact identity

Each original capture and its new archive was independently hashed and matched byte-for-byte:

| Original `/tmp` path | New evidence file | SHA-256 |
|---|---|---|
| `/tmp/t09-unit4-phase2-focused.raw` | `unit4-phase2-focused.raw` | `77a005a93c0c350c922e831b08af7a2c4a4e48f61f3dc43384edbe1be69e1ad0` |
| `/tmp/t09-unit4-phase2-focused.exit` | `unit4-phase2-focused.exit` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| `/tmp/t09-unit4-phase2-focused-preflight.raw` | `unit4-phase2-focused-preflight.raw` | `cf4cdc0814fce72b67b66f8949a5fd36df374a21c2ee5243fd8ba4faaa04a931` |
| `/tmp/t09-unit4-phase2-live.raw` | `unit4-phase2-live.raw` | `419d60d9ba001586a57931405f52564e351b54143ddb822b7bf4bbfa20eb9025` |
| `/tmp/t09-unit4-phase2-live.exit` | `unit4-phase2-live.exit` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |
| `/tmp/t09-unit4-phase2-live-preflight.raw` | `unit4-phase2-live-preflight.raw` | `a9e65debcadcc6f7960dcf21b275fd8681025a5607dd65f284818e6f7d0c3d7c` |

## Disposition

**REJECT unit 4 for the current frozen source/test graph.** The production repairs and focused tests are materially sound under the reviewed requirements, but the required real-kernel test does not compile due to the package cycle. No broader package/module gate was run after this source/test defect was found. Compiler token is released to root. A fresh builder must repair only the live test package boundary; then an independent review must rerun the focused tests, live proof, and required canonical/full Go gates. Full T09 remains pending.
