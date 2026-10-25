# Current status

**Status revision:** 172

**Stage:** Release tarball carries a signed build-provenance attestation

**Summary:** The release-binaries job now runs actions/attest-build-provenance (v4.2.2, pinned by SHA) on the packaged tarball before uploading it. The job gains id-token: write and attestations: write. The attestation is a Sigstore-signed (keyless) SLSA provenance statement binding the tarball's sha256 to this repository, workflow and commit, so a downloader can detect a tarball that this workflow did not build with gh attestation verify. The release-management runbook documents the verification command and that releases attached before this change need a re-attach to gain an attestation. A re-attach dispatched from main for an older tag builds the tag's commit while the run is for main's head; the job then skips the attestation with a warning instead of naming the wrong source commit, and the runbook verifies the source commit with --source-digest.

**Verified:**

- actionlint .github/workflows/release-please.yml exits 0
- yaml.safe_load parses the workflow; release-binaries permissions are contents: write, id-token: write, attestations: write

**Limits:**

- The live release PR content is produced by the Release run on main after this merges; it has been checked here by the updater run and the locked resolution, not yet by CI on the regenerated PR
- Until RELEASE_PLEASE_TOKEN exists, a release PR opened by GITHUB_TOKEN gets its pull_request runs in an approval-required state; a maintainer approves them (carried from 167)
- Creating RELEASE_PLEASE_TOKEN is a maintainer credential action, not done here (carried from 167)
- Only the obligation modality settles; prohibition and permission are refused as unsupported (carried from 164)
- The DomainForge write-authority engine is still a filename-stem approximation, not model-semantic authority (SUP-09h); readiness B-08 records it as a documented limitation (carried from 164)
- A crate added later with versioned path dependencies must be added to the extra-files list, or the next release PR fails the same way
- Binaries are built for x86_64-unknown-linux-gnu only; no other platform is shipped
- Only sea-forge and sea-forge-server ship; sandbox helper binaries sea-forge-workload-exec and sea-forge-mcp-connect are internal
- The provenance attestation proves which workflow run built the tarball; it does not make the build reproducible, and trust still rests on the GitHub-hosted runner and the pinned toolchain
- The attestation step is verified by actionlint locally; it runs for real on the next release or tag re-attach

**Next move:** Merge after CI is green, then dispatch release-please.yml with tag=v0.2.0 to re-attach v0.2.0 with an attestation and check gh attestation verify on the download.

**Evidence:**

- `.github/workflows/release-please.yml`
- `docs/skills/release-management.md`
