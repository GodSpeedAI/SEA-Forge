# SEA Forge CI/CD Architecture

Updated: 2026-07-12

This document is the single source of truth for how local checks, GitHub
Actions, branch protection, releases, and crate publication fit together in
`GodSpeedAI/SEA-rs`. The companion audit is `.agents/reports/ci-cd-audit.md`;
the runbook for cutting a release is `docs/skills/release-management.md`.

## 1. The parity contract

Local development and GitHub Actions share one layer of behavior. A second,
smaller layer of controls is GitHub-only because they depend on GitHub itself.

### Shared parity layer (local == CI)

| Concern | Source of truth |
| --- | --- |
| Pinned toolchain | `rust-toolchain.toml` (Rust 1.92.0, rustfmt + clippy) |
| Pinned system tools | `devbox.json` + `devbox.lock` (git, rustup, just, direnv, sops, age, gitleaks, cargo-deny) |
| Dependencies | `Cargo.lock` (committed); installed via `cargo fetch --locked` |
| Formatting | `cargo fmt --all` (config in `rustfmt` defaults) |
| Lint | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` |
| Type check | `cargo check --workspace --all-targets --locked` |
| Tests | `cargo test --workspace --all-features --locked` |
| Build | `cargo build --workspace --all-targets --locked` |
| Supply chain + secrets | `cargo deny check` + `gitleaks detect` |
| Agent handoff | `scripts/check-agent-context.sh` (run via `just context-check`) |
| Canonical command surface | every check above is a `just` recipe in `justfile` |

The two execution paths converge on the same commands: hooks call `just`
recipes; GitHub Actions call the same recipes via `devbox run -- just ...`.

### GitHub-only enforcement layer (no local equivalent)

| Concern | Where |
| --- | --- |
| Conventional Commit PR titles | `.github/workflows/pr-title.yml` |
| Required status checks (gate) | `.github/workflows/ci.yml` job `gate` |
| Branch ruleset on `main` | repository ruleset (manual action — see §7) |
| Repository token permissions | per-workflow `permissions:` |
| Tag + GitHub Release creation | `.github/workflows/release-please.yml` |
| crates.io OIDC exchange | `cargo-publish` job (id-token: write, scoped) |
| Dependency review on PRs | `.github/workflows/security.yml` |
| Dependabot updates | `.github/dependabot.yml` |

There is no local GitHub Actions emulator and none is needed: the `just`
contract is what local and remote agree on. Anything that can only happen on
GitHub is documented as such.

## 2. The Just command contract

Recipes are grouped in `justfile`. The contract every contributor needs to
remember is small.

| Recipe | When to use it | What it runs |
| --- | --- | --- |
| `just setup` | first checkout, or after `Cargo.lock`/`rust-toolchain.toml` change | `devbox install`, rustup toolchain install, `cargo fetch --locked` |
| `just sync` | after `git pull` that touched dependency files | re-pin toolchain + refetch deps |
| `just doctor` | when something feels off; emits JSONL evidence to `target/bootstrap-evidence/` | `scripts/doctor.sh` |
| `just hooks-install` | first checkout, or after a hook file changes | sets `core.hooksPath=.githooks` |
| `just check-fast` | before each commit (also run by `pre-commit` hook) | context-check + fmt-check + typecheck |
| `just fmt` / `just fmt-check` | apply / verify rustfmt | `cargo fmt --all` |
| `just fix` | apply only safe automatic fixes | `cargo fmt --all` (clippy fixes are reviewed manually) |
| `just lint` | clippy | `cargo clippy ... -D warnings` |
| `just typecheck` | fast type-check | `cargo check --workspace --all-targets --locked` |
| `just security` | supply-chain + secret scan | `cargo deny check` + `gitleaks detect` |
| `just test` | unit + integration tests | `cargo test --workspace --all-features --locked` |
| `just build` | full build | `cargo build --workspace --all-targets --locked` |
| `just check` | developer quality sweep | context-check + fmt-check + lint + typecheck + security |
| `just ci` | canonical CI verification (union of required CI job *commands* on this platform) | check + test + build |
| `just pre-commit` | invoked by `.githooks/pre-commit` | `just check-fast` |
| `just pre-push` | invoked by `.githooks/pre-push` | `just ci` |
| `just pr-check` | alias of `just ci` (what the gate sees) | `just ci` |
| `just pr` | verify + push + open a PR via gh | see recipe body; refuses from `main`, never auto-merges |
| `just release-check [tag]` | verify version synchronization across manifests | reads `workspace.package.version`, `sea-forge-core`, `sea-forge-cli`, optional tag; fails on any mismatch |
| `just publish-bootstrap <crate>` | one-time manual crates.io publish (F-010) | uses `$CARGO_REGISTRY_TOKEN`; refuses without it |
| `just context-check` | agent-handoff validation; runs inside `check-fast` and `ci` | `scripts/check-agent-context.sh` |
| `just proof` | minimum-spec P1–P4b conformance | `spec-minimum.md` §12.2 |
| `just clean` | remove `target/` and bootstrap evidence | `cargo clean` + `rm -rf target/bootstrap-evidence` |

Existing `secrets-*` recipes (`secrets-init`, `secrets-edit`,
`secrets-check`, `secrets-rekey`) and `integration` are unchanged.

> **macOS is a CI-only cross-check.** `just ci` runs the full sweep on the
> *current* OS. The `macos` CI job additionally re-runs `just ci` on a real
> macOS runner; that cross-platform verification cannot be reproduced by a
> local `just ci` on Linux. So `just ci` is the union of the required CI job
> *commands* on this platform, not a guarantee of macOS-green.

## 3. Hook responsibilities

The hook manager is native `core.hooksPath` pointing at the checked-in
`.githooks/` directory. No Husky, Lefthook, or pre-commit framework is in
use. `just hooks-install` performs the one-time setup. Hooks always delegate
to a `just` recipe, so the contract is the recipe, not the hook.

| Hook | Runs | Time budget | Bypass |
| --- | --- | --- | --- |
| `pre-commit` | `just pre-commit` → `just check-fast` (context + fmt + typecheck) | <10s warm | `git commit --no-verify` |
| `pre-push` | `just pre-push` → `just ci` (full sweep) | ~10–30s warm | `git push --no-verify` |
| `post-checkout` | advisory drift hint only; never mutates the tree | <1s | none needed |

Hooks are developer feedback, not enforcement. `--no-verify` skips local
checks; GitHub's required `CI / gate` check still applies to the pull request
regardless. The drift hint in `post-checkout` prints `next move: just sync`
when `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `devbox.json`,
`devbox.lock`, `.githooks`, or `.gitignore` changed since the previous HEAD;
it never runs `sync` for you.

