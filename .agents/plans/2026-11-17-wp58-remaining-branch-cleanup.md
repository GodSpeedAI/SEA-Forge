# WP-58 remaining branch cleanup: agent handoff (2026-11-17)

Give this file to an agent as its full brief. It assumes no prior conversation context.

## Context
WP-58 consolidated every GodSpeedAI repository onto a clean `main`. Branches, tags and worktrees
were integrated or deleted, and each disposition was recorded. Four items remain. The audit trail lives in:

- `~/projects/sea-rs/.agents/reports/fde-convergence-2026-11-12/` (uncommitted, about 1.5 MB):
  `STATUS.md` (chronological log), `branch-ledger.md` (every disposition), `briefs/wp58-repo-consolidation-brief.md`
  (the per-repo rules; read its Rules section, they apply here too).

The owner's standing authorization covers commits, PRs, merges, releases and tags. Never bypass
branch protection, rulesets, required CI or reviews. Never force-push `main`. Never use `--no-verify`. Never change
git config. Never expose credentials.

## Shared rules
- Work in new worktrees under `~/projects/_wt/<short>-<topic>`, and leave each primary checkout on a clean `main`.
- Commits are conventional, with a scope. End each commit message with:
  ```
  Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_015ecmYo3dcgLoDShQ3J7c9g
  ```
  End each PR body with:
  ```
  🤖 Generated with [Claude Code](https://claude.com/claude-code)

  https://claude.ai/code/session_015ecmYo3dcgLoDShQ3J7c9g
  ```
  (If you are a different session, use your own session's attribution lines instead.)
- Run `gitleaks git --log-opts="origin/main..HEAD" --redact --no-banner` before every push; zero new findings.
- Squash-merge only when the required checks are green and every CodeRabbit thread is resolved.
- If a hook, ruleset or permission check refuses an action, STOP that action. Do not retry it another way (no splitting,
  rewording or alternate tool). Record it in STATUS.md with a recommendation.
- Host: 8 GB WSL. Run `free -m` first and do one heavy job at a time. Run cargo as `flock -o ~/projects/_wt/cargo.lock cargo … -j2`.
  No Docker (Postgres comes from pg0). No curl or wget.
- Another session may be working in `~/projects/_wt/fde-sea-rs-integration` (branch `fde/integration`) and
  `~/projects/_wt/cwb-t3`. Do not touch those worktrees or branches.

## Task 1: commit the audit trail (SEA-Forge, `~/projects/sea-rs`)
1. Untracked stale output: `workbench/packages/contracts/schema/` and
   `workbench/packages/contracts/generated/PendingApproval.ts`. SEA-Forge #85 (b94e810a) moved the generated SFWP
   schemas to a server-owned `contracts/` home, so these are leftovers from an old generator run.
   - Confirm that the current generator does not write there: `grep -rn "packages/contracts/schema" scripts justfile package.json`.
   - Then remove them with `git clean -n -- <paths>` (preview), followed by `git clean -f -- <paths>`.
   - If something current does still write there, commit them instead and say why.
2. Scan `.agents/reports/fde-convergence-2026-11-12/` with `gitleaks dir <path> --redact --no-banner`. Fix or justify each hit;
   it must contain no credentials or private keys. The modified tracked files `E-…`, `F-…`, `G-…`, `H-…` and `STATUS.md` are part of it.
3. Open one PR, `docs(fde): WP-58 consolidation audit trail and convergence reports`, containing the whole directory. Merge it when green.
   - SEA-Forge's local pre-push `status-check` hook refuses pushes from a dirty tree. Commit everything first. If the hook still
     refuses for a reason unrelated to the change, record it; do not bypass it.
4. Do NOT edit the content of evidence files (`evidence/**`, `decisions/**`). They are frozen records.

## Task 2: releases warranted by the consolidation
The ledger marks these as release-warranted. In each repo, check `git log <last tag>..origin/main` to confirm there are
user-facing changes since the last tag; if there are none, record "no release needed" instead. Choose the version bump from the
conventional commits (feat → minor; fix or chore only → patch; 0.x stays 0.x).

| Repo (local path) | Last tag | Mechanism |
|---|---|---|
| domainforge-vsc-extension (`~/projects/domainforge-vsc-extension`) | v0.1.0 | Pushing a `v*.*.*` tag runs `release.yml`, which publishes to the VS Code Marketplace (`VSCE_PAT`) and Open VSX (`OPEN_VSX_TOKEN`). Bump `package.json` "version" in a PR first and merge it, then tag that merge commit. |
| domainforge-lsp (`~/projects/domainforge-lsp`) | v0.0.2 | A `v*.*.*` tag runs `release.yml`. Bump `Cargo.toml` (and `Cargo.lock`) in a PR, merge it, then tag. |
| SWE_SEED (`~/projects/SWE_SEED`) | none (`Cargo.toml` 0.1.0) | There is no release workflow; `native-candidate.yml` only builds. Do NOT invent a pipeline. Create an annotated tag `v0.1.0` (or the bump the commits justify) on `origin/main` and a GitHub Release with notes from the commits. |
| DomainForge (`~/projects/domainforge`), optional | v0.22.2 | release-please (`release-please.yml`). Merge its open release PR if one exists and its checks are green; otherwise skip and record it. Its PRs that GITHUB_TOKEN created may need `gh api -X POST repos/GodSpeedAI/DomainForge/actions/runs/<id>/approve`. |

For each release, verify that the workflow run succeeded and the release assets or marketplace version exist. Record the tag,
run id and URL in STATUS.md. If a publish step fails on a missing or expired secret, stop and record it; secrets are the owner's.

## Task 3: re-pin the readiness heads and run readiness
- Readiness is run by `~/projects/godspeed-readiness/scripts/readiness.sh`. It syncs every `~/projects/gsr-heads/<id>` to
  origin's default branch, builds DomainForge's CLI with `--features cli`, starts pg0, and writes a GO/NO-GO report.
- The heads are cep, cognate, cognate-workbench, context-kernel, domainforge, gauntlet, horizon, realitytrace, sea-forge and swe-seed.
- Pin from `origin/main`, never from an operator checkout's local branch. Afterwards, for every head:
  `git merge-base --is-ancestor <pin> origin/main` succeeds, and `git rev-list --count <pin>..origin/main` is 0.
- The report refuses evidence from a dirty checkout, so every head must be clean.
- Known gate gotchas:
  - O01 reads the check run on the exact pinned commit, and path filters can skip Cognate's `package.yml`. Dispatch it with
    `gh workflow run package.yml -R GodSpeedAI/Cognate` and wait for it to finish.
  - Some oracles pin exact test counts (e.g. CK02). When a repo has added tests, raise the count and name the new tests.
    Never lower an expectation to make a gate pass.
  - BM-AUTH c1 p95 can overrun right after heavy gates. Rerun it alone before calling it a regression.
  - Run readiness without `TMPDIR` under `$HOME`.
- Several repos changed a lot during WP-58 (CI rewrites, dependency bumps, a new Workbench capability runtime), so expect
  some gates to need oracle updates. Each must be justified by the change, never by convenience.
- Record the verdict (`GO n/n` or `NO-GO` with the failing gates and causes) in STATUS.md.
  - Fix a real product failure with a PR in the owning repo.
  - If a fix needs a product decision, record it with a recommendation.
- Full stack runs need at least 3.5 GB free; run a watchdog and nothing else heavy at the same time.

## Task 4: final sweep
1. For every repo under `~/projects` with a GodSpeedAI remote:
   - `git fetch --prune`, then `git worktree prune`;
   - list the remaining worktrees under `~/projects/_wt`;
   - remove only clean ones whose branch is merged or deleted, leaving the two active worktrees above.
2. Confirm that every remote branch and tag still present is either in the brief's Keep list or recorded as kept in
   `branch-ledger.md`. Anything unrecorded gets a disposition first.
3. Do not touch:
   - `fde/integration` branches (active lanes);
   - `release-please--*` branches;
   - `gh-pages`;
   - `backup/jollimemory-summaries-v3-20261023` (Gauntlet);
   - `backup/od4-2026-11-16/jollimemory/summaries/v3` (RealityTrace).
4. canonical-evaluation-protocol `fde/integration` has 2 commits that are not merged. They wait on the owner accepting the
   interaction-event contract, so leave them.
5. hermes-agent-sea-forge and zeroclaw-sea-forge are deleted on GitHub. The owner decided NOT to recreate them, and their
   bundles are in `~/backups/consolidation-2026-11-17/`. Do nothing more.

## Report (under 300 words)
- the PRs with merge SHAs;
- the releases (tag, run, URL), or why each was skipped;
- the readiness verdict, and each gate change with its justification;
- the worktrees removed;
- every refusal or blocker, with a recommendation.
