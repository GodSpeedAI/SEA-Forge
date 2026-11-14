# GodSpeed readiness run 2026-11-13: diagnosis of 11 non-passing gates

Scope: read-only. Run: 2026-11-13T06:59-07:20 UTC, NO-GO 19/30. Evidence: `/home/sprime01/projects/godspeed-readiness/evidence/runs/2026-11-13T0[67]*/` (manifest.json plus per-gate logs). Console: `/home/sprime01/projects/sea-rs/.agents/reports/fde-convergence-2026-11-12/evidence/baseline/readiness-run-3.log`.

## 1 Summary

| Gate | Outcome (duration) | Proximate cause (evidence) | Class | Owner | Smallest fix |
|---|---|---|---|---|---|
| C01 | FAIL (23 ms) | `C01 FAIL: DomainForge versions disagree: 0.21.1, 0.21.2` (`runs/2026-11-13T07-02-31-907Z/C01.log`) | (b) | sea-forge, cognate, horizon (0.21.1) vs realitytrace (0.21.2) | Pick one lockstep version (see section 3) |
| L01 | BLOCKED (probe) | probe `${domainforge}/target/release/domainforge --version \| grep -q ' 0.21.1$'` fails; binary prints `domainforge 0.21.2` (manifest `07-04-27-105Z`) | (b) | same as C01 | same as C01; probe string follows the chosen version |
| SW01 | FAIL (2,694 ms) | `DomainForge CLI .../domainforge reports "domainforge 0.21.2", but the live SEA-Forge tests need domainforge 0.21.1` (`SW01.log`) | (b) | same as C01 | same as C01 |
| SW02 | FAIL (1,089 ms) | same message (`SW02.log`) | (b) | same as C01 | same as C01 |
| J02 | FAIL (71,375 ms) | `RealityTrace refused the observation: realitytrace: sxr ingest failed (exit 2): realitytrace: unknown --stdin-format "json"; supported: ... agent-event-v1` (`07-03-01-812Z/J02.log`) | (b) | cognate client (sends json) and realitytrace head (dropped json) | Cognate: send `--stdin-format agent-event-v1` (horizon `scripts/seed.ts:857` already does) |
| J02R2 | FAIL (11,545 ms), 6 failing tests | `RealityTraceError: git commit failed: fatal: cannot lock ref 'HEAD': reference already exists`; `could not lock config file /tmp/horizon-j02r2-Bx9nin/rt-repo-obligations/.git/config: File exists` (`07-04-13-766Z/J02R2.log`) | (e) | horizon test fixture | Give each test its own temp repo; rerun J02R2 alone |
| U01 | FAIL (92,132 ms) | `Workbench did not become ready within 90s`; log named as `.tmp-u01-RhTzRh/logs/workbench.log`, which does not exist (`07-10-22-668Z/U01.log`) | (e) | cognate-workbench start / harness | Capture Workbench stdout on an idle host and rerun |
| BM-STACK | FAIL (94,928 ms) | `assertion failed: stack-up-exit-0 (gsr up exit 1)`; stderr empty; time_to_ready and rss missing (`07-18-42-056Z/BM-STACK.log`) | (e) | same as U01 | Same as U01 (likely the same Workbench start failure) |
| BM-SETTLE | FAIL (273,072 ms) | settled 0 of 10; `evidence or settlement refused: no execution evidence was submitted for op-...` (`07-14-08-247Z/BM-SETTLE.log:2`) | (b), unverified | as J02 | As J02; confirm the ingest path first |
| BM-AUTH | FAIL (7,161 ms) | `auth.c4.p95_ms = 168.431 exceeds max 112`; c1 p95 22.391 is within 25.5 (`07-13-24-334Z/BM-AUTH.log`) | (e) | sea-forge perf / harness budgets | Rerun 3x on an idle host; treat as regression only if it reproduces |
| CK01 | BLOCKED (setup step 0, exit 101) | `error: package ID specification 'ck-bin' did not match any packages` (`07-02-35-880Z/CK01.setup.log`) | (d) | godspeed-readiness scenario | Change `-p ck-bin` to `-p context-kernel` (scenario lines 6 and 30) |

Passes are not affected by the failures above. Fresh passes are listed in section 4.

## 2 Shared root causes

- **G1, DomainForge version skew (4 gates: C01, L01, SW01, SW02).** Real cross-repo drift, not a broken binary. Every product builds or expects one version, and the set is split.
- **G2, RealityTrace ingest format drift (2 gates: J02, BM-SETTLE).** `cognate/packages/realitytrace/src/sxr-cli.ts:133` sends `--stdin-format json`. RealityTrace commit `b616335` (2026-10-30, `feat(ingest): agent-event-v1 ingest`) removed `json`. Its supported list is now `cargo-json, cargo-test-json, libtest, car-go-json, pytest-progress, df-validate, corpus, agent-event-v1`. The previous GO (2026-10-25 heads) predates this commit, so this is origin-head drift. Horizon's `scripts/seed.ts:857` and `test/economics-live.test.ts:62` already use `agent-event-v1`.
- **G3, Workbench not ready (2 gates: U01, BM-STACK).** Cause not determined (section 5).
- **G4, stale harness (1 gate: CK01).** The package was renamed. Single gate.
- Singles: J02R2 (test isolation, likely), BM-AUTH (perf, likely noise until reproduced).

