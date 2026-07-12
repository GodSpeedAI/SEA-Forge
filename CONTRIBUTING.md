# Contributing to SEA Forge

Updated: 2026-07-12

SEA Forge is a governed capability-execution kernel. The CI/CD system is
designed so one maintainer can ship safely with short feedback loops and
GitHub as the final enforcement boundary. This document is the happy path
and the failure-mode reference; the full architecture lives in
`docs/ci-cd-architecture.md` and the release runbook in
`docs/skills/release-management.md`.

## Prerequisites

Install three host tools: Git, [Devbox](https://www.jetify.com/devbox), and
[direnv](https://direnv.net/). Everything else (Rust 1.92.0, `just`, `sops`,
`age`, `gitleaks`, `cargo-deny`) is pinned by Devbox and `rust-toolchain.toml`.

## First-time setup

```sh
git clone git@github.com:GodSpeedAI/SEA-rs.git sea-rs && cd sea-rs
devbox shell           # enter the pinned environment
direnv allow           # one time; activates SOPS-decrypted secrets if present
just setup             # converge on the pinned toolchain + workspace deps
just hooks-install     # set core.hooksPath to the checked-in .githooks/
just doctor            # machine-readable environment check
just ci                # full canonical verification (≈7–30s warm, longer cold)
```

If `just doctor` reports a `fail` row, the `next_move` field on that row
tells you exactly what to do.

## The happy path (every change)

```text
git switch main
git pull --ff-only
git switch -c feat/short-description

# make changes; update .agents/CURRENT_STATUS.md if you changed tracked
# project files (scripts/check-agent-context.sh enforces this)

just check-fast        # context + fmt + typecheck — what pre-commit runs
git add ...
git commit             # the pre-commit hook runs `just pre-commit`

just pre-push          # `just ci` — what pre-push runs (≈7s warm; longer cold)
git push -u origin HEAD

just pr                # verify + push + open a PR via gh; refuses from main
```

That's the whole loop. `just pr` opens the PR; merging is a manual step.

## Naming a branch

`feat/`, `fix/`, `chore/`, `docs/`, `refactor/`, `ci/`, `build/`,
`test/`, `perf/`, `revert/`, then a short kebab-case description:
`feat/cli-recall-subcommand`, `fix/sandbox-path-escape`, `ci/pin-actions`.

## Titling a pull request

The PR title is the canonical squash-commit title and **must** follow
Conventional Commits so Release Please can derive releases from it:

```
feat(cli): add recall subcommand
fix(sandbox): reject symlink escape from generated zone
perf(ledger): batch hash-chain append
refactor(types): narrow AuthorityAction surface
docs(spec): clarify settlement boundary
test(min): add timeout-kill conformance
build(deny): prune stale license allowlist
ci(hooks): add pre-push verification
chore(deps): bump chrono to 0.4.41
```

Add `!` for a breaking change: `feat(cli)!: drop legacy --intent flag`. The
`PR title / conventional-commit` workflow enforces this; a PR with a bad
title cannot merge until the title is fixed.

Dependabot PRs already use `chore(deps)` (or `fix(deps)` for security
updates) so they pass the title check.

## What runs where

| When | What runs |
| --- | --- |
| `git commit` | `.githooks/pre-commit` → `just pre-commit` → `just check-fast` |
| `git push` | `.githooks/pre-push` → `just pre-push` → `just ci` |
| `git checkout` / `git merge` | `.githooks/post-checkout` prints drift hint if `Cargo.lock` / `rust-toolchain.toml` / `devbox.*` / `.githooks` / `.gitignore` changed |
| pull request opened | `CI / gate` (lint + test + build + macos + stable gate), `PR title / conventional-commit`, `Security / dependency-review` (advisory) |
| push to `main` after a normal PR merge | `release-please.yml` updates the standing release PR; **no tag, no release** |
| push to `main` after the release PR merge | `release-please.yml` cuts the tag, creates the GitHub Release, then publishes both crates to crates.io via OIDC |
| weekly schedule | `Security / cargo-deny advisories` runs the network-bound half of `just security` |

`--no-verify` skips the local hooks. It does not skip GitHub CI; the required
`CI / gate` check still applies to the pull request.

## Common failures and fixes

| Failure | Cause | Fix |
| --- | --- | --- |
| `context_error: project files changed without a corresponding CURRENT_STATUS.md update` | a tracked project file changed but `.agents/CURRENT_STATUS.md` was not touched | edit `.agents/CURRENT_STATUS.md` (bump `Updated:` and add a line under the relevant section), then rerun |
| pre-commit fails on `cargo fmt --check` | rustfmt would modify a file | `just fmt`, re-stage, retry |
| pre-commit fails with a Rust syntax error | an unclosed delimiter or similar | fix the syntax; the error names the file and line |
| pre-push fails on clippy with `-D warnings` | a clippy lint fired | read the lint; `cargo clippy --workspace --all-targets --all-features --fix` for safe auto-fixes, then re-run; do not weaken the lint |
| `cargo deny check` fails on advisories | a workspace dep has a known advisory | `cargo update -p <crate>` to a fixed version; if impossible, document an `ignore` entry in `deny.toml` with the advisory ID and a re-evaluation date |
| `gitleaks detect` finds a secret | a real or apparent secret was committed | rotate the secret immediately; then scrub history if needed; never just add a false-positive allowlist entry |
| `CI / gate` fails on `cargo build --locked` | `Cargo.lock` is stale relative to `Cargo.toml` | run `cargo update -p <crate>` or `cargo generate-lockfile` locally, commit `Cargo.lock`, push |

## Updating dependencies

Two paths:

1. **Dependabot PRs**: weekly, grouped, titled `chore(deps): bump <crate>
   ...`. Review the diff, run `just ci` locally, merge if green. Security
   updates arrive as `fix(deps): ...` and appear under "Bug Fixes" in the
   next changelog.
2. **Manual**: `cargo update -p <crate>` for a patch bump, or edit
   `Cargo.toml` for a minor/major bump, then run `just ci` and open a PR
   titled `chore(deps): bump <crate> to <version>`.

Never `cargo update` without a `Cargo.lock` commit; `cargo build --locked`
will fail in CI otherwise.

## How releases work (short version)

Releases are owned by Release Please. The full runbook is in
`docs/skills/release-management.md`. The one fact most often confused:

> **Merging the release PR is what produces a tag.** Ordinary PR merges
> never produce a tag. The tag is the effect of merging the standing release
> PR, never the trigger for a merge.

To cut a release: open the standing release PR (titled
`chore(main): release vX.Y.Z`), verify CI is green and the pending changelog
is acceptable, and merge it. The tag and the GitHub Release follow
automatically from that one merge. The crates.io publish of both crates also
fires automatically **once** the one-time trusted-publisher bootstrap is
complete for both crates (see `docs/skills/release-management.md` §6 — both
crates are unpublished today). Until that bootstrap is done the crates will
not publish automatically and a separate `just publish-bootstrap` step is
required; do not assume a merged release PR implies a published crate. There
is no separate `cargo publish` or `git tag` step in the normal path once
bootstrap is complete.

## Local-only and remote-only checks

| Check | Where | Why |
| --- | --- | --- |
| `pr-title.yml` (Conventional Commit title) | remote only | the title does not exist until the PR is opened |
| `dependency-review-action` (vulnerability diff) | remote only | depends on GitHub's dependency graph |
| Required `CI / gate` enforcement | remote only | GitHub branch protection |
| Tag + Release + crates.io publish | remote only | triggered by the release-PR merge |
| Everything else | both local and remote, same `just` recipe | parity contract |

## Where to read more

- `.agents/reports/ci-cd-audit.md` — findings register, before-numbers,
  removed debt.
- `docs/ci-cd-architecture.md` — full architecture, decision log, the causal
  chain as a Mermaid diagram.
- `docs/skills/release-management.md` — release runbook, partial-publish
  recovery, trusted-publisher configuration, token rotation, branch
  protection commands.
- `.agents/specs/Shell-SPEC.md` — development shell, toolchain, secrets, CI
  foundations.
- `AGENTS.md` — agent guide; canonical for repo conventions.
