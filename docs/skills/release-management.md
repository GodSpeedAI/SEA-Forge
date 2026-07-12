---
name: release-management
description: Consult when cutting a release of sea-forge-core / sea-forge-cli, when a release failed partway through, when configuring crates.io trusted publishing, when rotating the RELEASE_PLEASE_TOKEN, when activating or verifying the main-branch ruleset, or when asked "how do releases work here."
---

# Release management

This is the runbook for releases of `GodSpeedAI/SEA-rs`. It describes what
Phase 10 of the CI/CD directive actually built. Read together with
`docs/ci-cd-architecture.md` §5 (the causal chain) and §6 (publication).

## 1. The causal chain, stated plainly

- An ordinary PR (`feat`, `fix`, `chore`, …) merges into `main` and
  **never** produces a tag. Its only release-relevant effect is that
  Release Please reads its squash-commit title to keep one standing release
  PR up to date.
- The maintainer merges the standing release PR when ready to release.
- That merge is the **only** event that causes Release Please to cut a tag
  (`vX.Y.Z`) and create the GitHub Release, in that order, in the same
  `release-please.yml` run.
- The `cargo-publish` job in the same workflow then fires (conditioned on
  `release_created == 'true'`), checks out the tag, verifies version
  synchronization, exchanges an OIDC token with crates.io, and publishes
  `sea-forge-core` then `sea-forge-cli`.
- Nothing triggers a merge by way of a tag. A tag is the effect, not the
  trigger.

If any part of the system ever expects a tag to exist before a PR can merge,
that is a defect against this runbook.

## 2. Precondition checklist before merging a release PR

- [ ] `CI / gate` is green on the release PR.
- [ ] `PR title / conventional-commit` is green (the release PR's title is
      Release Please's own conventional-commit shape; if it fails, something
      has overridden the title and must be investigated, not bypassed).
- [ ] The release PR's diff shows the intended `Cargo.toml`
      `[workspace.package] version` bump and the matching
      `.github/release-please-manifest.json` bump.
- [ ] The release PR's body (pending changelog) is acceptable for public
      release. Hide or rewrite entries that should not appear publicly
      before merging — Release Please will publish what is in the body.
- [ ] No open partial-publish recovery is in progress for the same version.
- [ ] (After §6 is complete) crates.io trusted publishing is configured for
      both crates, workflow path `.github/workflows/release-please.yml`,
      job `cargo-publish`, environment `crates-io`.

## 3. Version synchronization decision rule

**Default: synchronized.** Both crates share `version.workspace = true` in
`Cargo.toml`. One Release Please bump moves both. There is no per-crate
version independence in this repository today.

**Exception evidence required:** if a future change needs one component to
version independently (e.g. a developer-only internal tool published
alongside the public library), the change must:

1. Document the reason in `docs/ci-cd-architecture.md` §5.
2. Move Release Please to manifest mode with explicit per-component
   versioning.
3. Update `just release-check` to expect the divergence explicitly (an
   unexplained divergence is a defect, not a stylistic choice).

Without all three, synchronization is the rule.

## 4. Normal release procedure

1. Look at the open release PR (titled `chore(main): release vX.Y.Z`).
2. Verify §2's checklist.
3. Merge the release PR via **squash merge** (the configured default).
4. Watch `.github/workflows/release-please.yml`:
   - The `release-please` job should set `release_created=true` and
     `tag_name=vX.Y.Z`.
   - The `cargo-publish` job should fire, check out the tag, pass
     `just release-check`, exchange the OIDC token, and publish
     `sea-forge-core` then `sea-forge-cli`.
5. Confirm both crates are visible at their new version:
   `curl -sL https://crates.io/api/v1/crates/sea-forge-core | jq .versions[-1].num`
   and the same for `sea-forge-cli`.
6. Confirm the GitHub Release exists at the new tag with the changelog body.

That's the whole procedure. There is no separate manual `cargo publish`,
`git tag`, or `gh release create` step in the normal path.

## 5. Partial-publish recovery

A partial publish happens when, e.g., `sea-forge-core` publishes successfully
but `sea-forge-cli` fails. crates.io does not support rollback, so the
recovery is **to complete the missing publish for the already-existing
tag**, not to re-tag or re-release the same version.

### Recovery procedure

