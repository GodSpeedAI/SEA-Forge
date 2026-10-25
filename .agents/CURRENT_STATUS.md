# Current status

**Status revision:** 175

**Stage:** release provenance restored

**Summary:** Restores the signed build-provenance attestation step in release-please.yml and its verification section in docs/skills/release-management.md exactly as merged in #58 (a914709). The squash of #62 was built from a tree that predated #58 and reverted both files; no other #58 or #59 file was affected (checked by intersecting the changed-file sets).

**Verified:**

- git diff a914709 -- .github/workflows/release-please.yml docs/skills/release-management.md is empty after this change
- the changed-file intersection of #58/#59 with #62 is only these two files plus CURRENT_STATUS

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
- obligation.rs record_closure 'window is still open; a closure is recorded once it closes' still returns cep_evidence_refused by decision (it concerns recording a closure, not settlement)
- ledger repair is offline by contract but does not check that the server is stopped; the operator must stop the server first
- startup now refuses a ledger whose chain is broken as well as mid-ledger corruption (fail closed); a crash-torn final record is still repaired silently at startup
- a complete final record missing only its newline is quarantined as one torn record

**Next move:** Merge, then replay #60 and #61 onto main with the fixed helper (it now replays only each branch's own diff).

**Evidence:**

- `.github/workflows/release-please.yml`
- `docs/skills/release-management.md`
