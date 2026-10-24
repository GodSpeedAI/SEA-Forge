# Current status

**Status revision:** 170

**Stage:** Release-binaries job hardened per review: write token scoped to the upload step, checkout credentials not persisted

**Summary:** SEA-Forge is no longer distributed through crates.io. Every workspace crate now sets publish = false, and the release-please.yml publish job (crates-io environment, OIDC auth, two cargo publish steps) is replaced by a release-binaries job that builds sea-forge and sea-forge-server in release mode for x86_64-unknown-linux-gnu, packages sea-forge-<version>-x86_64-unknown-linux-gnu.tar.gz with SHA256SUMS, and uploads both to the GitHub Release with gh release upload --clobber. The job runs when release-please creates a release or when workflow_dispatch receives an existing tag, which re-attaches binaries to that release. Docs (release-management runbook, CI/CD architecture, CONTRIBUTING) now describe binary distribution, checksum verification and the re-attach procedure. Verification is local only; no GitHub run has executed the new job yet. The obsolete just publish-bootstrap recipe is removed. Review follow-up: GH_TOKEN is now set only on the upload step, actions/checkout uses persist-credentials: false so dependency build scripts cannot read a write credential, and the failure report states that a failed --clobber upload can leave the release without the tarball or with a mismatched SHA256SUMS until a dispatch rerun succeeds.

**Verified:**

- actionlint .github/workflows/release-please.yml exits 0 after the review fixes
- cargo metadata --no-deps --format-version 1 | jq '.packages[]|{name,publish}': all 24 workspace packages report publish == []
- actionlint .github/workflows/release-please.yml exits 0; yaml.safe_load parses it with jobs release-please and release-binaries
- Local release build: flock -o cargo.lock env RUSTC_WRAPPER= CARGO_INCREMENTAL=0 cargo build --release -j 1 --locked -p sea-forge-cli -p sea-forge-server exited 0 (10m 53s, ram-guard lowest available 2556 MB); target/release/sea-forge --version prints sea-forge 0.2.0
- Package steps rehearsed locally with the workflow's commands: tar.gz contains sea-forge-0.2.0-x86_64-unknown-linux-gnu/{LICENSE,sea-forge,sea-forge-server}; sha256sum -c SHA256SUMS reports OK

**Limits:**

- The live release PR content is produced by the Release run on main after this merges; it has been checked here by the updater run and the locked resolution, not yet by CI on the regenerated PR
- Until RELEASE_PLEASE_TOKEN exists, a release PR opened by GITHUB_TOKEN gets its pull_request runs in an approval-required state; a maintainer approves them (carried from 167)
- Creating RELEASE_PLEASE_TOKEN is a maintainer credential action, not done here (carried from 167)
- Only the obligation modality settles; prohibition and permission are refused as unsupported (carried from 164)
- The DomainForge write-authority engine is still a filename-stem approximation, not model-semantic authority (SUP-09h); readiness B-08 records it as a documented limitation (carried from 164)
- A crate added later with versioned path dependencies must be added to the extra-files list, or the next release PR fails the same way
- The release-binaries job has not run on GitHub; it is unverified until a release run or a workflow_dispatch with tag v0.2.0 completes
- Binaries are built for x86_64-unknown-linux-gnu only; no other platform is shipped
- SHA256SUMS is produced in the same job as the tarball and proves integrity against corruption, not against a compromised release; no signing is in place
- Only sea-forge and sea-forge-server ship; sandbox helper binaries sea-forge-workload-exec and sea-forge-mcp-connect are internal

**Next move:** Merge the PR after CI is green (maintainer decision; no merge in this change). After merge, verify the next release run attaches assets, or dispatch release-please.yml with tag=v0.2.0 to attach binaries to the existing v0.2.0 release.

**Evidence:**

- `.github/workflows/release-please.yml`
- `crates/sea-forge-cli/Cargo.toml`
- `crates/sea-forge-server/Cargo.toml`
- `docs/skills/release-management.md`
- `docs/explanations-and-references/ci-cd-architecture.md`
- `CONTRIBUTING.md`
- `.github/release-please-config.json`
- `Cargo.toml`
