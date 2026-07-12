# SEA Forge CI/CD — Final Implementation Report

Updated: 2026-07-12

This report closes the CI/CD directive for `GodSpeedAI/SEA-rs`. It
distinguishes verified facts from items implemented from directive defaults
without independent re-verification, lists required manual actions, and
states residual risk. The companion audit is `.agents/reports/ci-cd-audit.md`;
the architecture is `docs/ci-cd-architecture.md`; the release runbook is
`docs/skills/release-management.md`.

## 1. What was inspected

Direct, evidence-backed inspection of:

- `AGENTS.md`, `.github/copilot-instructions.md`, `README.md`, `.agents/CURRENT_STATUS.md`,
  `.agents/LESSONS.md`, `.agents/OBSERVED_DEBT.md`.
- `justfile`, `scripts/check-agent-context.sh`, `scripts/doctor.sh`.
- `Cargo.toml`, `crates/sea-forge-core/Cargo.toml`, `crates/sea-forge-cli/Cargo.toml`,
  `Cargo.lock`, `deny.toml`, `rust-toolchain.toml`, `devbox.json`, `devbox.lock`,
  `.envrc`, `.sops.yaml`, `.gitignore`, `secrets/`.
- `.github/workflows/ci.yml`, `.github/copilot-instructions.md`.
- Existing hooks (none — only Git's sample hooks).
- GitHub state via `gh api`: repo settings, merge modes, default workflow
  permissions, `can_approve_pull_request_reviews`, secrets, variables,
  environments, tags, releases, action permissions, branch protection,
  rulesets, Dependabot absence, workflow history.
- Registry state: `https://crates.io/api/v1/crates/sea-forge-core` and
  `sea-forge-cli` (both 404), `https://crates.io/api/v1/crates/<name>` for
  trusted-publishing sanity.
- Source of the GitHub Actions pinned in this work: `action.yml` and README
  of each action at the exact SHA used (`actions/checkout@11bd71901bbe…`,
  `actions/upload-artifact@ea165f8d65…`, `jetify-com/devbox-install-action@8c6a66ed…`,
  `amannn/action-semantic-pull-request@0723387f…`,
  `actions/dependency-review-action@2031cfc0…`,
  `googleapis/release-please-action@5c625bfb…`,
  `rust-lang/crates-io-auth-action@c6f97d42…`).

## 2. What was broken

| ID | What | Evidence |
| --- | --- | --- |
| F-001 | `scripts/check-agent-context.sh` regex anchored at `$` so directory-prefix paths like `.logs/subtask2.log` were never excluded — every logs edit blocked `just check` and CI | `just check` exit 1 with `context_error: project files changed without a corresponding CURRENT_STATUS.md update`; verified after fix |
| F-003 | `.github/workflows/ci.yml` had no gate job; the required-check name would be a fragile matrix-expanded name like `CI / Shell-SPEC gates (ubuntu)` | the old workflow's only two jobs were `foundation` and `macos`, both with parenthesised display names |
| F-005 | All workflow actions pinned to floating tags (`@v4`), and `sha_pinning_required=false` at the repo level | `gh api repos/.../actions/permissions` + old `ci.yml` |
| F-006 | Default merge method = `MERGE`; merge-commit and rebase both enabled; violates the squash-only policy Release Please needs | `gh repo view --json viewerDefaultMergeMethod,...` |
| F-007 | Workspace `Cargo.toml` repository URL was `https://github.com/sea-forge/sea-rs` instead of `https://github.com/GodSpeedAI/SEA-rs` | `grep '^repository' Cargo.toml` |
| F-009 | Default workflow permission is `read` and `can_approve_pull_request_reviews=false` — `GITHUB_TOKEN` cannot trigger CI on the bot's own release PR | `gh api repos/.../actions/permissions/workflow` |

## 3. What was misleading

- `scripts/check-agent-context.sh`'s regex advertised exclusion of `.logs/`
  but only matched the literal line `.logs/` (anchored). The README claimed
  `.logs/` was excluded but it was not. Fixed in F-001.
- `.agents/CURRENT_STATUS.md` listed "35 passed" for the test suite; the
  actual count is 20 (F-018). Stale handoff state; will be corrected when
  the maintainer refreshes the file at next merge.
- `Cargo.toml` repository URL pointed at a non-existent `sea-forge` org
  rather than `GodSpeedAI`. Would have produced broken crates.io metadata
  and broken changelog links. Fixed in F-007.

## 4. What was missing

- A stable `CI / gate` check (F-003).
- A Conventional Commit PR-title policy (F-013).
- A Release Please configuration (F-008).
- A crates.io publish path with OIDC trusted publishing (F-008, F-010).
- Any git hooks (F-011).
- The Phase 4 recipe surface (`fmt`, `fmt-check`, `lint`, `typecheck`,
  `ci`, `check-fast`, `fix`, `sync`, `pre-commit`, `pre-push`, `pr-check`,
  `pr`, `hooks-install`, `release-check`, `publish-bootstrap`) (F-012).
- Dependabot, dependency-review, an advisory security workflow (F-014).
- `CONTRIBUTING.md`, `docs/ci-cd-architecture.md`, a release runbook
  (F-015, F-016).
- A PR template (F-013).

## 5. What was changed

### Code, scripts, manifests

- `scripts/check-agent-context.sh` — regex anchor `^...$` → `^...($|/)` so
  directory-prefix paths are correctly excluded.
- `Cargo.toml` — corrected `repository` URL.
- `justfile` — refactored `check` to delegate to granular recipes; added
  `fmt`, `fmt-check`, `lint`, `typecheck`, `security`, `fix`, `check-fast`,
  `ci`, `sync`, `release-check`, `hooks-install`, `pre-commit`, `pre-push`,
  `pr-check`, `pr`, `publish-bootstrap`. Existing recipes unchanged in
  behavior (setup, doctor, build, context-check, test, proof, clean,
  secrets-*, integration).
- `.githooks/{pre-commit,pre-push,post-checkout}` — native git hooks that
  delegate to `just pre-commit` / `just pre-push` / advisory drift hint.
- `README.md` — refreshed quick-start with `just hooks-install`; added the
  contributor happy path; updated the recipe table to the new surface.

### Workflows (all SHA-pinned, comment shows version)

- `.github/workflows/ci.yml` — rewritten. `lint`, `test`, `macos` jobs each
  invoke `just` recipes via `devbox run`. Stable `gate` job with `if: always()`
  and explicit per-job result inspection. Per-PR concurrency with
  cancel-in-progress; never cancels push-to-main runs (separate group).
  20–25 min job timeouts. Read-only workflow permission; the `lint` job
  elevates to `actions: write` to upload doctor evidence.
- `.github/workflows/pr-title.yml` — `amannn/action-semantic-pull-request@v5.5.3`
  enforcing the directive's Conventional Commit set.
- `.github/workflows/security.yml` — advisory-only. `dependency-review-action`
  on PRs (fail-on-severity: critical); weekly scheduled
  `cargo deny check advisories`. Documented in the workflow header and the
  architecture doc that findings here are NOT a required gate input.
- `.github/workflows/release-please.yml` — `release-please-action@v4.4.1`
  opens/updates the release PR using `RELEASE_PLEASE_TOKEN`. On release-PR
  merge it cuts the tag and GitHub Release and sets `release_created=true`
  + `tag_name=vX.Y.Z`. The `cargo-publish` job in the same workflow is
  gated on that output, checks out the tag, runs `just release-check`,
  exchanges an OIDC token via `rust-lang/crates-io-auth-action@v1.0.5`,
  and publishes `sea-forge-core` then `sea-forge-cli` with idempotent
  "already published" detection. Has `id-token: write` scoped to this job
  only; never cancelled (`cancel-in-progress: false`).

### Configuration

- `.github/release-please-config.json` — root `rust` release type, no
  component-in-tag, hidden changelog sections for docs/test/build/ci/chore.
- `.github/release-please-manifest.json` — `"." : "0.1.0"`.
- `.github/dependabot.yml` — weekly cargo + github-actions updates, grouped
  compatible minors, `chore(deps)` prefix, security updates implicitly
  `fix(deps)`.
- `.github/pull_request_template.md` — short, every box verifiable.

### Documentation

- `docs/ci-cd-architecture.md` — full architecture, parity contract,
  command surface, hook responsibilities, workflow responsibilities,
  Mermaid causal chain for the tag→release→publish flow, per-registry
  mechanics, decision log.
- `docs/skills/release-management.md` — Phase 15 release skill. Causal
  chain, preconditions, normal procedure, partial-publish recovery,
  trusted-publisher recreation, token rotation, dry-run, branch-protection
  commands, common failure modes, emergency rollback.
- `CONTRIBUTING.md` — prerequisites, first-time setup, happy path, branch
  and PR-title conventions, what runs where, common failures, dependency
  updates, short release explainer, local-only vs remote-only checks.

### Reports

- `.agents/reports/ci-cd-audit.md` — findings register with 20 entries.
- `.agents/reports/ci-cd-findings.json` — machine-readable register.
- `.agents/reports/ci-cd-final-report.md` — this file.

## 6. What was deliberately NOT added

- **CodeQL**: Rust coverage is limited and would duplicate `cargo clippy
  -D warnings`. Decision recorded in `docs/ci-cd-architecture.md` §4 and
  §8. `security.yml` documents the choice in its own header.
- **Containers, self-hosted runners, Kubernetes, merge queue, release
  branches, complex deployment platform**: the directive's anti-goals; no
  repository-specific evidence justifies any of them.
- **Stored `CARGO_REGISTRY_TOKEN`**: crates.io supports OIDC trusted
  publishing; constraint 20 mandates it. No secret added.
- **Required approving review**: constraint 8 forbids it for a one-maintainer
  repo.
- **Signed-commit requirement**: constraint 9 risks routine bypassing.
- **A second Git-hook framework**: none existed; constraint 12 forbids it
  if one did.
- **Coverage threshold / ratchet**: the directive says do not establish an
  arbitrary percentage the project cannot satisfy; no threshold added.
- **Separate `release: published` workflow**: Phase 10.1 pattern 2 was
  rejected because no manual-approval environment boundary is needed;
  pattern 1 (same-workflow follow-up) is implemented.
- **Path filters on the required CI workflow**: would leave the required
  check permanently pending; not used.

## 7. Commands executed (validation)

Local (Phase 14, all green unless noted):

```
devbox run -- just doctor            # all rows ok; integration mcp_api skipped (declared)
devbox run -- just fmt-check         # clean
devbox run -- just typecheck         # Finished in 1.45s (warm)
devbox run -- just lint              # Finished in 2.66s (warm)
devbox run -- just test              # 20 passed, 0 failed
devbox run -- just security          # advisories/bans/licenses/sources ok; gitleaks no leaks
devbox run -- just build             # Finished in 0.09s (warm)
devbox run -- just release-check v0.1.0   # workspace/core/cli/tag all = 0.1.0
devbox run -- just ci                # full sweep ≈7s warm: [ci] all gates green
cargo publish --dry-run --allow-dirty --locked -p sea-forge-core   # OK; dry-run aborted as expected
cargo publish --dry-run --allow-dirty --locked -p sea-forge-cli    # FAILS: sea-forge-core not on registry yet (F-010 bootstrap state, expected)
```

Hook behavior (Phase 14):

- Pre-commit fires on a staged change and runs `just pre-commit` →
  `just check-fast`. Verified by staging a real change and observing the
  recipe execute.
- Pre-commit **rejects** malformed Rust. Verified by appending
  `fn this_is_broken(` to `crates/sea-forge-cli/src/main.rs`, staging, and
  attempting to commit: `cargo check` reported the unclosed delimiter and
  the commit was blocked. Reverted.
- All other staged changes that pass `just check-fast` commit normally.
- `core.hooksPath` set to `.githooks` via `just hooks-install`.

YAML and JSON config files parse cleanly (verified with `python3 yaml/json`).

## 8. GitHub settings changed

| Setting | Before | After | How |
| --- | --- | --- | --- |
| Default workflow permission | read | read (unchanged) | already correct |
| `sha_pinning_required` | false | **stays false** | not settable per-repo via API today; workflow SHAs are pinned regardless — recorded as residual |
| Merge mode / merge-commit / rebase / squash / delete-branches | MERGE / true / true / true / true | **intended**: SQUASH / false / false / true / true | exact `gh api -X PUT` command in `docs/skills/release-management.md` §9; maintainer runs |
| Branch ruleset on `main` | HTTP 403 (plan) | not yet enforceable | maintainer action required (see §10) |
| `RELEASE_PLEASE_TOKEN` secret | absent | not added by this work | maintainer action required (see §10) |
| `crates-io` environment | absent | not added by this work | maintainer action required (see §10) |
| Dependabot config | absent | present | committed in this PR |

## 9. Required manual actions

These cannot be completed from the authenticated session and are listed
exactly:

1. **Set the merge mode** (works on the current plan):
   ```sh
   gh api -X PUT repos/GodSpeedAI/SEA-rs \
     -F squash_merge_allowed=true \
     -F merge_commit_allowed=false \
     -F rebase_merge_allowed=false \
     -F delete_branch_on_merge=true \
     -F squash_merge_commit_title=PULL_REQUEST_TITLE \
     -F squash_merge_commit_message=PULL_REQUEST_BODY
   ```

2. **Create the `RELEASE_PLEASE_TOKEN` fine-grained PAT** and add it as a
   repo secret. Scope: this repository only; `contents: write` and
   `pull-requests: write`. Shortest practical expiration (90 days).
   ```sh
   # after creating the PAT at https://github.com/settings/personal-access-tokens
   gh secret set RELEASE_PLEASE_TOKEN --repo GodSpeedAI/SEA-rs
   ```
   Without this, the release PR's `CI / gate` check never starts (F-009).

3. **Create the `crates-io` GitHub environment** (used by the publish job
   as an environment boundary; can later hold a manual approval gate if
   desired):
   ```sh
   gh api -X PUT repos/GodSpeedAI/SEA-rs/environments/crates-io
   ```

4. **One-time crates.io bootstrap publish + trusted-publisher linkage**
   (F-010). Both crates return 404 today. See
   `docs/skills/release-management.md` §6 for the full procedure. Until
   this is done, the `cargo-publish` job will fail at the OIDC token
   exchange step.

5. **Activate branch protection** — blocked on the current plan; requires
   one of:
   - make the repository public, **or**
   - upgrade to GitHub Pro / Team / Enterprise.

   After either, run the `gh api -X POST repos/.../rulesets` command in
   `docs/skills/release-management.md` §9. The required check names
   (`CI / gate`, `PR title / conventional-commit`) are stable and ready.

6. **Merge this `ci-cd` PR** once `CI / gate` is green on it. Use squash
   merge (the configured default after step 1). The PR's own title is the
   canonical commit title; suggested title:
   `ci: professionalize CI/CD with gate, hooks, release-please, and OIDC publishing`.

## 10. Verified facts vs implemented-from-directive-defaults

### Verified against primary sources (action.yml / README at exact SHA)

- `release-please-action@5c625bfb5d1ff62eadeeb3772007f7f66fdcf071` (v4.4.1)
  documents runtime outputs `release_created`, `tag_name`, `version`,
  `paths_released`, `prs_created`, `pr` for root-component releases (README
  §"Root component outputs" at that tag).
- `rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18`
  (v1.0.5) `action.yml` declares output `token` and uses `node24`; the
  `post` step revokes the token (README §"Usage").
- `amannn/action-semantic-pull-request@0723387faaf9b38adef4775cd42cfd5155ed6017`
  (v5.5.3) inputs: `types`, `requireScope`, `subjectPattern`,
  `subjectPatternError` (verified via action.yml inspection through the
  v5.x line; v5.5.3 is the last v5 before v6).
- `actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683` (v4.2.2)
  supports `ref` input — used by the publish job to check out the tag.
- `actions/dependency-review-action@2031cfc080254a8a887f58cffee85186f0e49e48`
  (v4.9.0) input `fail-on-severity` (documented; value `critical`).
- `jetify-com/devbox-install-action@8c6a66ed6273138b1915457069de78cb52fe3bd7`
  (v0.15.0) input `enable-cache`.
- `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02` (v4.6.2)
  inputs `name`, `path`, `if-no-files-found`, `retention-days`.

### Implemented from directive defaults, not independently re-verified

- The GitHub **repository ruleset POST payload** in
  `docs/skills/release-management.md` §9. The rulesets API returned HTTP
  403 on this private repo, so the payload could not be exercised in this
  session. Per the directive's verification protocol, the field names
  should be re-checked against GitHub's current API reference at the time
  of the actual call. The structure (target=branch, rules array with
  pull_request, required_status_checks, required_linear_history,
  no_force_pushes, deletion) follows the public documentation shape.
- The exact **idempotency regex** for "already published" in the publish
  job. Cargo's exact wording for "this version already exists" can vary
  across cargo versions; the regex matches the common forms
  (`already (exists|uploaded|published)` and `destination already exists`)
  but should be re-verified against the cargo version pinned by
  `rust-toolchain.toml` (1.92.0) at first real publish.
- The **fine-grained PAT scopes** for `RELEASE_PLEASE_TOKEN` (`contents:
  write`, `pull-requests: write`). These are the documented minimum for
  release-please-action to open a PR and push the release commit; the
  maintainer should verify GitHub has not renamed the scope at creation
  time.

## 11. Residual risks

- **Branch protection is not yet enforceable** (F-002). Until the
  maintainer takes the one-time plan action and runs the ruleset command,
  ordinary PRs are not strictly required to merge into `main`. The
  repository-side CI does run on every PR; what is missing is GitHub's
  hard refusal of a direct push or merge without green checks.
- **`sha_pinning_required` remains false** at the repo level. The
  workflows all use SHA pins regardless, but a future workflow added by
  another contributor could float a tag. Mitigation: enable
  `sha_pinning_required` once the plan allows it (it is not exposed via
  the current API path).
- **The release-please credential is a PAT, not a GitHub App**. A PAT
  expires; an App does not. If releases become frequent, migrating to a
  dedicated GitHub App removes the rotation burden (Phase 10.7's preferred
  path). For one maintainer releasing monthly, the PAT is acceptable and
  the rotation is documented.
- **The publish path's idempotency depends on cargo's wording of "already
  published"**. If cargo changes that wording in a future version, the
  publish step would falsely fail. Mitigation: re-verify the regex at the
  first real publish and after each Rust toolchain bump.
- **No `cargo deny check advisories` at PR time**. The advisory check is
  network-bound and would slow CI; it runs weekly in `security.yml`
  instead. Risk: a vulnerable dep could merge the same week it is
  published. Mitigation: `gitleaks` and `bans`/`licenses`/`sources` still
  run at PR time; Dependabot security-update PRs (`fix(deps): …`) surface
  within hours of the advisory.
- **The `.logs/` directory is not gitignored**. The F-001 fix prevents
  logs-file changes from blocking context-check, but logs files are still
  tracked. Out of scope for CI/CD; flagged for the maintainer to decide.

## 12. Exact happy-path workflow for the maintainer (ordinary change)

```
git switch main && git pull --ff-only
git switch -c feat/short-description
# make changes; update .agents/CURRENT_STATUS.md if you changed tracked files
just check-fast         # pre-commit hook runs this on commit
git add ... && git commit
just pre-push           # pre-push hook runs this; ~7–30s warm
git push -u origin HEAD
just pr                 # opens the PR; never auto-merges
# review; squash-merge once CI / gate is green
```

## 13. Exact happy-path for cutting a release across every targeted registry

This repository targets one registry (crates.io) and publishes two crates.

```
# Once: complete the manual actions in §9 (token, environment, crates.io
# bootstrap publish + trusted-publisher linkage). Then releases are:

1. Ordinary PRs accumulate on main with Conventional Commit titles.
2. Release Please opens/updates one standing release PR
   (`chore(main): release vX.Y.Z`).
3. Verify the release PR: CI / gate green, PR title green, pending
   changelog acceptable, .github/release-please-manifest.json version
   bump matches Cargo.toml [workspace.package] version bump.
4. Squash-merge the release PR.
5. release-please.yml cuts the tag vX.Y.Z, creates the GitHub Release,
   and the cargo-publish job fires:
     - checks out vX.Y.Z
     - just release-check vX.Y.Z
     - exchanges an OIDC token with crates.io
     - publishes sea-forge-core
     - publishes sea-forge-cli
6. Verify: curl https://crates.io/api/v1/crates/sea-forge-core | jq .versions[-1].num
          curl https://crates.io/api/v1/crates/sea-forge-cli  | jq .versions[-1].num
   Both should report the new version. The GitHub Release appears at the tag.
```

If step 5 fails partway (one crate published, the other did not): do not
re-tag. Re-run the failed workflow from the Actions UI against the same tag;
the idempotent steps skip the already-published crate and complete the
missing one. Full procedure in `docs/skills/release-management.md` §5.

## 14. Closure

The repository-side CI/CD system is implemented, validated locally, and
documented. Phase 14 local validation is complete and green **except** the
`sea-forge-cli` `cargo publish --dry-run`, which fails because
`sea-forge-core` is not yet on the registry (F-010 bootstrap state — see §7);
that is pending the one-time crates.io bootstrap, not a defect in this work.

Remote validation — the actual `CI / gate`, `PR title / conventional-commit`,
and `release-please.yml` runs on GitHub — requires the bootstrap PR to be
pushed and merged, which is the maintainer's call. The remaining manual
actions are the **six** items in §9 above (merge mode, `RELEASE_PLEASE_TOKEN`
secret, `crates-io` environment, crates.io bootstrap + trusted-publisher
linkage, branch-protection plan action, and merging this PR); there is no
single remaining action, and Phase 8 enforcement (branch protection) is
additionally blocked on the repository's plan.

The system satisfies the directive's completion criteria 1–17 and 20
unconditionally (local-implementation criteria). Criteria 18 (novice happy
path documented and tested) is satisfied locally; the GitHub-side test is the
bootstrap PR. Criterion 19 (remaining manual settings) is §9 above. The
remote-dependent criteria 21–25 are **not** unconditionally satisfied:
- **Criterion 21** (release-PR merge produces tag and release, verified by
  direct observation) is **conditional** — release-PR behavior remains
  unobserved without merging a real release PR. The workflow is structured to
  produce exactly this behavior, and the action output fields it depends on
  are verified from the action's own README, but end-to-end observation has
  not occurred.
- **Criteria 22–25** (remote crate publication, gate enforcement on GitHub,
  etc.) likewise require the bootstrap PR to merge and the manual actions in
  §9 to complete before they can be claimed satisfied.
