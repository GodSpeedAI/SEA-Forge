# Current status

**Status revision:** 166

**Stage:** release-please manifest fix (Release workflow still failing after the token fix)

**Summary:** With the token fixed (#51), release-please ran and failed with "unable to parse version string" because .github/release-please-manifest.json carried a "$schema" key, which release-please reads as a package path with a version. The manifest now holds only {".": "0.1.0"}, matching [workspace.package] version 0.1.0. release-please.yml passes secrets.RELEASE_PLEASE_TOKEN || secrets.GITHUB_TOKEN (#51). Obligation settlement (#48) and Horizon #5 are merged; the live J02 revision 2 check passes 8/8.

**Verified:**

- Release run on 4780c26 (after #51) failed with: release-please failed: unable to parse version string: https://raw.githubusercontent.com/googleapis/release-please/main/schemas/manifest.json
- The manifest's only package entry "." = 0.1.0 equals Cargo.toml [workspace.package] version

**Limits:**

- Until RELEASE_PLEASE_TOKEN exists, a release PR opened by GITHUB_TOKEN gets its pull_request runs in an approval-required state; a maintainer approves them so the required CI / gate and conventional-commit checks execute
- Creating RELEASE_PLEASE_TOKEN (docs/skills/release-management.md section 7) is a maintainer credential action, not done here
- Only the obligation modality settles; prohibition and permission are refused as unsupported (carried from 164)
- The DomainForge write-authority engine is still a filename-stem approximation, not model-semantic authority (SUP-09h); readiness B-08 records it as a documented limitation (carried from 164)
- A release PR opened by release-please is not merged here: merging it publishes to crates.io and needs the maintainer's go

**Next move:** Merge once CI is green; confirm the next Release run on main succeeds and opens a release PR; the maintainer decides when to merge it.

**Evidence:**

- `.github/release-please-manifest.json`
- `.github/workflows/release-please.yml`