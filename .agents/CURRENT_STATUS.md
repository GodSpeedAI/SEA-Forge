# Current status

**Status revision:** 143

**Stage:** security-round4-review-threads-cleanup

**Summary:** Complete snapshot of round 4 after the 24.04 kernel-security gates went green and the PR #38 review threads were addressed. Round 4 implements pathname Unix-socket containment (SEC-06, ADR-014): probe-gated between a Landlock RESOLVE_UNIX ruleset (ABI >= 9) and, below ABI 9 on x86_64, a seccomp user-notification supervisor that brokers or refuses every trapped connect/sendto/sendmsg (sendmmsg refused) for the process that trapped, pinned by pidfd and validated with NOTIF_ID_VALID. Socket reach is an explicit WorkloadConfinement grant (Postgres socket dirs, the run's canonical MCP dir, the workload's own trees). Abstract AF_UNIX is brokered only in the workload's own network namespace and refused under a TCP grant. Federation export_bundle is on the reserved surface under the destination-authority rule; import/adopt stay denied. This revision also fixes review findings: a dangling iovec temporary on the brokered pathname sendmsg path (undefined behaviour), ensure_helper_reachable widening host directories (it now requests only other-x on directories and other-rx on the helper, repairs only the helper and its build directory, and refuses ancestors with the exact chmod the owner must run; the gates script performs that one-time grant visibly), and a non-canonical MCP socket grant that refused launches under a symlinked root. The E8/attribution proposal now refuses (never flags) bounded v1 comparisons, binds observer keys through a registry the ledger writer cannot alter, and defines a versioned cutover for pre-signature v1 records. Revision 143 removes a Python bytecode file committed by mistake with revision 142 and ignores __pycache__/ and *.pyc repository-wide.

**Verified:**

- run 39496384269 on ubuntu-24.04 (kernel 7.0.0-1017-azure, Landlock ABI 8, supervisor path): PASS sandbox-suite, PASS postgres-broker, PASS postgres-lifecycle; mutation controls M-S2 (supervisor allows all) and M-S4 (grants dropped) KILLED and restored PASS
- same run: unix_socket_reach 8/8 including granted connects from a child process and a non-main thread; workload_boundary isolated_network_layers_hold_independently passes (abstract refused under a TCP grant)
- all other PR #38 checks green on b44ce74; the live ladder failed twice on UI wait timeouts at different journeys (L5, then L-RECOV) on a slow runner and passed 11/11 on rerun
- root causes found on CI and fixed in this round: wrong AUDIT_ARCH_X86_64 (i386 value) left the filter a no-op; positive errno in replies; descriptors taken from the root workload instead of the trapping process
- rustls-webpki and webpki-roots bumped for RUSTSEC-2026-0382/0381; cargo deny advisories pass
- sea-forge-sandbox unit tests 11 pass locally; this host cannot run the isolation tests (inherited user-notify listener), so they pass vacuously here and the 24.04 job is the authority

**Limits:**

- Round 4 is not independently re-attacked yet (item D pending); nothing is SETTLED
- The ubuntu-22.04 job is vacuous for the socket gates (Landlock ABI 4: isolation UNAVAILABLE, NOT-EXECUTABLE); only 24.04 executes them
- The Landlock ABI 9 RESOLVE_UNIX branch is compiled but unexercised (CI 24.04 is ABI 8, this host ABI 7); M-S1 is unrunnable without ABI 9
- datagram_sendto_to_outside_canary_is_refused passes on file permissions alone and does not yet prove the supervisor refuses it
- A workload granted TCP ports cannot use its own abstract-socket IPC (refused, fail closed)
- Operators whose checkout sits under a non-traversable directory must grant o+x once; the launcher no longer does it for them
- Debug-only diagnostics remain behind SEA_FORGE_SOCKET_SUPERVISOR_DEBUG (helper self-probe thread, supervisor audit lines) plus the preflight notif_trap_probe; removal is an operator decision
- Federation import/adopt remain denied as unclassified pending an ingest-classification design
- conformance_sfwp admission overflow not reproduced in 140 targeted runs; conformance_m12 t12 flake not investigated; the casework live ladder has an intermittent UI-timeout flake
- The E8 refuse-on-bound policy and observer signing are proposals only (docs/security/realitytrace-e8-attribution-proposal.md), not implemented

**Next move:** Kernel-security and CI run for this commit; resolve the PR #38 review threads; merge on the operator's standing approval if every check is green. Then the independent re-attack (item D) and the datagram test tightening.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md
- docs/decisions/ADR-014-unix-socket-containment.md
- docs/security/realitytrace-e8-attribution-proposal.md
- scripts/security/round4-socket-gates.sh
- .agents/evidence/security-v2/round4-20261018/admission-race/INVESTIGATION.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
