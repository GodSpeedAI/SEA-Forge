# SEA Forge CI/CD Audit

Updated: 2026-07-12

Scope: full CI/CD professionalization of `GodSpeedAI/SEA-rs`. This document is
the evidence-backed findings register required by the CI/CD directive Phase 2.
The companion machine-readable register is `ci-cd-findings.json` next to this
file. Severity legend: **blocker** (prevents correct release/CI), **high**
(reliability or security gap), **medium** (developer-experience or debt),
**low** (cosmetic), **informational** (verified fact, no action).

## 1. Initial architecture

| Aspect | State on 2026-07-12 |
| --- | --- |
| Languages / runtimes | Rust 1.92.0 stable (rust-toolchain.toml pinned) |
| Package manager | Cargo (workspace, 2 crates), `Cargo.lock` committed |
| Crates | `sea-forge-core`, `sea-forge-cli` (both inherit `workspace.version = "0.1.0"`) |
| Registries targeted | crates.io only (both currently **unpublished** — 404) |
| Dev environment | Devbox (`devbox.json` + `devbox.lock`) + direnv + `rust-toolchain.toml` |
| Canonical commands | `just` recipes in `justfile` |
| Existing CI | One workflow `.github/workflows/ci.yml` (ubuntu + macos, sequential setup→doctor→check→test per OS) |
| Gate job | None |
| Release system | None (no tags, no releases, no changelog, no release-please config) |
| Publication | None (no publish workflow, no secrets, no environments) |
| Branch protection | None available — repo is **private** on a plan that returns HTTP 403 for both rulesets and classic branch protection |
| Merge mode | Default merge method = **MERGE**; merge-commit, rebase, squash all allowed; `deleteBranchOnMerge=true` already set |
| Hooks | None (only Git's sample hooks installed) |
| Secrets at rest | SOPS + age (`secrets/dev.enc.env`); no long-lived registry tokens in the repo |
| Security tooling | `gitleaks detect` invoked in `just check`; `cargo deny check`; no Dependabot config; no CodeQL; no dependency-review |
| Workflow perms | Default workflow permission = **read**; SHA pinning not enforced |
| Dependabot | Not configured |

## 2. Baseline measurements (before)

| Command | Result | Runtime | Notes |
| --- | --- | --- | --- |
| `cargo fmt --all -- --check` | pass | <1s | clean |
| `cargo check --workspace --all-targets --locked` | pass | 10.85s | clean |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | pass | 5.09s | clean |
| `cargo test --workspace --all-features --locked` | pass | ~1s (test phase) | 20 passed, 0 failed |
| `cargo deny check licenses bans sources` | pass | <1s | warns about unmatched allowlist entries (harmless) |
| `gitleaks detect` | not run in baseline | — | already wired into `just check` |
| `just check` (full chain) | **fail** | 3.3s | blocked by F-001 |
| `just context-check` | **fail** | <1s | F-001 |
| `just proof` (spec-minimum §12.2) | not run in baseline | — | known-good per CURRENT_STATUS.md |
| `cargo deny check advisories` | requires network (RustSec fetch) | — | deferred; not gated |

## 3. Findings register

Machine-readable companion: `ci-cd-findings.json`.

| ID | Severity | Location | Evidence | Failure mode / impact | Root cause | Correction | Validation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| F-001 | blocker | `scripts/check-agent-context.sh:47` | regex `^(\.agents/CURRENT_STATUS\.md|target/|\.logs/)$` anchors at `$`, so `.logs/subtask2.log` is not matched and is treated as a project change requiring a status bump | any tracked file under `target/` or `.logs/` blocks `just check`, CI, and the pre-push hook even when CURRENT_STATUS is current | regex intends prefix match but the `$` anchor defeats it; README claims `.logs/` is excluded but it is not | change anchor to `($|/)` so directory prefixes match | `just check` passes when only `.logs/*` changed |
| F-002 | blocker | GitHub repo settings | `gh api .../branches/main/protection` and `.../rulesets` both return HTTP 403 ("Upgrade to GitHub Pro or make this repository public") | no enforcement of PR-required, no force-push block, no required checks, no linear history on `main` | repository is **private** on a plan without rulesets/branch protection | implement all workflows + gate; provide exact `gh api` ruleset command; document the required manual action (make repo public, OR upgrade plan) — cannot be automated here | after maintainer action: re-run `gh api .../rulesets` returns 200 with the configured ruleset |
| F-003 | blocker | `.github/workflows/ci.yml` | single `foundation` job (and `macos` duplicate) runs sequential `setup→doctor→check→test`; **no gate job**; required-check name would be `CI / Shell-SPEC gates (ubuntu)` which is fragile | if/when a matrix or extra job is added the required-check name changes; partial skips leave a permanently pending check; cannot satisfy Phase 7's gate contract | pre-Phase-7 design | rewrite `ci.yml` with granular jobs (`lint`, `test`, `build`, `security`) plus a stable `gate` job using `if: always()` and explicit `needs.<job>.result` inspection | Phase 14: PR opened, `CI / gate` is the only required name, an intentional failure blocks merge |
| F-004 | high | `.github/workflows/ci.yml:42-63` | `macos` job duplicates `setup→doctor→check→test`; no separate cross-platform value documented | doubles CI minutes, doubles cold-cache penalty, no failure mode currently caught only on macOS that justifies the cost | copy-paste from ubuntu job | keep macOS as a separate job (Rust CLI ships cross-platform, project already commits to it), but route through the same `just` recipes and have it feed `gate` | same recipes produce same result on both OSes |
| F-005 | high | `.github/workflows/ci.yml` (and elsewhere) | `actions/checkout@v4`, `actions/upload-artifact@v4`, `jetify-com/devbox-install-action@v0.13.0` are pinned to a **floating tag**, not a SHA; `sha_pinning_required=false` at the repo level | mutable references can be retagged; supply-chain risk; violates Phase 7 quality requirement | pre-Phase-7 design | **Partially remediated.** Workflow SHA pinning is complete (every action pinned to a verified SHA with a version comment — verified via `rg "uses:" .github/workflows`). The repo-level `sha_pinning_required` control is **still false** and remains an unresolved residual risk: it is not exposed via the per-repo API path on this plan, so a future workflow added by another contributor could still float a tag. Mitigation: enable `sha_pinning_required` once the plan allows it. | SHA refs: verified done. `sha_pinning_required=false`: residual (see final report §11) |
| F-006 | high | repo settings | `viewerDefaultMergeMethod=MERGE`, `mergeCommitAllowed=true`, `rebaseMergeAllowed=true` | squash-merge is required so the PR title becomes the canonical commit title (Release Please depends on this) | default GitHub settings | set squash as the default; disable merge-commit and rebase; keep `deleteBranchOnMerge=true` | `gh api repos/... --jq '.viewerDefaultMergeMethod,.mergeCommitAllowed,.rebaseMergeAllowed'` returns `SQUASH,false,false` |
| F-007 | medium | `Cargo.toml:13` | `repository = "https://github.com/sea-forge/sea-rs"` — wrong; real repo is `https://github.com/GodSpeedAI/SEA-rs` | crates.io metadata would publish a broken link; release-please changelog links would 404 | typo or stale placeholder | correct the URL | `cargo metadata --no-deps` shows the right URL |
| F-008 | high | release pipeline | no `release-please.yml`, no `release-please-config.json`, no `release-please-manifest.json`, no publish workflow | releases are manual, error-prone, and there is no path from PR title → tag → crate publication | never built | add release-please config (simple/root mode for the single workspace version) plus a publish job gated on the verified `release_created` output of `release-please-action@v4.4.1` (SHA `5c625bfb...`) | Phase 14: simulated release PR run produces a tag and triggers the publish job in `--dry-run` |
| F-009 | blocker | release-please credential strategy | GitHub default workflow permission is `read`; `can_approve_pull_request_reviews=false`. When `release-please-action` opens its release PR using `GITHUB_TOKEN`, that push **does not trigger** the required CI workflow (GitHub prevents GITHUB_TOKEN-caused workflow recursion) | the release PR would never receive the required `CI / gate` check and could not be merged under protection | GitHub Actions design rule | use a fine-grained PAT stored as repo secret `RELEASE_PLEASE_TOKEN` (scoped: contents write, pull-requests write, workflows unchanged) and document its creation/rotation; never print it | Phase 14: the release PR created by the bot triggers `CI / gate` automatically |
| F-010 | high | crates.io bootstrap | `curl https://crates.io/api/v1/crates/sea-forge-core` → **404**; same for `sea-forge-cli` | trusted publishing (OIDC) for crates.io requires (a) one manual classic-token publish and (b) repository linkage on crates.io before trust can be configured — neither has been done | the crates have never been published | provide a one-time `just publish-bootstrap` recipe using a manually-supplied `CARGO_REGISTRY_TOKEN`; document the manual crates.io side; the automated `release-please.yml` publish job only works after this completes | maintainer runs the bootstrap once; subsequent releases publish via OIDC |
| F-011 | high | hook layer | no `core.hooksPath`, no `.githooks/`, no Lefthook/Husky/pre-commit config; only Git's sample hooks present | fast failures caught only after push, not at commit; the directive's pre-commit/pre-push contract is unsatisfied | never built | add `.githooks/{pre-commit,pre-push,post-checkout}` and `just hooks-install` that sets `git config core.hooksPath .githooks`; all hooks invoke `just pre-commit` / `just pre-push` so the contract is the recipes | Phase 14: misformatted staged file is caught before commit; deliberate test failure is caught before push |
| F-012 | medium | `justfile` | no `fmt`/`fmt-check`/`lint`/`typecheck`/`ci`/`check-fast`/`fix`/`pre-commit`/`pre-push`/`pr-check`/`pr`/`hooks-install`/`release-check`/`sync` recipes; existing `check` bundles fmt+clippy+deny+gitleaks so the gate job cannot parallelize or report a single failure category | cannot meet Phase 4 contract; CI cannot run granular gates; no quick local feedback path | pre-Phase-4 design | extend justfile additively, keep existing recipes, refactor `check` to call the new granular recipes | Phase 14: every recipe in Phase 4 exists, is non-empty, and is invoked by either a hook or a workflow |
| F-013 | medium | PR standards | no `.github/pull_request_template.md`; no PR-title policy workflow | PR titles are not Conventional Commit shaped → Release Please cannot derive releases reliably | never built | add a PR template; add `pr-title.yml` enforcing `^(feat|fix|perf|refactor|docs|test|build|ci|chore|style)(\(.+\))?!?: .+$` | Phase 14: a deliberately bad title fails the check |
| F-014 | medium | security automation | no `.github/dependabot.yml`; no dependency-review workflow; no CodeQL | stale dependencies; no automated PR-time vulnerability check | never built | **Remediated with one documented waiver.** Added `dependabot.yml` (cargo + github-actions, weekly, grouped, `chore(deps)`/`fix(deps)` titles) and `dependency-review-action` (fail-on-severity: critical) in `security.yml`. **CodeQL is explicitly waived and skipped**: Rust coverage is limited and would duplicate `cargo clippy -D warnings`; the waiver is recorded in `docs/ci-cd-architecture.md` §4/§8 and the `security.yml` header. Remaining security coverage at PR time: `cargo deny check` (licenses/bans/sources), `gitleaks detect`, and dependency-review; weekly `cargo deny check advisories` in `security.yml`. | dependabot opens a correctly-titled PR; dependency-review reports on a vulnerable-version PR; CodeQL confirmed skipped-by-decision |
| F-015 | medium | `docs/ci-cd-architecture.md` | does not exist | no single source of truth for the local→remote flow, the Just contract, the release causal chain, or the publish credential boundary | never built | write `docs/ci-cd-architecture.md` with Mermaid diagram of the tag→release→publish causal chain | maintainer sign-off |
| F-016 | medium | `CONTRIBUTING.md` | does not exist | no documented happy path for a new contributor; CI/CD conventions live only in chat/specs | never built | add `CONTRIBUTING.md` with the happy path, hook install, common failures, and release flow pointers | maintainer sign-off |
| F-017 | low | `deny.toml:21-22` | `"ISC"` and `"Unicode-DFS-2016"` are unmatched (warning) | harmless noise but signals stale allowlist | upstream license changes; allowlist predates current deps | leave as-is (cargo-deny still passes); consider pruning in a separate maintenance PR | not gated; no action |
| F-018 | low | `.agents/CURRENT_STATUS.md` | listed "35 passed" but `cargo test` shows 20 | drift in handoff state | the 35 likely included `--all-targets` integration tests in an earlier run | **Resolved.** Re-ran `cargo test --workspace --all-features --locked`; actual count is 20 passed, 0 failed. CURRENT_STATUS.md and the final report now both record 20 consistently. | refreshed; final tally 20 across all references |
| F-019 | informational | `devbox.json:3-12` | Devbox pins git 2.49.0, rustup 1.28.2, just 1.55.1, direnv 2.37.0, sops 3.10.2, age 1.2.1, gitleaks 8.30.1, cargo-deny 0.19.9 — pinned at the minor, not the patch | minor-floating pins can drift between contributors' `devbox install` and CI | Devbox's lockfile (`devbox.lock`) actually fixes the hash, so this is not a real reproducibility hole — Devbox resolves via `devbox.lock` first | leave minor pins, rely on `devbox.lock` for true reproducibility; document this in `docs/ci-cd-architecture.md` | `devbox.lock` present and committed |
| F-020 | informational | repo settings | `gh api .../actions/secrets` and `.../actions/variables` both return empty | clean slate — no leftover `NPM_TOKEN`, `PYPI_TOKEN`, `CARGO_REGISTRY_TOKEN`, or workflow vars to clean up | never had any | preserve the clean state; only add `RELEASE_PLEASE_TOKEN` (fine-grained PAT) for the release PR trigger; cargo publish uses OIDC after the bootstrap publish | `gh api .../actions/secrets` shows exactly the expected set |

## 4. Removed debt

(as phases complete — filled in the final report)

## 5. Deferred findings and why

- **F-017** (deny.toml unmatched allowlist entries): cosmetic; no correctness impact; deferred to a separate maintenance PR to keep this CI/CD change focused.
- **Cargo workspace CI matrix across multiple Rust versions**: the project pins a single Rust version (`rust-toolchain.toml`) deliberately; matrixing against older versions would contradict the MSRV policy. Deferred indefinitely.
- **Self-hosted runner, merge queue, containers**: explicitly out of scope (Phase 7 anti-goals).

## 6. Before-and-after workflow timing

| Stage | Before | After | Notes |
| --- | --- | --- | --- |
| cold-cache `just check` | ~17s component time (blocked by F-001) | TBD after Phase 4 refactor | measured in Phase 14 |
| `cargo check --locked` | 10.85s | unchanged | not optimized |
| `cargo clippy` | 5.09s | unchanged | not optimized |
| `cargo test` | ~1s | unchanged | 20 tests, fast |
| CI wall-clock (single OS) | ~setup+doctor+check+test sequential | TBD | gate job enables parallelism in Phase 7 |

(Filled in with measured numbers in the final report.)