1. Identify which crate did not publish. The failed `cargo-publish` job's
   `::error::` annotation names it.
2. From the Actions UI, re-run the failed workflow against the same tag
   (`vX.Y.Z`). Both publish steps are idempotent: the step that already
   succeeded will detect the "already published" message from cargo and
   skip; the step that failed will publish.
3. If the re-run also fails for the same crate, debug locally:

   ```sh
   git fetch --tags --quiet
   git checkout "vX.Y.Z"
   devbox shell
   just setup
   just release-check "vX.Y.Z"
   # Bootstrap only if the OIDC path itself is broken; otherwise prefer
   # rerunning the workflow.
   ```

4. **Never** cut a new tag or release for the same content. A second
   `vX.Y.Z` cannot exist; the only fix is to complete the publish of the
   already-tagged version, or — if the version genuinely cannot ship — to
   yank it after the fact and cut `vX.(Y+1).0`.

### What "idempotent" means concretely

Each `cargo publish` step captures cargo's combined stdout/stderr. If cargo
returns success, the step passes. If cargo's output matches the pattern
`already (exists|uploaded|published)` or `destination already exists`, the
step treats the run as success (the version is already on the registry) and
prints a `::notice::` annotation. Any other non-zero exit code is a real
failure and the step fails loudly.

## 6. crates.io trusted-publisher configuration

Lives at `https://crates.io/crates/<crate>/settings` for each crate, under
"Trusted Publishing". The configuration that matches this repository:

- Repository: `GodSpeedAI/SEA-rs`
- Workflow filename: `.github/workflows/release-please.yml`
- Environment: `crates-io`
- Job: `cargo-publish` (implied by the workflow filename + environment)

### How to recreate it if lost

1. Visit `https://crates.io/crates/sea-forge-core/settings` (and the same
   for `sea-forge-cli`).
2. Under "Trusted Publishing", add a new publisher with the values above.
3. Run an **intentional real patch release** (e.g. a docs-only Conventional
   Commit `docs: ...`) to confirm the OIDC exchange works end to end. This is
   not a no-op: it bumps the version, cuts a real `vX.Y.Z` tag, creates a
   GitHub Release, and publishes that version to crates.io. Do this only when
   you are ready to ship a real (if minor) version, since the published
   version cannot be unpublished (only yanked). There is no way to validate
   the trusted-publishing OIDC path without a real publish; `cargo publish
   --dry-run` does not exercise it.

### Required manual bootstrap (one-time, not yet done)

Trusted publishing on crates.io requires the crate to already exist. Today
both crates return HTTP 404 from `https://crates.io/api/v1/crates/<name>`.
The bootstrap is:

```sh
# 1. Create a one-time classic API token at:
#    https://crates.io/settings/tokens
#    Scope: publish-new only. Copy the token to your clipboard.

# 2. In a devbox shell at the repo root. Read the token with hidden input so
#    it never enters shell history or the terminal scrollback:
read -rs -p "CARGO_REGISTRY_TOKEN: " CARGO_REGISTRY_TOKEN; echo
export CARGO_REGISTRY_TOKEN
just setup
just release-check        # sanity: versions synchronized, currently 0.1.0
just publish-bootstrap sea-forge-core
just publish-bootstrap sea-forge-cli

# 3. Visit https://crates.io/crates/sea-forge-core/settings and the same
#    for sea-forge-cli:
#    a. Link this GitHub repository.
#    b. Add the Trusted Publishing entry from the table above.

# 4. Revoke the classic token at https://crates.io/settings/tokens.
unset CARGO_REGISTRY_TOKEN
```

After step 3, every subsequent release publishes via OIDC. The
`cargo-publish` job does not need and does not read any stored
`CARGO_REGISTRY_TOKEN` repository secret.

## 7. Token rotation

The repository has exactly one secret that needs rotation: `RELEASE_PLEASE_TOKEN`.

### `RELEASE_PLEASE_TOKEN` (fine-grained PAT)

- **Why it exists:** the default `GITHUB_TOKEN` cannot trigger CI on the bot's
  own release PR. Without this PAT, the release PR would never receive the
  required `CI / gate` check (F-009).
- **Scope:** fine-grained, this repository only, `contents: write` and
  `pull-requests: write`. No `workflows`, no admin, no org scope.
