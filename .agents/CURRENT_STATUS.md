# Current status

**Status revision:** 113

**Stage:** security-round4-sockets-and-authority

**Summary:** Round 4 implements pathname Unix-socket containment per ADR-014: probe-gated between a raw Landlock RESOLVE_UNIX ruleset (kernel ABI >= 9) and a seccomp user-notification supervisor on connect/sendto/sendmsg (x86_64 + x32 aliases) with grant-prefix validation, O_PATH inode checks and a post-allow recheck-and-kill compensating control. Socket reach is now an explicit WorkloadConfinement grant (socket_paths): the configured PostgreSQL socket directories, the run's MCP broker directory and the workload's own trees; readable trees no longer imply socket reach. The helper closes every inherited descriptor above stdio at startup. Federation export_bundle joins the reserved surface under the operator-approved destination-authority rule (a rule must permit every label of the exported runs' exposure union); import/adopt stay denied as unclassified. Gitleaks: round-1/2 evidence and casework T10/T09 matches are value-scoped to exact identifiers, so new values under those paths fail closed. ADR-014 records that this host kernel is Landlock ABI 7 (probed) and that a kernel install was offered and declined. All claims LOCALLY TESTED except the supervisor's end-to-end, which this agent harness structurally cannot run (see limits).

**Verified:**

- sea-forge-sandbox: 79 tests pass on this host; socket_policy unit tests cover the BPF trap set via an in-process interpreter (which caught an inverted arch-jump mutant before merge), sockaddr decoding for connect/sendto/sendmsg, abstract/passthrough families and unresolvable-path refusal
- Kernel facts probed, not recalled: Landlock ABI 7 on 6.18.54 WSL2 (kernel-probes-20261018.txt); LANDLOCK_ACCESS_FS_RESOLVE_UNIX is ABI 9 (mainline 7.1) and absent here; connect(2) succeeds on read-only bind mounts (mount-view gap); seccomp EBUSY on a second listener traced to has_duplicate_listener in kernel/seccomp.c
- This environment (whole WSL distro) runs under an inherited user-notify seccomp filter (/proc/1/status Seccomp: 2), so the supervisor's NEW_LISTENER install correctly fails closed here; every isolation test reports unavailable rather than degrading; scripts/security/round4-socket-gates.sh runs the real gates plus M-S2/M-S4 mutation controls from a clean process tree (operator execution pending)
- just security green including staged-secret canaries; the new value-scoped allowlists verified three ways: repo-config tree scan 0 in-scope findings, history scan 0 in-scope findings, and a new high-entropy value under a scoped path is still reported
- export_exposure: an admitting rule exports empty, direct, mixed and inherited-label unions; a rule admitting no label refuses every labeled union; noplan/traversal never reach authority
- Admission-race item B: 140 consecutive passes (60 idle + 60 loaded single-test, 20 loaded whole-binary) with instrumentation, no reproduction; full-suite loop continuing during gates; static analysis in round4-20261018/admission-race/INVESTIGATION.md

**Limits:**

- Round 4 is not independently re-attacked yet (item D pending); nothing is SETTLED
- The supervisor's end-to-end proof awaits the operator's clean-side run of scripts/security/round4-socket-gates.sh; this harness cannot install a second seccomp listener, by kernel design
- The Landlock ABI 9 branch is compiled but unexercised on this host (ABI 7); a kernel upgrade was offered and declined by the operator (2026-10-18), so the supervisor is the enforcement here
- Mutation controls M-S2/M-S4 run on the operator side with the gates script; M-S1 (RESOLVE_UNIX removal) is unrunnable without ABI 9
- Federation import/adopt remain denied as unclassified pending an ingest-classification design
- conformance_sfwp admission overflow: not reproduced in 140 targeted runs plus continuing full-suite loops; no fix until a failure is captured; conformance_m12 t12 flake not investigated (cause not shown)
- Work is on branch security/round4-20261018 (worktree ~/projects/_wt/security-round4-20261018); push, PR and merge need operator intent

**Next move:** Operator runs the clean-side socket gates; verify and record their logs. Independent context-isolated re-attack of round 3 (38 scenarios) plus round-4 changes. Remaining gates: just ci one crate at a time, frozen v1 runner with SEA_FORGE_SECURITY_MCP_COMMAND provisioned, gitleaks history, every mutation control. Then summarize and ask the operator before push/PR/merge.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/kernel-probes-20261018.txt
- .agents/evidence/security-v2/round4-20261018/DESIGN.md
- .agents/evidence/security-v2/round4-20261018/admission-race/INVESTIGATION.md
- docs/decisions/ADR-014-unix-socket-containment.md
- docs/security/realitytrace-e8-attribution-proposal.md
- scripts/security/round4-socket-gates.sh

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
