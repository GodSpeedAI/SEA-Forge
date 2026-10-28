# Current status

**Status revision:** 217

**Stage:** capability-harvest: pass-3 R2 ack-path rework — mode-granted reopen-by-path (the lifeline's proven pattern) instead of inherited-fd dup; pushing for final lane verification

**Summary:** Run-6's 24.04 fixture failure showed the inherited-ack-fd dup breaks inside the real launch chain (helper exit before the handshake byte, EOF at the launcher). The ack now uses exactly the mechanism the lifeline proves works on every lane: the launcher fchmods the ack WRITE end 0222, the helper reopens it by /proc/self/fd path as the subordinate identity, and the reopened descriptor is close-on-exec so the workload never inherits it. The pre_exec re-admits only the lifeline read end. The concurrent-inheritance leak (pass-3 finding 2) stays closed: the pipe is CLOEXEC at creation and only the pre_exec path unsets it in the intended child.

**Verified:**

- sandbox lib 31 green; syscall_filter, unix_socket_reach (real launch chain), workload_isolation, workload_boundary all green locally; clippy -D warnings clean; fmt clean

**Limits:**

- Final lane rerun (kernel 24.04 sandbox-suite + portability + ci) pending

**Next move:** Green lanes -> consolidated final report + handoff. Codex rate-limited; pass 3 ran on the independent Claude model (disclosed).

**Evidence:**

- /tmp/harvest-pass3-claude-review.md
- run 40602941845 amd64-green evidence
