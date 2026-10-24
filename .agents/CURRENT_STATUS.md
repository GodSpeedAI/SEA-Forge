# Current status

**Status revision:** 165

**Stage:** release-please token fix (Release workflow failing on every push to main)

**Summary:** The Release workflow's release-please job failed on every push to main since 2026-10-13 because its token input referenced secrets.RELEASE_PLEASE_TOKEN, which does not exist in the repository (zero Actions secrets), so the input was empty. release-please.yml now passes secrets.RELEASE_PLEASE_TOKEN || secrets.GITHUB_TOKEN: the PAT when present (release PRs trigger CI, F-009), otherwise the job's GITHUB_TOKEN under its existing contents:write and pull-requests:write permissions. The cargo-publish OIDC job is unchanged. The release-management skill and CI/CD architecture docs describe the fallback and that GitHub holds a GITHUB_TOKEN release PR's pull_request runs for maintainer approval. Carried forward from revision 164: obligation settlement merged (#48) and Horizon #5 merged with the live J02 revision 2 check passing 8/8 against a real RealityTrace.

**Verified:**

- python yaml.safe_load parses .github/workflows/release-please.yml
- gh secret list and gh api actions/secrets report zero repository secrets, so RELEASE_PLEASE_TOKEN is absent
- gh run 40062078573 and the last 30 release-please runs all conclude failure with the empty-token error
- actionlint is not installed on this host, so workflow lint was not run; CI parses the workflow

**Limits:**

- Until RELEASE_PLEASE_TOKEN exists, a release PR opened by GITHUB_TOKEN gets its pull_request runs in an approval-required state; a maintainer approves them so the required CI / gate and conventional-commit checks execute
- Creating RELEASE_PLEASE_TOKEN (docs/skills/release-management.md section 7) is a maintainer credential action, not done here
- No Rust build or test was run for this change; it touches only the workflow token input and docs
- Only the obligation modality settles; prohibition and permission are refused as unsupported (carried from 164)
- The DomainForge write-authority engine is still a filename-stem approximation, not model-semantic authority (SUP-09h); readiness B-08 records it as a documented limitation (carried from 164)

**Next move:** Merge once CI is green; a maintainer may create RELEASE_PLEASE_TOKEN to restore automatic CI on release PRs, otherwise approve the release PR's runs.

**Evidence:**

- `.github/workflows/release-please.yml`
- `docs/skills/release-management.md`