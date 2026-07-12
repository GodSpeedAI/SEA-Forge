# Current Status

Updated: 2026-07-12

## Objective

Professionalize CI/CD for `GodSpeedAI/SEA-rs`: extend the `just` command
boundary, add strategic Git hooks, rebuild GitHub Actions as thin orchestration
with a stable gate, add Release Please + crates.io OIDC publishing, add
Dependabot and security automation, document everything for a solo maintainer,
and validate the whole pipeline end to end. Reports live under
`.agents/reports/`.

## Worktree State

On branch `ci-cd` (created from `main`). Not yet pushed or opened as a PR —
that step is gated on Phase 7 (workflows must exist so the bootstrap PR can
observe the real `CI / gate` check name before activating protection). All
work is additive; no existing Rust code or v0.1 schema is touched except the
`Cargo.toml` repository URL (F-007) and `scripts/check-agent-context.sh`
prefix matching (F-001).

## Changed Files

All phases below are implemented (not pending). This list reflects the final
state of the `ci-cd` branch.

- `.agents/reports/ci-cd-audit.md` (findings register — Phase 2)
- `.agents/reports/ci-cd-findings.json` (machine-readable register)
- `.agents/reports/ci-cd-final-report.md` (closing report)
- `.agents/CURRENT_STATUS.md` (this file)
- `scripts/check-agent-context.sh` (F-001 fix + nested-path exclusion fix)
- `Cargo.toml` (F-007 fix; corrected repository URL)
- `justfile` (Phase 4 recipe surface)
- `.githooks/{pre-commit,pre-push,post-checkout}` (Phase 6)
- `.github/workflows/{ci,pr-title,release-please,security}.yml` (Phases 7, 10, 11)
- `.github/{dependabot.yml,release-please-config.json,release-please-manifest.json}` (Phases 10, 11)
- `.github/pull_request_template.md` (Phase 9)
- `docs/ci-cd-architecture.md`, `docs/skills/release-management.md`, `CONTRIBUTING.md` (Phases 9, 13, 15)
- `README.md` (Phase 13 happy path + recipe table)
- `AGENTS.md`, `.gitignore`, `.githooks/post-commit` (Understand-Anything knowledge-graph support + auto-update hook)
- `docs/explanations-and-references/ci-cd-architecture.md` (relocated from `docs/`; permissions section corrected)
- `.github/workflows/ci.yml` (stale `actions: write` comment removed)

## Completed

- Phase 1 inspection: repo substrate, manifests, hooks, workflows, GitHub
  settings, registry state, ruleset availability, secrets, environments.
- Phase 2 baseline: ran fmt, cargo check, clippy, test, deny locally;
  captured before-numbers; wrote the findings register and machine-readable
  companion.
- Verified key facts against the source: `release-please-action@v4.4.1`
  (SHA `5c625bfb5d1ff62eadeeb3772007f7f66fdcf071`) documents runtime output
  `release_created` for root-component releases; `crates.io` returns 404 for
  both crates (unpublished); GitHub rulesets API returns 403 on this private
  repo's plan; `can_approve_pull_request_reviews=false` confirms the
  GITHUB_TOKEN-cannot-trigger-its-own-CI constraint (F-009).
- Phases 4–15: **all implemented.** Phase 4 justfile recipe surface; Phase 5
  environment pinning documented; Phase 6 `.githooks/`; Phase 7 rewritten
  `ci.yml` with stable gate + `pr-title.yml` + `security.yml`; Phase 9 PR
  template + `CONTRIBUTING.md`; Phase 10 `release-please.yml` + config +
  publish job + partial-failure recovery; Phase 11 Dependabot +
  dependency-review; Phase 13 architecture doc + README happy path;
  Phase 14 local validation (green); Phase 15 release skill. The final
  report at `.agents/reports/ci-cd-final-report.md` closes the directive.

> The per-phase "pending" list that previously occupied this section was a
> pre-implementation snapshot and is superseded by the final report. All
> repository-side implementation is complete; only the manual/remote actions
> under **Remaining** below are outstanding.

- Understand-Anything knowledge graph integrated: the `.ua/` projection is
  documented in `AGENTS.md` and gitignored as rebuildable, kept fresh by a new
  `.githooks/post-commit` hook (zero-token meta bump on non-source commits;
  in-session trigger on source commits). The hook was hardened under a review
  pass: JSON config parse, `META` read via argv, atomic write, explicit
  git-dir resolution, and bail-on-diff-failure. CI permissions doc corrected
  (lint job is `contents: read`; release-please uses `RELEASE_PLEASE_TOKEN`,
  not the default `GITHUB_TOKEN`).
