# Current status

**Status revision:** 112

**Stage:** security-round3-integrating

**Summary:** Round 3 closes live authorization gaps and hardens isolation at the implementation level: a trusted-server PostgreSQL session supervisor ends expired sessions on live connections without a later issuance or the launcher; per-role in-DB timeouts are removed because a workload can SET them away; a process-wide exposure gate revokes issued DB credentials, across episodes and delegated contexts, before classified plaintext moves; the state tree is owner-only by DAC independent of Landlock; a seccomp filter refuses io_uring and namespace creation (including x32 numbers); pre-commit scans the staged index and four over-broad gitleaks allowlists are narrowed; federation exports state their runs' exposure labels; the owner-accepted evidence.recorded record kind is registered in RealityTrace (main 42879a8, pushed) and SEA-Forge pins it, ingesting proofs under the frozen security-acceptance producer. All LOCALLY TESTED with mutation controls; none independently re-verified or settled. Frozen v1 remains 6 PASS / 10 NOT-YET-EXECUTABLE. The branch also carries the operator's staged main-worktree changes (Explore agent definition, two editor workspace folders) and status revisions 104-108. Merged with origin/main (#31 CodeRabbit review-only, status record kept as revision 111) for integration through a squash-merge pull request.

**Verified:**

- Full gates on the round-3 tree: fmt, lint, typecheck, security (deny, gitleaks history, staged canaries), no-async-kernel, proof; all 24 crates 1510 passed / 0 failed (.agents/evidence/security-v2/round3-20261017/gates).
- Frozen v1 runner with MCP provisioned: 6 PASS / 10 NOT-YET-EXECUTABLE; tests/security-v1 identical to sea-forge-security-v1-2026-10-16.
- 16 code-level mutations (M1-M15 plus the evidence.recorded schema) each failed their targeted test; sources restored and hash-verified.
- Staged-secret canaries: 13 expectations met after the allowlist fixes; the original config admitted new secrets in four allowlisted path classes (staged-canaries-before).
- Defects found and fixed this round: fchmodat(AT_SYMLINK_NOFOLLOW) unsupported so files stayed 0644; bypassable per-role timeouts; scope_admits revoking a session's own read label.
- RealityTrace 42879a8: fmt, clippy -D warnings, 209 tests pass; SEA-Forge realitytrace suite passes against it, including registered-row reconciliation and producer binding (mutation: SweSeed producer fails 5 of 6 sec12 tests).
- Git hooks no longer hard-fail when devbox cannot reach its package cache: .githooks/run-recipe probes devbox with a time limit, falls back to the same just recipe on the host toolchain with a stderr notice, and SEA_FORGE_HOOK_TOOLCHAIN=devbox restores strict pinning (hook-runner.txt).
- PR #32 CI round 1 failures fixed: isolation availability now probes a mapped user namespace (GitHub Ubuntu runners refuse them, so isolation tests report NOT-EXECUTABLE instead of failing), test jobs fetch the protected frozen v1 tag the SEC-12 auditor resolves, and Linux-only code (delegation inputs, the isolated M16 tests and their PostgreSQL support) is compiled only on Linux, so macOS builds.
- Workspace effect observation now works on all Unix: macOS had no observer (procfs-only), so it could never witness an effect and SEC-12 (including the frozen v1 positive control) could not settle there; other Unix now walks openat with O_NOFOLLOW, and the observer tests run on any Unix.
- Pinning a workspace now follows only host-configured symlinks in its path prefix (root-owned, in a root-only directory, e.g. macOS /var -> /private/var); user-owned links are still refused and nothing inside the workspace is followed (mutation: trusting every link lets a user alias through).
- A server killed between a ledger entry write and its derived mmr.json no longer refuses to start: startup rebuilds every ledger's MMR from its entries and quarantines a torn tail, under the stream lock, before any verify (the F-04 repair existed but no production path called it; found by the Go live kill-mid-commit test; mutation: removing the startup call fails both crash_recovery_startup tests).
- Throwaway-repo test scripts no longer act on the hook's repository: run from pre-push with GIT_DIR exported, the staged-secret canaries had set core.bare, a canary identity and commit.gpgsign=false in the shared config and moved the branch to a seed commit (push rejected, nothing reached the remote, operator repaired locally). Both scripts now unset git's local env vars first; a sentinel-repo control reproduces the old corruption and shows none after the fix (hook-env-escape.txt).

**Limits:**

- Round 3 is not independently re-verified; nothing is SETTLED.
- Open: pathname UNIX sockets outside declared trees are reachable from isolated workloads (crates/sea-forge-sandbox/tests/unix_socket_reach.rs, ignored, fails when run); excluded from the guarantee.
- Federation export/import/adopt are denied as unclassified by the authority engine whatever the policy (export_bundle absent from the reserved-surface list); export now carries exposure labels for when it is enabled.
- A depth-1 CI scan likely reports 22 generic-api-key matches in committed round-1/2 evidence that pass history scans only through commit-pinned exceptions; not widened (outside authorization).
- A new generic-api-key match under casework evidence paths is still admitted; credential-specific rules are not.
- Work is on branch security/enforcement-gaps-20261017 (worktree ~/projects/_wt/sec-gaps); merge to main and any push need operator intent.
- RealityTrace E8 field-level differences are bounded at 256 entries; ledger observer attribution is not a signature (ADR-012 obligation 4).
- On GitHub Ubuntu runners, unprivileged mapped user namespaces are refused, so workload-isolation tests are NOT-EXECUTABLE in CI; they execute on hosts that allow them (this WSL host, with SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION=1).
- Pre-existing intermittent server tests seen at the gated push, not introduced here: conformance_sfwp request_admission_refuses_overflow_and_cancels_waiters_without_effects ('pre-admission timeout reached provider', 5 vs 4) fails 1/13 on this branch and 1/40 on unmodified origin/main c9f8b4e; a fifth provider connection while four permits are held is an open admission question for its own investigation. conformance_m12 t12 failed once under full-suite load and passes alone.

**Next:** Independent context-isolated re-attack of round 3 (all 38 prior scenarios plus new attacks); operator decisions on pathname-socket containment, the CI shallow-scan evidence matches and enabling federation export; then governed integration prep.

**Evidence:**

- .agents/evidence/security-v2/round3-20261017/README.md
- .agents/evidence/security-v2/retention/round3-20261018.json
- docs/security/realitytrace-evidence-recorded-proposal.md
- docs/decisions/ADR-012-realitytrace-security-evidence.md
- .agents/evidence/security-v2/round3-20261017/sxr-gates/

**Spec:** .agents/specs/security-acceptance-v1.spec.md

**Ledger:** .agents/reports/security-coverage-ledger-20261017.md
