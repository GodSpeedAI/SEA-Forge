---
name: release-management
description: Consult when cutting a release of SEA-Forge, when a release's binaries failed to attach or need to be re-attached to an existing tag, when verifying a downloaded release binary's checksum, when rotating the RELEASE_PLEASE_TOKEN, when activating or verifying the main-branch ruleset, or when asked "how do releases work here."
---

# Release management

This is the runbook for releases of `GodSpeedAI/SEA-Forge`. SEA-Forge is **not**
published to crates.io. Every release ships built binaries and a SHA-256
checksum file as assets of its GitHub Release. Read together with
`docs/explanations-and-references/ci-cd-architecture.md` §5 (the causal chain)
and §6 (publication).

## 1. The causal chain, stated plainly

- An ordinary PR (`feat`, `fix`, `chore`, …) merges into `main` and
  **never** produces a tag. Its only release-relevant effect is that
  Release Please reads its squash-commit title to keep one standing release
  PR up to date.
- The maintainer merges the standing release PR when ready to release.
- That merge is the **only** event that causes Release Please to cut a tag
  (`vX.Y.Z`) and create the GitHub Release, in that order, in the same
  `release-please.yml` run.
- The `release-binaries` job in the same workflow then fires (conditioned on
  `release_created == 'true'`), checks out the tag, runs `just release-check`,
  builds the shipped binaries in release mode, and uploads a tarball plus
  `SHA256SUMS` to that GitHub Release.
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
- [ ] No open binary-attach recovery is in progress for the same version.
- [ ] Every workspace crate still has `publish = false` in its `[package]`
      (`cargo metadata --no-deps --format-version 1 | jq '.packages[]|.publish'`
      must print only `[]`). This guards against an accidental crates.io upload.

## 3. Version synchronization decision rule

**Default: synchronized.** Every workspace crate shares `version.workspace = true` in
`Cargo.toml`. One Release Please bump moves all of them. There is no per-crate
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
   - The `release-binaries` job should fire, check out the tag, pass
     `just release-check`, build `sea-forge` and `sea-forge-server` with
     `--release --locked`, and run `gh release upload` for the tarball and
     `SHA256SUMS`.
