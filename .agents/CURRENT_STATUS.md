# Current status

**Status revision:** 141

**Stage:** security-round4-abstract-host-netns

**Summary:** Run 39494046927 on 24.04 confirms the revision-140 caller fix: postgres-lifecycle and postgres-broker pass and all eight unix_socket_reach tests pass, including the new child-process and thread connects. It also exposed a containment bypass that the never-trapping filter had hidden: with a TCP grant the workload shares the host network namespace, and the supervisor brokered abstract connects from its unrestricted thread, skipping the Landlock abstract scope, so workload_boundary's isolated_network_layers_hold_independently reached a host abstract listener. Abstract targets are now brokered only when no TCP port is granted (dedicated network namespace, by the same NetworkPosture normalization the launcher uses for --net) and refused otherwise.

**Verified:**

- run 39494046927 (24.04, kernel 7.0, Landlock ABI 8): PASS postgres-broker, PASS postgres-lifecycle, unix_socket_reach 8/8; FAIL workload_boundary isolated_network_layers_hold_independently (abstract connected under a TCP grant)
- cargo clippy -p sea-forge-sandbox --all-targets -D warnings clean; sandbox unit tests 11 pass

**Limits:**

- A workload granted TCP ports cannot use its own abstract-socket IPC (refused, fail closed). Unverified until the 24.04 kernel-security run for this commit passes; the datagram test still passes on file permissions alone. See revision 113 for the other round-4 open limits.

**Next move:** Kernel-security 24.04 run for this commit, including the mutation gate; then tighten the datagram test.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
