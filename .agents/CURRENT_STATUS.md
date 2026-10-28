# Current status

**Status revision:** 211

**Stage:** capability-harvest: pass-2 lane failures fixed — ack fd inherited (not reopened) across the unshare chain; junction test de-doubled

**Summary:** Lane rerun caught two integration defects: (1) the ack write end was reopened by path inside the helper, which fails EACCES after --setuid drops to the subordinate identity — the fd is inherited across the chain (not CLOEXEC), so the helper now marks it CLOEXEC, dups it, writes the byte, and never resolves a launcher-owned path as the workload uid; the socket-policy/isolated suites that launch through the real chain pass again. (2) R10's mid-pin contained write collided with the test's own held pin (WriteShareHeld) — the write now runs after the test's pin is released with the junction still planted, and the duplicate trailing block is removed.

**Verified:**

- sandbox lib 31 green; syscall_filter + unix_socket_reach (real launch chain) green; windows-target clippy green; fmt clean

**Limits:**

- Lanes rerun pending

**Next move:** Commit, push, lanes; codex pass 3 after green.

**Evidence:**

- gh runs 40605628123 / 40605628078 logs
