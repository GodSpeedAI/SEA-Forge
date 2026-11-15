# Readiness diagnosis: U01 and J02R2 (read-only, 2026-11-13)

No gates, builds, tests, installs or services were run. Labels: OBSERVED = read from files or logs; INFERRED = reasoned from them.

## U01: "Workbench did not become ready within 90s"

### Start path (OBSERVED)
- `godspeed-readiness/probes/workbench/u01.sh:3-5` makes a temp home, sets `PORT` to `GSR_U01_PORT` (default 3195), and runs `trap cleanup EXIT`. `cleanup` does `bun src/cli.ts down` and then `rm -rf "$HOME_DIR"`. Line 5 runs `bun src/cli.ts up || exit 1`.
- `godspeed-readiness/src/up.ts:171-172` starts Workbench with `startDetached("workbench", [process.execPath, "src/server.ts"], p.workbench, {...})`. The env is `PORT`, `COGNATE_WORKBENCH_HOME`, `COGNATE_POSTGRES_URL`, `COGNATE_LINUX_CONTAINMENT_CONFIG`, `COGNATE_GOVERNANCE=sea-forge`, `COGNATE_SEA_FORGE_SOCKET`, `COGNATE_WORLD_REF`, `COGNATE_REALITYTRACE_*`, `SXR_BIN`, `COGNATE_SWE_SEED_BIN` and `COGNATE_PROOF_SOURCE`.
- `up.ts:192` readiness is `fetch("http://127.0.0.1:3195/")` polled for 90 s, requiring `.ok`.
- `up.ts:86-94`: stdout and stderr go to `LOGS/workbench.log` (`LOGS = HOME/logs`, `up.ts:16`).

### Why the log is missing (OBSERVED, correcting the brief)
- The log did exist. `up.ts:102` reads `workbench.log` with `readFileSync`. A missing file would throw ENOENT, not print "last log lines:". The tail was blank, so the file was empty.
- The path is deleted on exit. `up` calls `fail()` (`up.ts:103`) and `exit(1)`, which fires the `u01.sh:4` EXIT trap and runs `rm -rf $HOME_DIR`. The "fix: see .../logs/workbench.log" hint points at a file that no longer exists by the time the operator reads it.

### Why Workbench is silent (OBSERVED: code; INFERRED: runtime)
- `cognate-workbench/src/server.ts` at 1b6fcb4 had a start block at lines 884-887: `if (import.meta.main) { ... startServer({ port, home }) }`.
- Commit `fe062a5` ("feat(cli): cognate-workbench start, doctor, --version and --help", 2026-10-30) is in `1b6fcb4..HEAD`. It removed that block from `server.ts` (HEAD has no `import.meta.main` in server.ts). The start command moved to `src/cli.ts:286` (`if (import.meta.main) main(...)`) and `cli.ts:91` (`PORT`, default 3180) and `cli.ts:92` (`COGNATE_WORKBENCH_HOME`). `server.ts` is now a library exporting `startServer` (HEAD:363).
- So `bun src/server.ts` loads the module, runs no listener, and exits 0 with no output. Port 3195 never opens and the 90 s poll fails. This matches the empty log tail.
- INFERRED confidence: high (~85%). I did not start the process, per the constraints.

### Ruled out (OBSERVED)
- The DomainForge lockstep commit `74601f3` touches tests only ("live SEA-Forge tests"). It is not on the startup path. The `gsr up` steps before Workbench (pg0, sea-forge socket, world ref, log lines 1-10 of the 13-01 run) all passed.
- Env names `PORT`, `COGNATE_WORKBENCH_HOME` and `COGNATE_POSTGRES_URL` are still honored by `cli.ts`.

### Version skew (OBSERVED, secondary and unverified)
- Workbench `package.json` moved `@godspeedai/cognate-*` from 0.7.1 (1b6fcb4) to 0.9.0 (HEAD). The harness's `sea-forge-server` is built from gsr-heads/sea-forge (pinned 0.21.1 per the brief). Whether SEA-Forge accepts the 0.9.0 client and world ref is untested. It matters only after the start fix.

