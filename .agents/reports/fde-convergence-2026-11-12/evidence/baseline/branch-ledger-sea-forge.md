# SEA-Forge branch ledger (WP-00c), 2026-11-13

Base: origin/main `09b628ec` (#97, status revision 233). Integration branch: `fde/integration` (worktree `~/projects/_wt/fde-sea-rs-integration`), merge commit `939eaf80` = main + `feat/horizon-desk-support` (`6e8b6aff`, PR #91, open). No PR to main, no merge to main. The operator checkout was only read.

## Method

For every local branch not merged into origin/main (35, from `git branch --no-merged origin/main`):

1. `git merge-tree --write-tree origin/main <branch>`, then diff the result tree against main's tree, ignoring `.agents/CURRENT_STATUS.*`. A zero delta means the branch's content is already on main. This is how squash merges (which hide patch-ids) were seen through.
2. For branches with a non-zero delta, per-commit reverse-apply (`git show <c> | git apply -R --check` on main) and a search of main's subjects and content markers.
3. For the pre-#67 cross-platform family, the same merge-tree test against `0fc236f3` (the #67 squash), plus reading the pass-1/2/3 review record in `.agents/plans/2026-10-25-cross-platform-native-execution.md`.

Limits of the method: "superseded" for the large pre-#67 and settlement chains rests on tree-delta, sampled commit reverse-apply (10 of the review2 commits) and the review record. It is not an exhaustive per-commit proof.

## Verdicts

Verdicts: MERGED (into fde/integration), LANDED (content on main, delta 0), SUPERSEDED (replaced by a main commit that changed the same code further), KEPT OUT.

| Branch | Tip | Verdict | Reason |
|---|---|---|---|
| feat/horizon-desk-support | 6e8b6aff | MERGED | PR #91 head; 5 commits, 48 files, no source conflicts (only status files). Merge commit 939eaf80. |
| fix/settlement-orphan-guard | 15c7ccf6 | LANDED | #89 `7db070a5`. Its last commit (versioned dev-dep, `allow(dead_code)` on non-unix) is on main: `crates/sea-forge-authority/Cargo.toml:29` has version 0.4.1, `command_launch.rs:122` carries the cfg_attr. |
| test/settlement-class-fail-closed | f9fe5818 | LANDED | #88 `bb0190ab`; merge-tree delta 0. |
| fde/domainforge-0.21.2 | 5cad1166 | LANDED | #94 `0439abb0`; delta 0. |
| fde/fix-e2e-world-loop | 929daf03 | LANDED | #97 `09b628ec`; tree identical. |
| fde/rustsec-2026-0459-0486 | 4dba46ed | LANDED | #95 `c2d6fbdc`; delta 0. |
| fix/b12-seed-cell-and-backup | 79bb05eb | LANDED | #65 `7f4be4b8`; delta 0. |
| fix/go-jose-4.1.6 | d0177df8 | LANDED | #78 `11efa4b1`; delta 0. |
| security/identity-binding-version | af02252b | LANDED | #77 `c30e51ed`; delta 0. |
| t1/case-task-approval | d5e34c54 | LANDED | #87 `1fe00c9c`; delta 0. |
| t1/rename-case-task-approval | 5d1338c7 | LANDED | #87 (duplicate of the above); delta 0. |
| fde/planning-baseline | 0bab4bd2 | OPEN PR | PR #98 (planning docs only, status revision 234). Not mine to merge here. |
| t1/contracts-home | 968273fe | SUPERSEDED | #85 `b94e810a` moved the schemas; #87 then renamed PendingApproval to CaseTaskApproval, so the branch's `PendingApproval.*` files conflict. `contracts/` exists on main. |
| fix/b08-domainforge-engine-fail-closed | 9268aacc | SUPERSEDED | #66 `4e67dfca` (same fix, plus removal of the stem-heuristic candidate); conflicts in `cep.rs`, `lib.rs` are later main edits. |
| fix/server-ambiguous-declare-keeps-lease | 961dd478 | SUPERSEDED | #84 `bdb42a52`; operator `--attest-command-gone` path is on main (`command_launch.rs`, `cep.rs`, `cli/commands/approve.rs`). |
| fix/settlement-attempt-claim-settlement-main | ccac67a2 | SUPERSEDED | #81 `09c2541b`. |
| fix/settlement-attempt-claim-settlement | 749a2173 | SUPERSEDED | 22-commit chain (settlement authority class, durable pending, claim-once, F1-F6 fixes) squashed as #79 `652ed710`, #81, #82 `f947e8e6`, #83 `22bf66a3`. Contains a `wip` checkpoint commit (167f9fbf); do not port. |
| fix/restore-release-attestation | 09ff8cf7 | SUPERSEDED | #64 `d04be702`; `release-please.yml` on main has the attestation steps (lines 103-104). |
| ci/release-provenance | 730add6e | SUPERSEDED | #58 `a9147097` (+ #64). |
| feat/xplat-consolidate | e7e275d9 | SUPERSEDED | #67 `0fc236f3` (cross-platform native execution). Pre-#67 stack; 18-33 non-status conflicts against main. |
| feat/xplat-win-sandbox-tests | 69397b4c | SUPERSEDED | #67. |
| feat/xplat-hardening | 129748db | SUPERSEDED | #67. |
| feat/openshell-capability | 73516733 | SUPERSEDED | #67 (OpenShell kept as governed backend). |
| feat/macos-app-sandbox-jail | e0fd2cf7 | SUPERSEDED | #67 (macOS App Sandbox jail). |
| feat/native-packages | cda24a8a | SUPERSEDED | #67 (deb/pkg/dmg/exe packaging, `package.yml`). |
| fix/review1-sandbox | 5641b4ee | SUPERSEDED | #67; review pass 1 resolutions are in the plan record. |
| fix/review1-windows | 4aaef3b4 | SUPERSEDED | #67. |
| fix/review2-sandbox | 4ba67d34 | SUPERSEDED | #67; sampled commits 156f54dc, 53c3bd9a, 72aaf96e, c7cce85d reverse-apply cleanly on main. |
| fix/review2-windows | a9cb3636 | SUPERSEDED | #67; tip a9cb3636 reverse-applies cleanly on main. |
| ci/openshell-live | f97199c9 | SUPERSEDED | #67 carries `.github/workflows/openshell-live.yml`; fixture regeneration (5c9350ce) and mTLS ephemeral keys (ea3a2b0c) reverse-apply on main; the rest conflict with later main edits in `openshell.rs`, `cell_episode_live.rs`. |
| feat/hardening-capability-harvest | 700c264e | KEPT OUT | 36 commits, about 7.7k inserted lines, post-#67 (2026-10-26/28). Closes the open #67 findings F05, F08, F15, F17, F32 (cgroup containment, pinned dirs, provable egress denial, lifecycle teardown), three codex review passes. Large and security-sensitive: needs its own PR and a `sea-forge-hardening-review`, not a drive-by port. It also touches `Cargo.toml`s (dependency changes need approval). Conflicts with main in 4 files (see Conflicts). Highest-value unmerged branch. |
| feat/arm64-jetson-packaging | e096e9c1 | KEPT OUT | Linux ARM64 packaging target (`package.yml`, `release-please.yml`, `packaging/deb/build.sh`, `arm-probe.yml`). Merges cleanly, but CI/release changes need approval and ARM64 is a product call. |
| ci/openshell-windows | e658b030 | KEPT OUT | One exploratory workflow building OpenShell for Windows with the MXC driver; #67 records SEA-Forge on MXC as blocked (no exec/upload). |
| ci/macos-app-sandbox-probe | 582162ef | KEPT OUT | One-off evidence probe (`probes/macos-app-sandbox/`, workflow); #67 includes the macOS lane and its findings. Abandoned scaffolding. |
| docs/egm-bootstrap-readmes | b072ec50 | KEPT OUT | The operator's current checkout branch: 151 README/contract doc files plus a drift report. Not mine to move or merge; no relation to the Horizon merge. |

Counts (35 branches): 1 merged, 10 landed, 18 superseded, 5 kept out, 1 open PR. Nothing was ported: every small fix that looked missing was found on main.

## Conflicts and resolutions

- Merge of `feat/horizon-desk-support`: only `.agents/CURRENT_STATUS.md` and `.agents/CURRENT_STATUS.yaml` conflicted. Resolved by keeping main's revisions (through 233) and adding revision 234 for the integration branch (the commit hook requires a status update with project changes). The horizon branch's own revision 235 was not carried; its two limits not already on main were copied into revision 234.
- `feat/hardening-capability-harvest` against main (not merged): `crates/sea-forge-authority/tests/cep_compiler_upgrade.rs`, `crates/sea-forge-domainforge/Cargo.toml`, `crates/sea-forge-server/Cargo.toml`, `crates/sea-forge-server/tests/conformance_cep_authority.rs`. The first, second and fourth come from main's 0.21.2 lockstep (#94) overlapping the branch's own `domainforge-core 0.21.2` bump (`56c6b51e`).

## Gate

Env: `RUSTC_WRAPPER= CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_{DEV,TEST}_DEBUG=0`, cargo under `flock -o ~/projects/_wt/cargo.lock`, `SEA_FORGE_SECURITY_PG_INSTALL=~/.pg0/installation/18.1.0`. Crates touched by the merge: core, planner, case-runner, cli, server (from `git diff --name-only`).

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo check --workspace --locked` | pass |
| `cargo test -p sea-forge-core --locked` | pass (104 passed) |
| `cargo test -p sea-forge-case-runner --locked` | pass (35 passed, 17 ignored: live prerequisites) |
| `cargo test -p sea-forge-cli --locked` | pass (90 passed) |
| `cargo test -p sea-forge-server --locked` | pass, 0 failures |
| `cargo test -p sea-forge-planner --locked` (first run) | FAIL to compile: `conformance_m2.rs` missing field `review` in two `ItemMarkers` literals (E0063). Caused by PR #91 adding `ItemMarkers.review` (`crates/sea-forge-core/src/types.rs:74`); `cargo check --workspace` does not build test targets, so it did not show it. Absent on main. |
| same, after the fix | pass (69 passed). |

Fix ported on `fde/integration` (`6c760a63`, status revision 235): `review: false` added at `crates/sea-forge-planner/tests/conformance_m2.rs` (two literals). PR #91 needs the same one-line fix before it merges.

Not run: `cargo test --workspace` / `just ci`, clippy, `just proof`, `workbench-check` (bun), the MSRV 1.92 job, live OpenShell/Windows/macOS lanes, `SEA_FORGE_REQUIRE_*` strict-mode runs (isolation and Postgres tests therefore may have skipped inside the server suite), and a baseline run of the same suites on bare main (acceptance was read as "no failure that main would not also have"; the one failure is a compile error traceable to the merged branch, not a main failure). Test targets of crates outside the five (e.g. any other `ItemMarkers` literal) were not compiled.

## Unfinished work remaining

- Open #67 findings, still OPEN on main (`.agents/plans/2026-10-25-cross-platform-native-execution.md`): F05 line 419, F08 line 421, F15 line 423, F17 line 425, F33 line 426. The `feat/hardening-capability-harvest` branch (final report in its `.agents/plans/2026-10-26-capability-harvest.md`) addresses these and is the route to close them.
- PR #91 itself: its status entry carries a limit that the `case.add_item` precondition is checked outside the write lock; the code at `crates/sea-forge-server/src/sfwp/case_mutations.rs:235-278` now reads and checks under the case lock (the blind-critic fix `6e8b6aff`), so that limit text looks stale. The branch also notes the generated TypeScript contracts under `workbench/packages/contracts/generated/` were regenerated in `cefa10a1` but `bun` checks were not run here.
- PR #98 (planning baseline) and PR #91 both want a status revision after 233: whichever merges second needs a renumbered revision (this branch's 234 collides with #98's 234 and #91's 235; irrelevant while this branch is not merged).
- Revision 233 limits carried unchanged: `cargo test --workspace` and `just ci` not run for #97; the MSRV 1.92 job is CI-only.
