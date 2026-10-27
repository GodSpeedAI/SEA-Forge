# Current status

**Status revision:** 210

**Stage:** capability-harvest: codex pass-2 fixes landed (R1-R10); pushing for lanes

**Summary:** R1+R9: content verification reads through the retained staging fd (RDWR staged temp, pread bounded at len+1, no destination reopen) — kills the symlink-follow/FIFO-block/unbounded-read variants and the mode-0200 regression; R2 fixed not disputed (hierarchically_populated via cgroup.events; populated leaf = Survivors, not hygiene); R3: UnadoptedInteractiveJailChild Drop + guards armed before fallible stream adoption; R4 confirmed and fixed (trusted ack handshake: helper writes 0x01 after all restrictions are in force; no ack = workload_isolation_setup_failed with no containment claim; test-only unshare override regression); R5 fixed with both controls proven (negated-if bug: status captured in the else of a non-negated if; sabotage shim FAILS); R6 implemented (root chown deleted; test tree migrates into the delegated subtree); R7/R8 disputes recorded; R10 junction test now exercises redirection.

**Verified:**

- sandbox lib 31 green incl. 3 R4 tests + R9 regression; agent suite green; cross-targets green; no-async-kernel both controls; fmt clean; status-check pass

**Limits:**

- Lane rerun for the pass-2 commit pending; codex pass 3 after

**Next move:** Commit, push, lanes, codex pass 3 (final).

**Evidence:**

- /tmp/harvest-pass2-review.md
- .agents/OBSERVED_DEBT.md
