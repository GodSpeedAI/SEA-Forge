# Current status

**Status revision:** 199

**Stage:** capability-harvest: codex pass-1 findings processed (fixes + disputes); pushing review round 2

**Summary:** Codex pass 1 (Critical 0/High 5/Medium 10/Low 1) triaged. Fixed: H1 macOS destroy_sweep scoped per sandbox instance; H2 Windows publish rebuilt (temp handle held through the release window, by-handle rename with pin released, repin with directory-ID comparison, published-entry FileId + reparse checks); H4 interactive jail carries the cgroup lease through the ACP session (LeafCleanupGuard runs the verified leaf kill before the group kill); M6 staging content verification (in-place same-uid tamper now detected AND the tampered entry removed); M7 guard serialization for the egress tally; M9 CONNECT parsing (normalized CONNECT_STATUS marker, full header-block read before any token); M10 ci.yml authority narrowed (test process migrates into the delegated subtree; mount-root procs chown removed); M11 per-call staging directories (concurrent writers never share); M12 changeset scanner refuses .sf-staging leftovers; M13 no-async-kernel fail-closed (if-capture preserves cargo's status; absence only via cargo's own not-found text); M14 Windows completion verifies the job empty; M15 disputed (unshare --pid --fork establishes the namespace before the helper runs; a setup failure can never produce a contained Completed run). H3/H5 admission-level refusal disputed with the honest-degrade contract; recorded as the lifecycle-admission debt. L16 record staleness deferred to the final record rewrite.

**Verified:**

- Local: fmt clean, clippy -D warnings across sandbox/os/agent/server/core/changeset/case-runner, sandbox 23 targets green (27 lib incl. the in-place tamper test), agent/changeset/case-runner/server suites green, no-async-kernel green with the fail-closed capture

**Limits:**

- Lane rerun required for the review round 2 freeze; codex pass 2 after the record rewrite

**Next move:** Commit, push, lanes green -> rewrite the record (L16), then codex pass 2.

**Evidence:**

- /tmp/zcode-bash-4BSlaa/.../call_b87c2fc9017141ef9ab89bed-stdout.log (pass-1 findings)
- .agents/OBSERVED_DEBT.md
