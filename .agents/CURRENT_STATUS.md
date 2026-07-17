# Current Status

Updated: 2026-07-17

## Objective

Make the public SEA Forge repository safely consumable: publish the licensing
package, protect `main`, and restore the required GitHub Actions CI gate.

## Worktree State

The public licensing package is committed and pushed to `main` as `dd7195c`.
This focused repair is on `fix/ci-workflow-parse`, based on that commit, in the
isolated `license-main` worktree. The active `full-spec` worktree and its
uncommitted implementation changes were not modified.

## Changed Files

- `.github/workflows/ci.yml` — removes the invalid literal GitHub expression
  placeholder from a shell comment so Actions can parse and schedule the CI
  workflow.
- `.agents/CURRENT_STATUS.md` — this resumable handoff.

## Completed

- `main` is public at `dd7195c` with the source-available, commercial, and
  enterprise license documents, plus crate packages that include `LICENSE`.
- `Protect main` is active with no bypass actors: pull requests, `CI / gate`,
  conventional PR titles, linear history, and no force-push or deletion.
- GitHub secret scanning and push protection are enabled.
- Root cause of the historical no-job CI failures is confirmed: GitHub parsed
  the literal `${{ ... }}` in the gate shell comment as an invalid expression.

## Verification

- The pre-fix reproduction found the invalid literal at
  `.github/workflows/ci.yml:138`.
- The post-fix workflow no longer contains the invalid literal.
- `devbox run -- just sync`, `devbox run -- just context-check`, and
  `devbox run -- just check` passed.
- `devbox run -- just test` passed (35 tests, 0 failed).
- GitHub Actions verification is pending the CI-fix PR.

## Remaining

1. Run the repository checks, commit the focused workflow repair, and open a
   PR.
2. Confirm `CI / gate` and `PR title / conventional-commit` pass, then merge
   the PR using squash.
3. Add the `RELEASE_PLEASE_TOKEN` repository secret (fine-grained PAT limited
   to this repository with Contents and Pull requests write) so Release Please
   can create its release PR.

## Blockers

- Release Please currently fails because `RELEASE_PLEASE_TOKEN` is not set.
  Do not reuse the broader local GitHub CLI credential as that secret.

## Decisions

- The repair changes only a parsed comment; it does not alter CI jobs, required
  checks, or branch-protection policy.
- The PR is required because `main` protection is now active and has no bypass
  actors.