## 4. Workflow responsibilities

| Workflow | Trigger | Gate? | Purpose |
| --- | --- | --- | --- |
| `ci.yml` | `pull_request` to `main`, `push` to `main`, `workflow_dispatch` | **yes** — required check `CI / gate` | lint, test (includes build), macos verify, stable gate |
| `pr-title.yml` | `pull_request` opened/edited/reopened/synchronize on `main` | yes — `PR title / conventional-commit` | Conventional Commit PR title enforcement for Release Please |
| `release-please.yml` | `push` to `main`, `workflow_dispatch` | no (post-merge only) | open/update release PR; on release-PR merge, cut tag + GitHub Release, then publish to crates.io |
| `security.yml` | `pull_request` to `main`, weekly schedule, `workflow_dispatch` | **advisory only** — findings never block the gate | dependency-review on PRs; weekly cargo-deny advisories (offline CI otherwise) |

`security.yml` is explicitly advisory. The blocking supply-chain and
secret-scan checks already run inside `just security` inside `CI / gate`.
Documenting this prevents the workflow from being mistaken for the gate.

### Required check name (stable)

Branch protection (when activated — see §7) requires exactly `CI / gate`. The
`gate` job depends on `[lint, test, macos]` with `if: always()` and
explicitly inspects each `needs.<job>.result`. There is **no standalone
`build` job**: `build` is a step inside the `test` job (see `.github/workflows/ci.yml`),
so a build failure fails `test` and thus fails the gate. Because the workflow
has no path-filter skip logic, any result other than `success` is treated as
failure (an unexplained `skipped` is a failure, not a pass). This is what
makes the gate name stable: matrix expansion, added jobs, or temporary flakes
cannot leave a permanently pending or silently-passing required check.

