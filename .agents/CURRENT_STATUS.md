# Current status

**Status revision:** 218

**Stage:** capability-harvest: pass-3 R2 corrected — pre_exec re-admits BOTH pipe ends (lifeline read + ack write, the latter enabled by the 0222 mode grant); pushing for the final green

**Summary:** The syscall_filter ENOENT was self-inflicted by the previous edit: the pre_exec re-admit list was cut to the lifeline fd only, so the ack write end closed at the first exec and the helper's fd number was invalid. The pre_exec now clears CLOEXEC on both fds in the intended child; the helper reopens the ack end by path under the 0222 grant. All launch-chain suites green locally.

**Verified:**

- sandbox lib 31 green; syscall_filter 1, unix_socket_reach 8, workload_isolation 3, workload_boundary 4 green; clippy -D warnings clean; fmt clean

**Limits:**

- Final lane rerun pending

**Next move:** Green lanes -> consolidated final report + handoff.

**Evidence:**

- run 40691041682 24.04 artifact
