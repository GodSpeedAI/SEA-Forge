# Current status

**Status revision:** 147

**Stage:** branch-reconciliation-pr-required

**Summary:** Reconciled useful topic work into local main and verified deliberate replacements. Integrated rusqlite, serde_json/rustix, rand, serde-saphyr and jsonschema upgrades with API adaptations; retained exact DomainForge 0.21.0 and retired Tauri/legacy frontend/YAML code. Four obsolete local branches and three extra worktrees removed with verified private full-history bundle retention. Full local CI passed (1534 tests, zero failures), P1-P4b passed, and independent Luna frontend/dependency/MSRV audits found no missing useful code or violated replacements. The normal pre-push gates passed again, but GitHub rejected direct main publication because a PR and two status checks are required; atomicity kept every remote ref intact. A temporary reconciliation PR is next; original workspace edit and handoff prompt remain untouched.

**Verified:**

- run 39496384269 on ubuntu-24.04 (kernel 7.0.0-1017-azure, Landlock ABI 8, supervisor path): PASS sandbox-suite, PASS postgres-broker, PASS postgres-lifecycle; mutation controls M-S2 (supervisor allows all) and M-S4 (grants dropped) KILLED and restored PASS
- same run: unix_socket_reach 8/8 including granted connects from a child process and a non-main thread; workload_boundary isolated_network_layers_hold_independently passes (abstract refused under a TCP grant)
- all other PR #38 checks green on b44ce74; the live ladder failed twice on UI wait timeouts at different journeys (L5, then L-RECOV) on a slow runner and passed 11/11 on rerun
- root causes found on CI and fixed in this round: wrong AUDIT_ARCH_X86_64 (i386 value) left the filter a no-op; positive errno in replies; descriptors taken from the root workload instead of the trapping process
- rustls-webpki and webpki-roots bumped for RUSTSEC-2026-0382/0381; cargo deny advisories pass
- sea-forge-sandbox unit tests 11 pass locally; this host cannot run the isolation tests (inherited user-notify listener), so they pass vacuously here and the 24.04 job is the authority
- run 39515574297 on 24.04 for 6fc3997/9f00a38: PASS sandbox-suite, postgres-broker, postgres-lifecycle; M-S2 and M-S4 KILLED; the gates script logged its explicit o+x grant on /home/runner
- the casework L-RECOV 'Timed out waiting for panel closed' failure also occurs on main's own scheduled run 39389771329 (257b622), so it predates this PR
- gates-script mode grant and restore exercised locally on a scratch tree: 700 -> 701 during the run, 700 again after a failing exit
- Branch audit: round-3 and round-4 trees match their squashes; CodeRabbit patch equivalent; archive ancestor of round 3. Frontend has 104 equivalent patches in PR #11 source history and seven superseded implementation/docs/license commits. Full-history Git bundle verified; one local branch and one registered worktree remain.
- RUSTC_WRAPPER='' just ci: all gates green; 174 suites, 1534 passed, 0 failed, 23 explicitly ignored. just proof: P1-P4b passed. just secrets-staged, git diff --check and graft build passed. Luna audit found no new MSRV >1.92.0 dependency.

**Limits:**

- Round 4 is not independently re-attacked yet (item D pending); nothing is SETTLED
- The ubuntu-22.04 job is vacuous for the socket gates (Landlock ABI 4: isolation UNAVAILABLE, NOT-EXECUTABLE); only 24.04 executes them
- The Landlock ABI 9 RESOLVE_UNIX branch is compiled but unexercised (CI 24.04 is ABI 8, this host ABI 7); M-S1 is unrunnable without ABI 9
- datagram_sendto_to_outside_canary_is_refused passes on file permissions alone and does not yet prove the supervisor refuses it
- A workload granted TCP ports cannot use its own abstract-socket IPC (refused, fail closed)
- Operators whose checkout sits under a non-traversable directory must grant o+x once; the launcher no longer does it for them
- Debug-only diagnostics remain behind SEA_FORGE_SOCKET_SUPERVISOR_DEBUG (helper self-probe thread, supervisor audit lines) plus the preflight notif_trap_probe; removal is an operator decision
- Federation import/adopt remain denied as unclassified pending an ingest-classification design
- conformance_sfwp admission overflow not reproduced in 140 targeted runs; conformance_m12 t12 flake not investigated; the casework live ladder's L-RECOV journey intermittently times out on 'panel closed', on main as well as this branch (not a required check)
- The E8 refuse-on-bound policy and observer signing are proposals only (docs/security/realitytrace-e8-attribution-proposal.md), not implemented
- GitHub branch protection requires a PR and status checks; direct push was rejected without mutations. macOS verification runs in GitHub CI, not on this Linux host.

**Next move:** Publish a temporary reconciliation branch, open the required PR, await required checks and merge normally; then remove unchanged obsolete topic branches and align main after verifying squash equivalence.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md
- docs/decisions/ADR-014-unix-socket-containment.md
- docs/security/realitytrace-e8-attribution-proposal.md
- scripts/security/round4-socket-gates.sh
- .agents/evidence/security-v2/round4-20261018/admission-race/INVESTIGATION.md
- .agents/reports/branch-reconciliation-20261021.md
- .agents/evidence/branch-reconciliation-20261021/verification.json

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
