# Current status

**Status revision:** 192

**Stage:** capability-harvest: round-7 root causes fixed — cross-test sweep kills, jail teardown on the restricted thread, cgroup.procs fd leak; pushing round 8

**Summary:** The jail 'network denial unexpectedly succeeded' was NOT a security regression: the process-wide destroy sweep killed sibling tests' live children mid-run (empty outcomes misread as Completed), which also explains the runtime env/timeout failures. Fixes: the destroy sweep now skips live leases (orphan-scoped; panicked runs still adopted); jail spawning split into spawn_jailed (Landlock-restricted thread) and finish_jailed (unrestricted teardown — the restricted domain was denying the jail's own cgroup.kill/rmdir, proven by log asymmetry); a reaped-but-signaled child is announced on stderr instead of silently reading Completed; the cgroup.procs fd gets FD_CLOEXEC (it previously leaked a write handle to the leaf into every workload); conformance deny-assert messages distinguish child-death from a denial break. Windows: the release-pin rename with FileId verification implemented exactly per design (pin brackets validate->write; release->rename; re-pin+full validation; published FileId must equal the captured temp id), with the two 32-under-pin forms added to the spike verdict.

**Verified:**

- Local: 23 sandbox test-targets green, runtime lib 5/5, clippy -D warnings host+windows clean, fmt clean, workspace check clean

**Limits:**

- Round-8 lanes must confirm: windows pinned-replace green; the active-cgroup lane green across runtime (5), conformance_m1 (19, incl. the network-denial zero-accepted-connections assertion), setsid_lifecycle (9, no jail EACCES degrades)

**Next move:** Push round 8; on green, freeze and run the codex adversarial review.

**Evidence:**

- gh runs 40429484729 / 40429484788
- .agents/OBSERVED_DEBT.md