- CodeRabbit review replay (21 ci-cd findings) re-verified against current
  code: 20 of 21 were already addressed by the committed ci-cd work; the one
  still-valid item (README `just pre-push` warm-runtime estimate) was corrected
  to the measured ~20s (`just ci` under devbox, all gates green).

## Verification

- Baseline `cargo fmt --all -- --check`: passed.
- Baseline `cargo check --workspace --all-targets --locked`: passed (10.85s).
- Baseline `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed (5.09s).
- Baseline `cargo test --workspace --all-features --locked`: 20 passed, 0 failed.
- Baseline `cargo deny check licenses bans sources`: passed (cosmetic unmatched-allowlist warnings, F-017).
- `gh api` probes for repo settings, rulesets, secrets, environments,
  releases, tags, registry state: completed; results recorded in
  `.agents/reports/ci-cd-audit.md`.
- Phase 14 local validation: complete and green (see final report §7).
  `just doctor/fmt-check/typecheck/lint/test/security/build/release-check/ci`
  all pass; hook behavior verified (pre-commit rejects malformed Rust; all
  green changes commit normally). Outstanding: `sea-forge-cli` `cargo publish
  --dry-run` fails because `sea-forge-core` is not yet on the registry
  (F-010 bootstrap state, expected — not a defect).

## Remaining

All repository-side work is complete. Only **manual / remote** actions remain
(cannot be completed from this session); see final report §9 for exact
commands:

1. Set the merge mode (works on the current plan): squash-only.
2. Create the `RELEASE_PLEASE_TOKEN` fine-grained PAT repo secret
   (`contents: write`, `pull-requests: write`, this repo only).
3. Create the `crates-io` GitHub environment.
4. One-time crates.io bootstrap publish + trusted-publisher linkage for both
   crates (F-010); both return 404 today.
5. Activate branch protection — blocked on the current private-repo plan;
   requires making the repo public or upgrading, then run the ruleset command
   in `docs/skills/release-management.md` §9.
6. Merge this `ci-cd` PR once `CI / gate` is green on it.

## Blockers

- F-002: branch protection cannot be enabled from this authenticated session
  because the repository is private on a plan that does not expose rulesets
  or classic branch protection (HTTP 403). Resolution requires a one-time
  maintainer action (make the repo public, OR upgrade the plan). The exact
  `gh api` commands are produced in the architecture doc and final report so
  the maintainer can complete the step without re-deriving it.

## Decisions

- Hook manager: native `core.hooksPath` pointing at a checked-in `.githooks/`
  directory, installed by `just hooks-install`. Rationale: zero new
  dependencies (no Node/Lefthook), devbox already pins `git` and `just`, and
  every hook simply invokes a `just` recipe. Satisfies "Do not add a second
  Git-hook manager" (no first one exists) and "Hooks must call `just` recipes."
- Release Please: simple/root mode. Both crates inherit
  `version.workspace = true`, so one root-level release unit bumps both. A
  single registry (crates.io) means cross-registry version sync (Phase 10.3)
  is automatically satisfied; `just release-check` still verifies the
  workspace version matches the tag.
- Release-please credential: fine-grained PAT stored as `RELEASE_PLEASE_TOKEN`
  (repo secret). Rationale: no GitHub App exists in this org; GITHUB_TOKEN
  cannot trigger CI on its own release PR (F-009). Scope: `contents: write`
  and `pull-requests: write` on this repo only; rotation plan documented.
- crates.io publishing: OIDC trusted publishing is the target end state, but
  both crates are unpublished (F-010). Provide `just publish-bootstrap` for
  the one-time classic-token publish and document the manual crates.io side.
  The automated `release-please.yml` publish job uses OIDC and will work
  after the bootstrap completes.
- `cargo publish` partial failure: split into two ordered steps
  (`-p sea-forge-core` then `-p sea-forge-cli` because cli depends on core).
  This is **intended recovery behavior**, not a verified guarantee: each step
  is designed to detect cargo's "already published" wording and treat it as
  success, so a re-run after a partial publish completes only the missing
  crate. That "already published" handling remains **unverified** until a real
  partial-publish retry confirms the regex matches the current cargo's output
  (the regex and its caveats are in final report §10/§11). The job fails
  loudly and names which crate did not publish.
- macOS CI matrix: kept (the project already commits to it and the proof
  recipe has sha256sum/shasum fallback). Feeds the same `gate` job.
- CodeQL: enabled for Rust as **advisory** (non-blocking) per Phase 7.
- Dependabot: weekly, grouped compatible updates, Conventional Commit
  titles mapped to `chore(deps)` (security updates map to `fix(deps)`).