- **Expiration:** set the shortest practical expiration when creating it
  (90 days is reasonable). GitHub will warn before expiry.
- **Rotation:**
  1. Visit `https://github.com/settings/personal-access-tokens`.
  2. Generate a new fine-grained token with the same scope on this repo.
  3. `gh secret set RELEASE_PLEASE_TOKEN --repo GodSpeedAI/SEA-rs` and paste.
  4. Revoke the old token.
  5. Trigger a release-please run (`workflow_dispatch` on
     `.github/workflows/release-please.yml`) to confirm the new token works.
- **Never** print the token in a workflow, log, or commit message.

There is no `CARGO_REGISTRY_TOKEN` to rotate: the registry side is OIDC only.

## 8. Dry-running a release

There is no fully safe way to publish to crates.io "for testing" — once a
version is on the registry it stays there. The safe dry-run sequence is:

1. On a feature branch, run `just release-check` and `just ci` locally.
2. Trigger `.github/workflows/release-please.yml` via `workflow_dispatch`
   against `main`. The release-please job will open (or update) the standing
   release PR; this is recoverable — close the PR if it is unwanted.
3. **Do not merge the release PR** until you are ready to publish. Merging is
   the irreversible step.
4. To verify the publish path itself without a real publish, manually run
   `cargo publish --dry-run --locked -p sea-forge-core` and the same for
   `sea-forge-cli` locally. `--dry-run` validates the package — build,
   manifest, packaging, and dependency resolution — **without uploading** it
   to the registry. It may still contact the registry to verify dependencies;
   pass `--offline` only if all registry data must come from the local cache.

If you need a prerelease channel (alpha/beta), use Conventional Commits'
`-alpha` / `-beta` suffixes in the version. Release Please will bump to a
prerelease version and crates.io will treat it as a prerelease (not the
latest). Configure this by setting `prerelease: true` for the relevant
release in `.github/release-please-config.json` only for the duration of the
prerelease cycle.

## 9. Activating branch protection (after the one-time plan action)

A repository ruleset endpoint call today returns HTTP 403 because the repo is
private on a plan without rulesets. After the maintainer takes one of the
one-time actions in `docs/ci-cd-architecture.md` §7, run:

```sh
# 1. Repository merge settings — these work on the current plan, run them now.
gh api -X PUT repos/GodSpeedAI/SEA-rs \
  -F squash_merge_allowed=true \
  -F merge_commit_allowed=false \
  -F rebase_merge_allowed=false \
  -F delete_branch_on_merge=true \
  -F squash_merge_commit_title=PULL_REQUEST_TITLE \
  -F squash_merge_commit_message=PULL_REQUEST_BODY

# 2. Default workflow permission = read (already set; verify).
gh api -X PUT repos/GodSpeedAI/SEA-rs/actions/permissions \
  -F default_permission=read \
  -F can_approve_pull_request_reviews=false

# 3. AFTER the one-time plan action (public repo or upgraded plan):
#    Create the main-branch ruleset via a JSON payload matching GitHub's
#    ruleset schema. Required check names are stable. Status checks live in
#    their own `required_status_checks` rule (not nested under pull_request),
#    bypass actors are an empty array, and the force-push rule is named
#    `non_fast_forward` per the current API.
gh api -X POST repos/GodSpeedAI/SEA-rs/rulesets --input - <<'EOF'
{
  "name": "Protect main",
  "target": "branch",
  "enforce_admins": false,
  "bypass_actors": [],
  "conditions": {
    "ref_name": {
      "includes": ["refs/heads/main"],
      "excludes": []
    }
  },
  "rules": [
    {
      "type": "pull_request",
      "parameters": {
        "required_approving_review_count": 0,
        "dismiss_stale_reviews_on_push": false,
        "require_code_owner_review": false,
        "require_last_push_approval": false,
        "required_review_thread_resolution": true,
        "automatic_copilot_code_review_enabled": false
      }
    },
    {
      "type": "required_status_checks",
      "parameters": {
        "strict_required_status_checks": true,
        "do_not_enforce_on_create": false,
        "required_status_checks": [
          { "context": "CI / gate" },
          { "context": "PR title / conventional-commit" }
        ]
      }
    },
    { "type": "required_linear_history" },
    { "type": "non_fast_forward" },
    { "type": "deletion" }
  ]
}
EOF

# 4. Verify the resulting ruleset and confirm no bypass actor remains.
#    `gh api .../rulesets` lists all rulesets; capture the id of "Protect main"
#    and re-fetch it to confirm bypass_actors is empty and the required
#    status checks are exactly CI / gate and PR title / conventional-commit.
gh api repos/GodSpeedAI/SEA-rs/rulesets
ruleset_id=$(gh api repos/GodSpeedAI/SEA-rs/rulesets --jq '.[] | select(.name=="Protect main") | .id')
gh api "repos/GodSpeedAI/SEA-rs/rulesets/$ruleset_id" \
  --jq '{name, bypass_actors, rules: [.rules[].type]}'
```