### Concurrency and cancellation

- `ci.yml` and `pr-title.yml` cancel obsolete in-progress runs for the same PR.
- `release-please.yml` is **never** cancelled. A push to `main` updates the
  standing release PR instead; an in-flight publish is allowed to complete.

### Permissions (least privilege)

- Every workflow defaults to `contents: read`.
- `ci.yml` `lint` job stays at `contents: read`; `actions/upload-artifact` works
  with the default `GITHUB_TOKEN` and needs no `actions: write`.
- `release-please.yml` `release-please` job has `contents: write` and
  `pull-requests: write` (it tags, releases, and opens the release PR). It uses
  the `RELEASE_PLEASE_TOKEN` secret (a fine-grained PAT), not the default
  `GITHUB_TOKEN`, so CI can run on the bot's own release PR (F-009; see
  `docs/skills/release-management.md` §7).
- `release-please.yml` `cargo-publish` job has `id-token: write` only, plus
  `contents: read`. It exchanges an OIDC token via
  `rust-lang/crates-io-auth-action@v1.0.5`; the token is auto-revoked in the
  action's `post` step. No long-lived `CARGO_REGISTRY_TOKEN` is stored as a
  repository secret.

### Action pinning

All actions are pinned to verified full commit SHAs with the human-readable
version in a trailing comment. Verified SHAs (Phase 1):

| Action | SHA | Version |
| --- | --- | --- |
| `actions/checkout` | `11bd71901bbe5b1630ceea73d27597364c9af683` | v4.2.2 |
| `actions/upload-artifact` | `ea165f8d65b6e75b540449e92b4886f43607fa02` | v4.6.2 |
| `jetify-com/devbox-install-action` | `8c6a66ed6273138b1915457069de78cb52fe3bd7` | v0.15.0 |
| `amannn/action-semantic-pull-request` | `0723387faaf9b38adef4775cd42cfd5155ed6017` | v5.5.3 |
| `actions/dependency-review-action` | `2031cfc080254a8a887f58cffee85186f0e49e48` | v4.9.0 |
| `googleapis/release-please-action` | `5c625bfb5d1ff62eadeeb3772007f7f66fdcf071` | v4.4.1 |
| `rust-lang/crates-io-auth-action` | `c6f97d42243bad5fab37ca0427f495c86d5b1a18` | v1.0.5 |

Dependabot watches the `github-actions` ecosystem and will open
`chore(deps)`-prefixed PRs when these SHAs go stale.

## 5. Release flow — the tag/release/publish causal chain

This is the part most often confused. State it exactly:

- Ordinary PRs (`feat`, `fix`, `chore`, …) merge into `main`. **They never
  produce a tag.** Their only release-relevant effect is that Release Please
  reads their squash-commit titles to keep one standing release PR up to date.
- Release Please maintains exactly one open, continuously-updated release PR
  at a time. Its body is the pending changelog; its diff is the pending
  version bump to `[workspace.package] version` in `Cargo.toml` and to
  `.github/release-please-manifest.json`.
- When the maintainer deliberately merges the release PR, the same
  `release-please.yml` workflow runs again on that push, recognizes it as a
  release-PR merge, and **only then** creates the Git tag (`vX.Y.Z`) and the
  GitHub Release, and sets `release_created=true` with `tag_name=vX.Y.Z`.
- The `cargo-publish` job in the **same workflow** is conditioned on
  `needs.release-please.outputs.release_created == 'true'`. It checks out the
  exact resulting tag, re-verifies version synchronization with
  `just release-check`, exchanges an OIDC token with crates.io, and publishes
  `sea-forge-core` then `sea-forge-cli`.
- **Nothing triggers a merge by way of a tag.** A tag is the effect of the
  maintainer merging the standing release PR.

