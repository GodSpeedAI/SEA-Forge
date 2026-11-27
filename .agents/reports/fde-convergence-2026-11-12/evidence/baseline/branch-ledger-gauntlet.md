# Branch ledger: Gauntlet, WP-00c

Integration branch: fde/integration @ 846830d (merge commit of origin/release/native-distribution onto origin/main 1543cbf; pushed). Worktree ~/projects/_wt/fde-gauntlet-integration.

| Branch | Tip | Verdict | Reason |
|---|---|---|---|
| release/native-distribution (PR #7, OPEN, MERGEABLE) | 5842777 | merged | 12 commits, fast-forwardable; merged --no-ff, no conflicts |
| fix/test-schema-teardown | 2710a0e | landed | PR #6 (1543cbf); reverse-apply succeeds |
| jollimemory/summaries/v3 | 5f538f5 | kept out | tooling summary branch |

## Gate (nextest --workspace --no-fail-fast, pg0 gauntlet-dev on 54329; cargo fmt --check)
- fmt rc=0.
- Final run 1: 2223 run, 2191 passed, 32 failed, 3 skipped. 26 failures were codemode tests needing workbench node_modules ("Cannot find package quickjs-wasi"); fixed by `bun install --frozen-lockfile` in the new worktree, rerun of those binaries: all pass.
- Remaining 6 failures, gauntlet-cli::conformance_cli: "gauntlet verify --all: reading the tree's HEAD: Not a directory". Cause: crates/gauntlet-cli/src/verify_cmd.rs:577-579 (head_commit) reads root/.git/HEAD, which fails in a git worktree where .git is a file. The code is identical on origin/main, so these fail in any worktree; not caused by PR #7. Treated as environmental/pre-existing; a separate full baseline on main was NOT run (same code, same cause), and a plain-clone run was not done.
- Net: 2217 pass, 6 fail (worktree-layout defect), 3 skipped.

## Unfinished: verify_cmd.rs:577 should resolve .git files (gitdir:) (suggested fix, not ported). PR #7 itself still open.
## Not run: clippy, cargo deny, ruler, surfaces-feature tests, tui-check, package workflow.