5. Confirm the GitHub Release at `vX.Y.Z` lists these assets:
   - `sea-forge-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
   - `SHA256SUMS`
6. Verify the checksum (§6) on a downloaded copy.

That's the whole procedure. There is no separate manual `gh release create`,
`gh release upload`, or `git tag` step in the normal path.

## 5. Re-attaching binaries to an existing tag (recovery)

Use this when the `release-binaries` job failed, was skipped, or a release
was cut before its binaries were attached (for example `v0.2.0`). Re-running
is safe: `gh release upload --clobber` replaces assets with the same name.

1. From the Actions UI, or from a terminal:

   ```sh
   gh workflow run release-please.yml --repo GodSpeedAI/SEA-Forge --ref main -f tag=v0.2.0
   gh run watch --repo GodSpeedAI/SEA-Forge   # optional: follow the run
   ```

2. The run checks out `v0.2.0`, runs `release-check` against that tag, builds
   the binaries, and re-uploads them to the existing release. The `tag` input
   must name a tag whose GitHub Release already exists; `gh release upload`
   fails otherwise.
3. The `release-please` job also runs on a dispatch. On a dispatch it normally
   only refreshes the standing release PR and does not cut a release, so it is
   harmless; if it reports `release_created=true`, a new release was cut and
   must be checked before merging anything else.
4. **Never** re-tag or re-create a release for the same version to fix a
   missing asset. Re-attach instead.

## 6. Release artifacts and checksum verification

| Asset | Contents |
| --- | --- |
| `sea-forge_X.Y.Z_amd64.deb` | Debian package: `/usr/bin/sea-forge`, `/usr/bin/sea-forge-server`, `/usr/share/doc/sea-forge/copyright`. `Depends` is computed from the binaries with `dpkg-shlibdeps` |
| `sea-forge-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz` | `sea-forge` (CLI), `sea-forge-server` (daemon), `LICENSE`, under a top-level `sea-forge-X.Y.Z-x86_64-unknown-linux-gnu/` directory |
| `sea-forge-X.Y.Z-macos-universal.pkg` / `.dmg` | Installs `sea-forge` and `sea-forge-server` to `/usr/local/bin`. Universal (arm64 + x86_64); the name is `-macos-arm64` if the x86_64 cross-build failed. Unsigned |
| `sea-forge-X.Y.Z-x86_64-pc-windows-msvc-setup.exe` | Per-user Inno Setup installer to `%LOCALAPPDATA%\Programs\SEA-Forge`; appends it to the user PATH and removes it on uninstall. Unsigned |
| `sea-forge-X.Y.Z-x86_64-pc-windows-msvc.zip` | `sea-forge.exe`, `sea-forge-server.exe`, `LICENSE` |
| `SHA256SUMS` | SHA-256 lines for every asset above |

Packages are built and install-tested by `.github/workflows/package.yml`, called
from the `release-binaries` job. Each package is installed (or mounted) on the
runner that built it before the release is published. Nothing is signed: there
is no Apple Developer ID and no Authenticode certificate. The Windows functional
smoke (cell + daemon) is not yet supported; its install test covers install,
PATH, `--version` and uninstall only.

To verify a download:

```sh
gh release download vX.Y.Z --repo GodSpeedAI/SEA-Forge -p 'sea-forge*' -p SHA256SUMS
sha256sum -c SHA256SUMS            # macOS: shasum -a 256 -c SHA256SUMS
# expected: every listed asset: OK
```

The checksum file is produced in the same workflow that uploads it, so on its own
it protects against a corrupted or truncated download, not against a compromised
release. Every asset also carries a signed build-provenance attestation
(Sigstore, keyless; step "Attest build provenance" in `publish-release`) when the
run builds the commit it is for, which is the case for every release Release
Please creates. It binds each asset's digest to this workflow and to the commit
it was built from. Verify before installing:

```sh
gh attestation verify sea-forge_X.Y.Z_amd64.deb \
  --repo GodSpeedAI/SEA-Forge \
  --signer-workflow GodSpeedAI/SEA-Forge/.github/workflows/release-please.yml \
  --source-digest "$(git rev-parse 'vX.Y.Z^{commit}')"
```

Replace the asset name for any other file. An asset replaced on the release page
by anything other than that workflow, or built from a different commit than the
tag, fails this check. A re-attach
(§5) dispatched from `main` for an older tag builds the tag's commit while the
run is for `main`'s head; the job then attaches the tarball without an
attestation and logs a warning rather than attest the wrong source. Releases
attached before the attestation step existed (v0.2.0) have none; the next
release is the first attested one.

## 7. Token rotation

The repository has exactly one secret that needs rotation: `RELEASE_PLEASE_TOKEN`.

### `RELEASE_PLEASE_TOKEN` (fine-grained PAT)

- **Why it exists:** the default `GITHUB_TOKEN` cannot trigger CI on the bot's
  own release PR. Without this PAT, the release PR would never receive the
  required `CI / gate` check (F-009).
- **When it is absent:** `release-please.yml` falls back to the job's
  `GITHUB_TOKEN` so release-please still runs (it failed with "Input required
  and not supplied: token" from 2026-10-13 while the secret did not exist). For a
  release PR opened or updated by `GITHUB_TOKEN`, GitHub creates the
  `pull_request` runs in an approval-required state: a maintainer approves them
  in the Actions tab so the required checks execute. Never bypass the ruleset
  instead.
- **Scope:** fine-grained, this repository only, `contents: write` and
  `pull-requests: write`. No `workflows`, no admin, no org scope.
- **Expiration:** set the shortest practical expiration when creating it
  (90 days is reasonable). GitHub will warn before expiry.
- **Rotation:**
  1. Visit `https://github.com/settings/personal-access-tokens`.
  2. Generate a new fine-grained token with the same scope on this repo.
  3. `gh secret set RELEASE_PLEASE_TOKEN --repo GodSpeedAI/SEA-Forge` and paste.
  4. Revoke the old token.
  5. Trigger a release-please run (`workflow_dispatch` on
     `.github/workflows/release-please.yml`) to confirm the new token works.