### Fix (U01)
- Smallest: harness, `godspeed-readiness/src/up.ts:171` → `[process.execPath, "src/cli.ts", "start"]`, with `cwd` the Workbench checkout. Optionally also `u01.sh` should not delete `HOME_DIR` on failure, so the log survives. Owner: harness.
- Alternative (Workbench owner): restore an `import.meta.main` start block in `server.ts`. Less preferred, since `cli.ts start` is the documented entry (`package.json` "start").

## J02R2: `git commit failed: fatal: cannot lock ref 'HEAD'` and `.git/config: File exists`

### Throwing code (OBSERVED)
- `horizon/src/adapters/realitytrace-obligation.ts:119-142` is `repo()`:
  - `:121` returns `this.repoId` only if it is already set.
  - `:122` `mkdirSync(repoDir)`.
  - `:123` `git rev-parse --git-dir`. If it fails, `:124-127` run `git init` and then `git commit --allow-empty`, throwing `RealityTraceError` at `:125` and `:127`.
  - `:140` assigns `this.repoId` only after `await`. There is no in-flight memo, so concurrent callers each run init and commit.
- `obligationRt` is built at `horizon/src/app.ts:409` with `repoDir: \`${realityTrace.repoDir}-obligations\``. That is the `rt-repo-obligations` path, derived from the test rig's `root/rt-repo` (`test/support/j02r2-rig.ts:109`).

### The two concurrent callers (OBSERVED)
- `src/app.ts:410` (and `:609` in the Horizon-gaps copy): `void obligationRt?.snapshot();`. It is fire-and-forget.
- `snapshot()` (`realitytrace-obligation.ts:153-172`) awaits `repo()` at `:161`. Its errors are swallowed at `:168-170` (`rulebookOk = false`).
- The caller's path is `commitNavigationIntent` (`src/app.ts:781`) → `settlement/obligation.ts:137` `declare()` → `realitytrace-obligation.ts:187` `repo()`.
- So a cell's first `commitNavigationIntent` can start `repo()` while the `snapshot()` from construction is still running `git init`/`git commit`. Both see no repo, both init and commit, and the second commit hits the ref and config locks. This matches both error messages.
- In the run, tests call `cell()` and then commit right away (`test/J02r2-live.test.ts:69-70`, `:101-102`, `:123-124`). `bun test` runs tests serially, so the race is inside a cell. That explains why only some tests fail (5 pass, 3 fail in 13-02).

### What changed (OBSERVED)
- `b7ce7f4` (2026-11-07, "feat(records): runs begin on the real rulebook snapshot id") is after the last pass (`55f1b42`, 2026-10-25). It is not an ancestor of 55f1b42.
- Before `b7ce7f4`, `repo()` was synchronous (`spawnSync`), so the race could not happen. `b7ce7f4` made it async and added `snapshot()` plus the `void` call at `app.ts:410`.
- RealityTrace's `4d05cc0..90eff85` only adds `repo enroll` in its own scripts and tests. Horizon never touches the RealityTrace repo, so this is not the cause.

### Horizon-gaps (OBSERVED)
- `diff -q` shows `realitytrace-obligation.ts`, `settlement/obligation.ts` and `test/support/j02r2-rig.ts` are identical to gsr-heads/horizon. There is no memo or lock fix in the adapter.
- `app.ts` differs only by `if (!options.readOnly) void obligationRt?.snapshot();` (gaps:609). The test cells are not read-only, so the race still applies.

### Confidence and fix (J02R2)
- Confidence: ~80% for the race. Timing-dependent, so not yet reproduced here.
- Smallest fix, owner Horizon: memoise the in-flight promise in `realitytrace-obligation.ts:119-142`. Use `private repoP: Promise<string> | null = null`; `repo()` returns `this.repoP ??= this.enrol().catch((e) => { this.repoP = null; throw e; })`, where `enrol()` is the current body. Optionally also `await` the `snapshot()` kickoff in `app.ts:410`, but the memo alone removes the race.

## Summary
- U01: Workbench's `src/server.ts` lost its start block in `fe062a5`, so the harness's `bun src/server.ts` exits silently. Fix in the harness (`up.ts:171`), owner harness. Confidence ~85%.
- J02R2: `repo()` has no in-flight memo, and `app.ts:410` fires `snapshot()` concurrently with the first declare. Introduced by `b7ce7f4`. Fix in Horizon (`realitytrace-obligation.ts:119-142`). Confidence ~80%.
