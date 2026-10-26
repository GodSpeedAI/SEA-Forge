# Current status

**Status revision:** 184

**Stage:** capability-harvest: all six findings implemented on feat/hardening-capability-harvest; lifecycle containment surfaced on ExecutionResult; ready for CI lanes and independent review

**Summary:** F15 skip integrity + cgroup preflight; F05 exclusive pinned dirs with anchored ops and per-use revalidation; F08 cgroup-v2 containment (Linux), verified job membership (Windows), census+typed CagePersists (macOS); F32 staged-dir publishes; F33 edge pipe adapter with unified-graph gate; F17 host-observed arrival proof pair; domainforge-core 0.21.2; ExecutionResult.lifecycle_containment (CgroupLeaf/JobObject/ProcessGroup/PidNamespace/ExternalSandbox/SandboxCagePersists/NoProcess) populated truthfully per backend and flowing into evidence, traces, and settlement.

**Verified:**

- just check EXIT=0 on the full worktree before W3b; W3b gates green: sandbox/core/cli/settlement/runtime/case-runner/server clippy+tests, windows-msvc and apple-darwin checks, no-async-kernel, fmt
- 8 commits landed through pre-commit (status coupling, fmt --all, gitleaks clean)
- Unavailable->ProcessGroup containment assertion ran locally; Active/Windows/macOS assertions are placed for their CI lanes

**Limits:**

- cgroup Active-path, windows JobObject, and macOS cage-persists assertions await their CI lanes
- F17 denial proof awaits the openshell-live KVM dispatch
- sea-forge-cli lifecycle post_start_internal_error_is_run_correlated_and_traced fails on pristine HEAD on this WSL2 host (stash-verified; likely host-specific; main CI is green)
- F05 NTFS/junction and anchored-rename tests await the portability windows lane

**Next move:** Commit W3b, push feat/hardening-capability-harvest, open the PR for ci.yml+kernel-security lanes, dispatch openshell-live.yml, then footprint and the independent codex review.

**Evidence:**

- .tmp/harvest/
- .agents/OBSERVED_DEBT.md
