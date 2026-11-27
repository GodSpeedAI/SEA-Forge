# Branch ledger: RealityTrace (sxr), WP-00c

Integration branch: fde/integration @ 62f75aa (= origin/main 62f75aa, pushed). Worktree ~/projects/_wt/fde-sxr-integration.
No merge or port was needed: every local unmerged branch is landed or superseded.

| Branch | Tip | Verdict | Reason |
|---|---|---|---|
| chore/domainforge-0.21.1 | ec1c416 | superseded | main pins DomainForge 0.21.2 (sxr-df/src/lib.rs:12); branch pins 0.21.1 |
| feat/command-verifier | a68a4b8 | landed | command@1 + target digest present on main (sxr-verify/src/verifier.rs, digest.rs, sxr-cli/tests/command_verifier.rs); remaining diff is main being ahead |
| feat/native-binary-distribution | c239d5c | landed | PR #12 squash (4d154b1); non-.agents diff vs main is only main-ahead content (agent-event ingest, release-please, changelogs); .agents has no branch-only content |
| feat/verifier-completed-at | 3fe3a50 | landed | reverse-apply of its diff succeeds on main |
| fix/bundled-sqlite | cf6e91d | landed | rusqlite "bundled" in Cargo.toml:33, no .cargo/config.toml, clean-install.yml on main |
| fix/ledger-read-only-no-create | ae4a7ce | landed | reverse-apply succeeds |
| fde/fix-ingest-json-spelling | 849d664 | landed | PR #14 (90eff85); reverse-apply succeeds |
| fde/fix-release-sha256sums | 354bde5 | landed | PR #15 (62f75aa); reverse-apply succeeds |
| jollimemory/summaries/v3 | 66d9652 | kept out | tooling summary branch, not product code |

## Gate (cargo fmt --all --check; cargo test --workspace --all-targets)
- Baseline (= final, tree identical to origin/main): fmt rc=0; 29 suites, 244 passed, 0 failed, 3 ignored.
- Acceptance: met (no change).

## Conflicts: none. Unfinished work: none ported; recon notes ARM64 glibc 2.35 install check and realitytrace-v* publish path unconfirmed (.agents/current_status.yml, stale on the unmerged branch).
## Not run: cargo clippy -D warnings (CI step), scripts/tests/rate-limit-wait.test.sh, real-binary DomainForge tests, e2e-world-loop hop.