The exact field names of the ruleset PATCH/POST payload should be
re-verified against GitHub's current API reference at the time of the call
(see CI/CD directive's Verification protocol). If a field has been renamed,
prefer the API reference over this snippet.

## 10. Common failure modes and their fixes

Observed or anticipated during Phase 14 validation:

| Symptom | Cause | Fix |
| --- | --- | --- |
| `CI / gate` is permanently pending on a PR | required check name was set to a matrix-expanded name like `CI / verify (ubuntu-latest)` instead of the stable `CI / gate` | update the ruleset to require `CI / gate` only |
| The release PR's `CI / gate` never starts | `release-please.yml` is using the default `GITHUB_TOKEN` instead of `RELEASE_PLEASE_TOKEN` (F-009) | confirm `RELEASE_PLEASE_TOKEN` secret exists and is non-expired; confirm the workflow reads it |
| `pr-title.yml` fails on the release PR's title | someone manually edited the release PR title away from Release Please's `chore(main): release vX.Y.Z` | do not bypass; investigate why the title was edited |
| `cargo-publish` fails "already published" but is treated as failure | the idempotency regex did not match the current cargo output | update the regex in `release-please.yml`; do not re-tag |
| `cargo-publish` fails on `sea-forge-cli` with "no matching package named `sea-forge-core`" | `sea-forge-core` did not publish first | the workflow already orders core-before-cli; check that the core step's idempotency branch did not falsely skip |
| Trusted-publishing token exchange returns 401/403 | trusted publisher not yet configured on crates.io, or workflow path / environment / job name in the publisher config does not match this workflow exactly | complete §6's bootstrap and configuration |
| `just release-check` fails with "workspace version 0.1.0 != tag v0.2.0" | the tag does not match the manifest; an out-of-band version bump was made | never bump `Cargo.toml` manually; let Release Please do it via the release PR |
| `just check` fails with `context_error: project files changed without a corresponding CURRENT_STATUS.md update` | a tracked project file changed without an `## Updated:` / status refresh | edit `.agents/CURRENT_STATUS.md` (bump the `Updated:` line and describe the change), then rerun |
| Pre-commit hook fails after pulling changes that touched `Cargo.lock` | toolchain or deps drifted | run `just sync`; the post-checkout hook also prints this hint |

## 11. Emergency rollback

CI/CD changes can be rolled back without losing evidence:

1. Revert the offending commit on `main` via a normal PR (use
   `revert(scope): ...` as the PR title — Release Please handles `revert`).
2. If a release shipped a real defect, **yank** the affected crate version(s).
   Both crates share `version.workspace = true`, so a defective release almost
   always affects both — yank each one explicitly:
   ```sh
   cargo yank --vers X.Y.Z sea-forge-core
   cargo yank --vers X.Y.Z sea-forge-cli
   ```
   Yanking does not delete the version; existing `Cargo.lock` files continue to
   resolve a yanked version, new ones do not. State this plainly to users in
   the next release's changelog. (If only one crate is defective, yank only
   that one — but then the workspace version has already diverged from the
   registry; the next release re-synchronizes.)
3. To disable the CI/CD automation entirely in an emergency: in the GitHub
   UI, disable `.github/workflows/release-please.yml`. This stops the
   release PR and publish jobs without touching enforcement of `CI / gate`
   on ordinary PRs. Re-enable by clicking "Enable" on the same workflow.

Never delete a workflow file to disable it; disable via the UI so history is
preserved.