```mermaid
sequenceDiagram
    participant M as Maintainer
    participant R as Repo (main)
    participant RP as release-please.yml
    participant C as crates.io

    Note over M,C: Ordinary change
    M->>R: push feature branch, open PR
    R-->>RP: PR opens; CI / gate runs (no tag, no release)
    M->>R: squash-merge feat/fix PR (Conventional Commit title)
    R->>RP: push to main (release-please job)
    RP->>R: open or UPDATE the standing release PR (no tag, no release)

    Note over M,C: Release
    M->>R: review release PR, then merge it
    R->>RP: push to main (release-please job, recognizes release merge)
    RP->>R: create tag vX.Y.Z + GitHub Release (release_created=true)
    RP->>RP: cargo-publish job fires (same workflow)
    RP->>RP: checkout tag, just release-check
    RP->>C: OIDC token exchange
    RP->>C: cargo publish -p sea-forge-core
    RP->>C: cargo publish -p sea-forge-cli
```

This implements Phase 10.1 pattern 1 (same-workflow follow-up jobs), the
mandatory default. Pattern 2 (separate `release: published` workflow) was
rejected because no manual-approval environment boundary is needed.

### Version synchronization

The repository targets one registry (crates.io) and publishes two crates
(`sea-forge-core` and `sea-forge-cli`) that share
`version.workspace = true`. The single source of truth is
`[workspace.package] version` in the root `Cargo.toml`. Release Please's
`rust` strategy bumps that field; both crates inherit it; nothing else
needs to move. `just release-check [tag]` verifies the workspace version,
both crate versions, and the tag all agree before any publish step touches
the registry, and again at publish time inside the workflow.

There is no multi-registry case in this repository. If a future change adds
a PyPI package or an npm wrapper built from the same source, add a
component-grouping configuration to force synchronized versions, and extend
`just release-check` to read the additional manifest. Independent version
drift across registries is a defect.

## 6. Per-registry publication mechanics

This repository publishes to **crates.io only**.

### crates.io — OIDC trusted publishing

- Workflow: `.github/workflows/release-please.yml`, job `cargo-publish`.
- Permission: `id-token: write` on that job only.
- Action: `rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18`
  (v1.0.5). Exchanges the GitHub OIDC JWT for a short-lived crates.io token.
  The action's `post` step revokes the token when the job ends.
- Bootstrap (one-time, **manual, not yet done** — both crates return HTTP 404
  on `https://crates.io/api/v1/crates/<name>` today):
  1. The maintainer creates a one-time classic API token at
     `https://crates.io/settings/tokens` (scope: `publish-new`).
  2. Runs `CARGO_REGISTRY_TOKEN=<token> just publish-bootstrap sea-forge-core`
     and the same for `sea-forge-cli`.
  3. Visits `https://crates.io/crates/<crate>/settings`, links this GitHub
     repository, and configures trusted publishing for workflow path
     `.github/workflows/release-please.yml`, job `cargo-publish`, environment
     `crates-io`.
  4. Revokes the classic token at `https://crates.io/settings/tokens`.
  After this, every subsequent release publishes via OIDC and no
  long-lived token is stored in the repository.

### Why no stored registry token

Per directive 10.4 / constraint 20: trusted publishing is available on
crates.io and is the default. There is no documented reason to store a
long-lived `CARGO_REGISTRY_TOKEN`; none is stored.

### Partial-publish failure

crates.io does not support rollback; a published version is published. The
publish job splits publication into two ordered, idempotent steps:

1. `sea-forge-core` first (no path-dependency).
2. `sea-forge-cli` second (its `sea-forge-core` dependency must already be on
   the registry).

Each step is idempotent: `cargo publish` already refuses to republish an
existing version, and the step detects the "already published" pattern in
cargo's output and treats it as success. If the workflow fails between the
two steps, re-running it completes only the missing crate. The failure-report
step emits a `::error::` annotation naming the tag and pointing at the
runbook; it explicitly tells the maintainer **not** to re-tag or re-release
the same version.

Full recovery procedure: `docs/skills/release-management.md` §Partial-publish
recovery.

## 7. Branch and merge policy

The intended state, all enforced by a single repository ruleset on `main`:

