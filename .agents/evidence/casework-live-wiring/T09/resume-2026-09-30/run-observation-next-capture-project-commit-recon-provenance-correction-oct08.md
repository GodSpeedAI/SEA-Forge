# Additive provenance correction: Next capture/commit recon

Date: 2026-10-08. This note supplements, and does not replace or rewrite,
`run-observation-next-capture-project-commit-recon-oct08.md`.

The earlier recon's statement “the repository has no graph manifest” was too
broad if read as saying Graft could not provide useful source context. The
actual initial freshness check reported no graph manifest. A later
`graft_file_api` query for
`apps/godspeed-casework-go/internal/server/run_observation_manager.go`
succeeded and returned the manager skeleton. The earlier statement should be
read narrowly as describing that freshness-check result, not as a claim that
all Graft retrieval was unavailable.

The earlier source hashes and line spans describe the source snapshot inspected
for that recon only. The manager and tests have since changed; those recorded
hashes are not asserted to identify the current worktree. No source conclusion
or runtime result is changed by this provenance correction.
