# Current status

**Status revision:** 215

**Stage:** capability-harvest: R4 test reports BLOCKED on unreachable-helper hosts (22.04 layout); pushing

**Summary:** The 22.04 gate's R4 failure is a host-layout precondition: isolated_command's helper-reachability probe refuses a non-other-x /home/runner (every host identity is unmapped inside the namespace, so 'other' bits decide). The test now reports BLOCKED for that precondition class and asserts the full setup-failure path only where the chain is reachable (the 24.04 lane, which passed).

**Verified:**

- sandbox lib 31 green; fmt clean

**Limits:**

- 22.04 lane rerun pending

**Next move:** Push; lanes green -> codex pass 3.

**Evidence:**

- run 40616167976 22.04 artifact