- Require a pull request before merging into `main`.
- Required approving reviews: **0** (one maintainer — see constraint 8).
- Require status checks: `CI / gate`, `PR title / conventional-commit`.
- Require branches to be up to date with `main` before merging.
- Require conversation resolution.
- Require linear history.
- Block force pushes.
- Block deletion.
- Block direct updates to `main`.
- No standing bypass actor.
- Keep `main` as the default branch.

Repository merge settings (settable via `gh api` today, even on the current
plan):

- Allow squash merging: `true` (already).
- Disable ordinary merge commits: `false` (currently `true` — must change).
- Disable rebase merging: `false` (currently `true` — must change).
- Squash commit title: pull-request title.
- Squash commit body: pull-request body.
- Automatically delete head branches: `true` (already).
- Enable auto-merge so the maintainer can let GitHub merge once green.

Signed commits are **not** required initially (constraint: signing that
routinely requires bypassing is not useful).

### Required manual actions for branch protection

A `gh api` call against the rulesets endpoint on this repository returns:

```
HTTP 403 — "Upgrade to GitHub Pro or make this repository public to enable
this feature."
```

The repository is **private** on a plan that does not expose rulesets or
classic branch protection. The repository-side workflow, gate, and PR-title
checks are implemented and will run on every PR; what is missing is GitHub's
enforcement that a PR must pass them before merging. To enable enforcement,
the maintainer must take **one** of the following one-time actions:

- Make the repository public (rulesets and branch protection are free for
  public repositories), **or**
- Upgrade the plan to GitHub Pro / Team / Enterprise.

After that, apply the ruleset using the exact `gh api` command in
`docs/skills/release-management.md` §Activating branch protection.

The repository-side settings that DO work on the current plan (merge mode,
auto-delete branches, default workflow permission read, actions enabled) can
and should be applied now; the commands are in the same runbook.

### Bootstrap sequencing

1. ✅ New workflows exist on this branch (`ci-cd`).
2. Maintainer pushes the branch and opens the bootstrap PR.
3. The new check names (`CI / gate`, `PR title / conventional-commit`) run
   and become visible.
4. Maintainer merges the bootstrap PR via the path currently available.
5. Maintainer takes the one-time plan action above (or declares deferral).
6. Maintainer applies the ruleset using the documented command.
7. Maintainer re-fetches the ruleset and confirms no bypass actor remains.

The `CI / gate` check name is already stable, so it can be listed as a
required check the moment the ruleset becomes available — no temporary
bypass is needed.

## 8. Decision log

| Decision | Rationale |
| --- | --- |
| Hook manager: native `core.hooksPath` over Lefthook/Husky | zero new dependencies; devbox already pins `git` and `just`; every hook is a `just` recipe |
| Release Please simple/root `rust` release type | both crates inherit `workspace.version`; one bump moves both; no monorepo complexity |
| Pattern 1 (same-workflow follow-up publish) over pattern 2 | no manual-approval environment boundary needed; avoids race between independent events |
| Fine-grained PAT (`RELEASE_PLEASE_TOKEN`) over GITHUB_TOKEN | GITHUB_TOKEN cannot trigger CI on the bot's own release PR (F-009); no GitHub App exists in this org |
| crates.io OIDC over stored `CARGO_REGISTRY_TOKEN` | trusted publishing is supported; constraint 20 mandates it |
| Skip CodeQL | Rust coverage limited; duplicates `cargo clippy -D warnings`; security.yml documents the choice |
| Single macOS matrix entry | project already commits to macOS; proof recipe has sha256sum/shasum fallback; not decorative |
| `just ci` runs the full sweep locally; CI splits into `lint`+`test`+`macos` jobs | parallelism in CI without diverging from the local contract; the gate job is the single required name |
| Treat unexplained `skipped` as failure in the gate | this workflow has no path-filter skip logic; an unexplained skip is a regression, not a pass |
| Dependabot `chore(deps)` prefix; security updates map to `fix(deps)` | keeps PR titles Conventional-Commit-shaped so they pass `pr-title.yml` and feed Release Please; `fix(deps)` makes security fixes appear in the public changelog under "Bug Fixes" |
