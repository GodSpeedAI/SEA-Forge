# Current status

**Status revision:** 219

**Stage:** capability-harvest: launcher fixture captures the helper's stderr — next 24.04 run names the ack-path failure; pushing

**Summary:** The 24.04 sandbox-suite's fixture child fails at the ack wait and its stdio is null, so the helper's fail() reason was unreachable. The launcher fixture now captures the launch stderr to a file and the assertion prints it. The failure first appeared with the pass-3 ack rework (CLOEXEC-at-creation + pre_exec re-admit + 0222-reopen) — syscall_filter's identical chain passes on the same VM, so the fixture's difference (sh -c script with exec sleep, spawned from a libtest process) is the variable the capture will explain.

**Verified:**

- workload_boundary 4/4 green locally; clippy/fmt clean; the capture path derived identically at both sites

**Limits:**

- 24.04 rerun pending

**Next move:** Push; read the captured stderr; fix the named cause.

**Evidence:**

- run 40694441440 log
