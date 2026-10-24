# Current status

**Status revision:** 165

**Stage:** release-please token fix (Release workflow failing on every push to main)

**Summary:** The Release workflow's release-please job failed on every push to main because its token input referenced secrets.RELEASE_PLEASE_TOKEN, which does not exist in the repository (zero Actions secrets), so the input was empty. release-please.yml now passes secrets.GITHUB_TOKEN under the job's existing contents:write and pull-requests:write permissions. The cargo-publish OIDC job is unchanged. Release PRs opened with GITHUB_TOKEN do not trigger other workflows, so their CI and gate checks need a follow-up decision before a release merge.

**Verified:**

- python yaml.safe_load parses .github/workflows/release-please.yml
- gh secret list and gh api actions/secrets report zero repository secrets, so RELEASE_PLEASE_TOKEN is absent
- gh run 40062078573 and the last 30 release-please runs all conclude failure with the empty-token error
- actionlint is not installed on this host, so workflow lint was not run

**Limits:**

- Release PRs created with GITHUB_TOKEN do not trigger workflows, so CI / gate and conventional-commit will not run on the release PR; gate and conventional-commit are required on main
- Mitigation options, none implemented: create the RELEASE_PLEASE_TOKEN secret per docs/skills/release-management.md section 7, or a GitHub App token; both need a maintainer action
- No Rust build or test was run (memory-constrained host); the change touches only the workflow token input
- Docs in docs/skills/release-management.md still describe RELEASE_PLEASE_TOKEN as the live token

**Next move:** Maintainer decides how release PRs get required checks (create RELEASE_PLEASE_TOKEN or a GitHub App token) before the next release merge; then review and merge this PR.

**Evidence:**

- `.github/workflows/release-please.yml`