## 3 C01 version table

Expected: all products pin one DomainForge producer version.

| Product (gsr-heads worktree) | Pinned DomainForge version | Declared at |
|---|---|---|
| sea-forge | `=0.21.1` (Cargo) | `sea-forge/crates/sea-forge-domainforge/Cargo.toml:13`; `sea-forge/crates/sea-forge-domainforge/src/lib.rs:101` (`EXPECTED_DOMAINFORGE_VERSION`); `sea-forge/devbox.json:22` (`cargo install domainforge-core --version 0.21.1`) |
| cognate | 0.21.1 | `cognate/packages/domainforge/package.json` (version reported by C01.log; line not extracted) |
| horizon | 0.21.1 | `horizon/bun.lock` (reported by C01.log; line not extracted) |
| realitytrace | `0.21.2` | `realitytrace/sxr-df/src/lib.rs:12` (`PINNED_DOMAINFORGE_VERSION`); asserted at `:23` |
| domainforge (head `f90905a`) | Cargo `0.21.2` | `domainforge/domainforge-core/Cargo.toml:3`; `.release-please-manifest.json` `"."` = `0.21.2` |

DomainForge head facts:
- Latest tag `v0.21.2` (2026-10-25 23:35 -0400). HEAD is 11 commits past it.
- `git describe --tags` returns `fde-backup/main/20261113`, a backup tag, so it is misleading here.
- Release PR #165 `chore: release 0.22.0` (branch `release-please--branches--main--components--domainforge`) is open. Nothing pins 0.22.0 yet.
- `target/release/domainforge` (built 2026-11-13 02:02:29) reports `domainforge 0.21.2`.

Fix options (operator decision):
- A: move sea-forge (Cargo.toml:13, lib.rs:101, devbox.json:22), cognate and horizon to 0.21.2, then update the L01 probe and the C01 expected list. This follows the current tag.
- B: pin realitytrace back to 0.21.1 (`lib.rs:12` and `:23`). This is a downgrade.

**SEA-Forge post-checkout hint.** The hint appears in `readiness-run-3.log` lines 10-12 ("dependency/toolchain files changed ... next move: just sync"). The hook is advisory and does not sync (`sea-forge/.githooks/post-checkout`). The sea-forge reflog shows checkout to `595b5270` at 2026-11-13 01:58:16. The `sea-forge-server` binary was rebuilt at 02:04:05, and `domainforge` at 02:02:29 after its 01:58:41 checkout. So the binaries were rebuilt after the checkout. The hint does not imply stale binaries. I did not confirm which step performed the rebuild.

## 4 Fresh vs carried-over passes

The manifest has a record for all 30 gates in this run. Each of the 19 passes has its own run directory with a PASS record from this run, so all 19 ran fresh at the pinned heads. No pass was counted from older evidence.

Passes: BM-RTO, CK02, G01, O01, O02, O03, O04, R01, R02, S01, S02, S03, U02a, U02b, U02c, U02d, U02e, U02f, U02g.

Caveats:
- O01 is evidence citation, not local execution. It cites GitHub Actions runs at pinned SHAs (for example `SEA-Forge@595b5270` run 42173525229). It was re-read in this run, not re-executed.
- O02, O03 and O04 are local backup and restore runs.
- BM-RTO's 20 s bound is inherited from sea-forge.

Duration (wall clock, from manifest) for the failing gates: C01 23 ms; CK01 setup failed by 07:02:37; J02 71,375 ms; J02R2 11,545 ms; L01 probe at 07:04:27 (no duration); SW01 2,694 ms; SW02 1,089 ms; U01 92,132 ms; BM-AUTH 7,161 ms; BM-SETTLE 273,072 ms; BM-STACK 94,928 ms.

## 5 What you could not determine

- **U01 and BM-STACK (G3).** Workbench logs are absent. `gsr up` stderr is empty. Memory was tight at start (`mem_available_mb.before = 4128`), but this is not confirmed as the cause. Next step: run `gsr up` alone on an idle host and capture the Workbench stdout.
- **BM-SETTLE.** I did not trace the evidence-submission path. G2 is the likely cause but is unverified.
- **J02R2.** I did not check whether the `/tmp/horizon-j02r2-*` fixture is shared between tests or left over from an earlier run.
- **BM-AUTH.** Whether the 168 ms p95 is a regression or host noise is not determinable from one run. The budget provenance (2026-10-24 baseline) and the debug-profile build were not compared.
- **CK01.** The scenario cites a pinned Context-Kernel head, `harden 3111d26`. That commit exists, but the checkout is `main` at `36bd8c8`. The rename to `context-kernel` is in `26ef9d8` (2026-10-31).
- **Cognate and horizon DomainForge lines.** Exact file:line values for these two were not extracted.
- **RealityTrace `json` removal.** I could not tell whether dropping `json` in `b616335` was intentional or a side effect.

## Suggested fix order

1. Decide the lockstep DomainForge version (option A or B). This clears C01, L01, SW01 and SW02.
2. Move cognate's RealityTrace client to `agent-event-v1`. This clears J02 and probably BM-SETTLE.
3. Fix the CK01 package name in godspeed-readiness.
4. Investigate U01/BM-STACK on an idle host. Isolate J02R2's fixture. Repeat BM-AUTH.
