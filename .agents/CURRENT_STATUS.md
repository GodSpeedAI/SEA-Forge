# Current status

**Status revision:** 167

**Stage:** release-please config fix: rust strategy cannot edit inherited member versions; switched to simple + extra-files (validated by dry run, not merged)

**Summary:** The Release run on b09cef6 fails with "release-please failed: value at path package.version is not tagged" while fetching crates/sea-forge-core/Cargo.toml. The cause is in release-please 17.6.0 (bundled by release-please-action v5.0.0): the built-in rust strategy walks every [workspace] member and rewrites each member's package.version with a TOML tagged-value edit. All members use version.workspace = true, so package.version is a table, not a string. The config now uses release-type simple with extra-files: a TOML jsonpath bumps [workspace.package] version in Cargo.toml, and a second TOML jsonpath bumps the path-dependency (workspace member) entries in Cargo.lock. Crate manifests are unchanged. The updater was checked locally against the real Cargo.toml and Cargo.lock (1 line in Cargo.toml, 24 sea-forge-* entries in Cargo.lock). The release-please CLI dry run is recorded in the PR body.

**Verified:**

- Release run on b09cef6 failed with: release-please failed: value at path package.version is not tagged (after reading the manifest and listing workspace crate versions)
- release-please 17.6.0 (the version pinned by release-please-action v5.0.0 package-lock) src/strategies/rust.js rewrites every [workspace] member Cargo.toml via CargoToml, which calls replaceTomlValue on ['package','version'] and throws when that value is not a tagged string
- Local check: GenericToml with jsonpath $.workspace.package.version changes only the root Cargo.toml version line to the new version; $.package[?(!@.source)].version changes exactly the 24 sea-forge-* lock entries and no registry entries
- release-please 17.6.0 CLI dry run against the pushed branch config (release-pr --dry-run, read-only, no PR created): computed 0.1.0 -> 0.2.0 from the commits since the manifest version, reported 'Would open 1 pull requests', and listed 5 updates: CHANGELOG.md, Cargo.toml (GenericToml), Cargo.lock (GenericToml), .github/release-please-manifest.json, and version.txt (missing in the repo, skipped by release-please with a warning). No error. The one 'commit could not be parsed' log line is a pre-existing non-conventional commit and is ignored by release-please

**Limits:**

- Dry run only: no release PR, tag, GitHub Release, or crates.io publish was created; the dry run reads config from the pushed branch, not main
- Until RELEASE_PLEASE_TOKEN exists, a release PR opened by GITHUB_TOKEN gets its pull_request runs in an approval-required state; a maintainer approves them so the required CI / gate and conventional-commit checks execute
- Creating RELEASE_PLEASE_TOKEN (docs/skills/release-management.md section 7) is a maintainer credential action, not done here
- Only the obligation modality settles; prohibition and permission are refused as unsupported (carried from 164)
- The DomainForge write-authority engine is still a filename-stem approximation, not model-semantic authority (SUP-09h); readiness B-08 records it as a documented limitation (carried from 164)
- A release PR opened by release-please is not merged here: merging it publishes to crates.io and needs the maintainer's go

**Next move:** Merge the PR after CI is green and only with the maintainer's go; then confirm the next Release run on main succeeds and opens a release PR (no release or tag is cut before the maintainer merges that release PR). The live Cargo.lock update has been checked only locally and by dry run, not by a live Release run.

**Evidence:**

- `.github/release-please-config.json`
- `.github/release-please-manifest.json`
- `.github/workflows/release-please.yml`
- `docs/explanations-and-references/ci-cd-architecture.md`