- **Never** print the token in a workflow, log, or commit message.

The release binaries job uses only the run's `GITHUB_TOKEN` (`contents: write`).
There is no registry token to store or rotate.

## 8. Dry-running a release

1. On a feature branch, run `just release-check` and `just ci` locally.
2. To rehearse the binary build, run the same command the job runs:
   `cargo build --release --locked -p sea-forge-cli -p sea-forge-server`, then
   check that `target/release/sea-forge` and `target/release/sea-forge-server`
   exist. Package and checksum steps are in the job's "Package tarball and
   SHA256SUMS" step.
3. Trigger `.github/workflows/release-please.yml` via `workflow_dispatch`
   against `main` without a `tag` input. The release-please job will open (or
   update) the standing release PR; this is recoverable — close the PR if it
   is unwanted.
4. **Do not merge the release PR** until you are ready to release. Merging is
   the step that cuts the tag and the GitHub Release.

If you need a prerelease channel (alpha/beta), use Conventional Commits'
`-alpha` / `-beta` suffixes in the version. Release Please will bump to a
prerelease version. Configure this by setting `prerelease: true` for the relevant
release in `.github/release-please-config.json` only for the duration of the
prerelease cycle. GitHub Releases marked as prerelease carry the same assets.

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
| The release PR's `CI / gate` never starts | `RELEASE_PLEASE_TOKEN` is absent or expired, so the workflow fell back to `GITHUB_TOKEN` (F-009) | approve the release PR's pending workflow runs in the Actions tab; create or rotate `RELEASE_PLEASE_TOKEN` (§7) to restore automatic CI |
| `pr-title.yml` fails on the release PR's title | someone manually edited the release PR title away from Release Please's `chore(main): release vX.Y.Z` | do not bypass; investigate why the title was edited |
| `release-binaries` fails "release not found" on a dispatch | the `tag` input names a tag with no GitHub Release | create the release through Release Please, or fix the tag name; do not create a release by hand |
| `release-binaries` fails `release-check` with "workspace version 0.2.0 != tag v0.3.0" | the tag does not match the manifest; an out-of-band version bump was made | never bump `Cargo.toml` manually; let Release Please do it via the release PR |
| `sha256sum -c SHA256SUMS` reports FAILED on a download | truncated or corrupted download, or assets re-uploaded after the file was downloaded | re-download both files from the same release; if it still fails, re-run §5 and compare |
| `just status-check` fails with "project files changed without corresponding CURRENT_STATUS.yaml and CURRENT_STATUS.md updates" | a tracked project file changed without a status record | add the next revision to `.agents/CURRENT_STATUS.yaml` and rewrite the top of `.agents/CURRENT_STATUS.md`, then rerun |
| Pre-commit hook fails after pulling changes that touched `Cargo.lock` | toolchain or deps drifted | run `just sync`; the post-checkout hook also prints this hint |
| `cargo metadata` shows a workspace crate with `publish` not equal to `[]` | a new crate was added without `publish = false` | add `publish = false` to its `[package]` table |

## 11. Emergency rollback

CI/CD changes can be rolled back without losing evidence:

1. Revert the offending commit on `main` via a normal PR (use
   `revert(scope): ...` as the PR title — Release Please handles `revert`).
2. If a release shipped a real defect, remove or replace its binaries and say
   so in the next release's changelog. Do not delete the tag and re-tag the
   same version. Remove one asset, for example:
   ```sh
   gh release delete-asset vX.Y.Z sea-forge-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz --repo GodSpeedAI/SEA-Forge --yes
   ```
   Then cut `vX.(Y+1).0` through the normal release PR, or re-attach corrected
   binaries to the same tag only when the content is unchanged (§5).
3. To disable the CI/CD automation entirely in an emergency: in the GitHub
   UI, disable `.github/workflows/release-please.yml`. This stops the
   release PR and binary jobs without touching enforcement of `CI / gate`
   on ordinary PRs. Re-enable by clicking "Enable" on the same workflow.

Never delete a workflow file to disable it; disable via the UI so history is
preserved.
