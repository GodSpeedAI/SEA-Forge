# WP-58 phase B: per-repository consolidation brief

The coordinator names one repository and its specifics in the prompt. Everything else is in this brief.

## Goal
Bring the repository to a clean, tested main. Every useful piece of branch work is integrated, everything else is deleted, and each disposition is recorded. Then open one CI optimization PR.
The owner prefers consolidation over preservation and says to treat all code as our own. Backward compatibility is NOT required: remove any shims and dead code you touch.
Never break evidence integrity (frozen specs, anchored evidence, evaluation records, release qualification records and assets) or security invariants. Never bypass branch protection, rulesets or required CI.

## Inputs
- Inventory: `/home/sprime01/projects/sea-rs/.agents/reports/fde-convergence-2026-11-12/recon/branch-inventory-2026-11-16.md`. Read the rows for your repo.
- Ledger: `.../branch-ledger.md`. Add a section `## <Repo> consolidation (2026-11-16)`. Use the Gauntlet, Horizon and SEA-Forge sections as the model for format and rigor. The OD-4 backup tags `backup/od4-2026-11-16/*` for your repo are listed there.
- The repo's AGENTS.md, CLAUDE.md, justfile and package.json, for its gates. Postgres comes from pg0, never Docker (see memory `no-docker-use-pg0.md`).

## Keep
- main.
- release-please branches and release branches.
- website or landing-page branches.
- Active `fde/*` lanes: those with an open PR or with commits newer than 2026-11-10.
- gsr-heads pinned worktrees.
- Any remote branch that holds the only copy of something the owner may want, with a reason recorded.

## Rules
- Implement in NEW worktrees under `~/projects/_wt/<short>-<topic>`.
- The primary checkout `~/projects/<repo>` is in scope. It ends on main, fast-forwarded and clean.
  - Before discarding any uncommitted change, either verify it equals its pushed backup or commit it to a branch first. To verify: build the tree with a temp `GIT_INDEX_FILE` (`read-tree HEAD`, `add -A`, `write-tree`) and check it equals `<dirty tag>^{tree}`.
  - Integrate anything useful through a PR. Never commit regenerated output over committed evidence.
  - Remove untracked files only with `git clean -f -- <explicit paths>`, after a `-n` preview. Never use `rm` with globs or with cd-relative paths.
- Stashes: review each one, integrate anything useful, then drop it.
- Commits use conventional format with a scope, and end with:
  ```
  Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_015ecmYo3dcgLoDShQ3J7c9g
  ```
  PR bodies end with:
  ```
  🤖 Generated with [Claude Code](https://claude.com/claude-code)

  https://claude.ai/code/session_015ecmYo3dcgLoDShQ3J7c9g
  ```
- Never change git config. Run commit and push sequentially in one command.
- Never force-push main or any branch you did not create.
- Let pre-push hooks run. `--no-verify` is not authorized. If a local hook refuses a tag or branch deletion for a reason unrelated to the change, delete the ref through `gh api -X DELETE repos/GodSpeedAI/<Repo>/git/refs/<heads|tags>/<name>`.
- Before every push, run `gitleaks git --log-opts="origin/main..HEAD" --redact --no-banner`. Zero new findings are required.
- Merging: squash-merge once the required checks are green and the CodeRabbit threads are resolved. Fix real issues first. Do not delete a PR's branch until the PR shows MERGED.
- You may approve held `action_required` runs on bot PRs, for the current head SHA only.
- REFUSALS: if a ruleset, a hook or the permission classifier refuses an action, STOP that action. Do NOT retry it in another form: no splitting into smaller commands, no rewording, no alternate tool. Report it with a recommendation and its tradeoffs.
- Do NOT create release tags or merge release PRs. Record in the ledger whether the merged changes warrant a release.
- Host: 8 GB WSL. Check `free -m` first and run one heavy job at a time. Run cargo as `flock -o ~/projects/_wt/cargo.lock cargo ...`, with `-j2` for builds. CI is authoritative when local runs are memory-limited. No curl, wget or Docker. Keep tool output small.
- Deletion: delete a branch locally or remotely only after the ledger records it as one of:
  - integrated (PR plus merge SHA, or content verified on main);
  - superseded (by what, with SHA);
  - rejected (one-line reason).
  Then remove clean worktrees and run `git fetch --prune` and `git worktree prune`.
- Dependabot: merge when green. If it is superseded or failing, replace it with a fixed PR or close it with a reason.
- Delete the repo's OD-4 backup tags after disposition.

## Steps
1. Audit and disposition every branch, PR, worktree and stash.
2. Integrate the valuable work in as few coherent PRs as sensible, each green before merge.
3. Delete, prune, and bring the primary checkout to a clean main.
4. Open the CI optimization PR.
   1. Measure the slowest required path on main first.
   2. Shard slow test suites: bun with timings, cargo nextest partitions, or pytest-xdist or splits.
   3. Use rust-cache with `save-if` set to main only.
   4. Cache tool, browser and apt installs.
   5. Trigger on pushes to main only, and on PRs with concurrency cancel-in-progress.
   6. Short-circuit docs-only changes inside the jobs. Check `gh api repos/GodSpeedAI/<Repo>/rules/branches/main`, and never path-skip a required check.
   7. Pin actions by SHA.
   8. Leave release, tag and publish workflows functionally unchanged.
   9. Report the slowest required path before, after on a cold cache, and after on a warm cache.
5. Update the ledger and append a WP-58 entry to STATUS.md in the reports directory. Do not commit these.

## Report (under 300 words)
- Disposition counts and notable items.
- Merged PRs with SHAs.
- Rejections with reasons.
- CI before and after numbers.
- Whether a release is warranted.
- What remains.
- Each blocked item or refusal, with a recommendation and its tradeoffs.
