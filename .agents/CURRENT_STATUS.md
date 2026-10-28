# Current status

**Status revision:** 216

**Stage:** capability-harvest: codex pass-3 (independent Claude model) findings fixed — dedicated jail launch thread, CLOEXEC-at-creation ack/lifeline pipes with pre_exec re-admit, scoped unshare override; pushing final lanes

**Summary:** Pass 3 (independent reviewer, different model family; codex rate-limited): Critical 0 High 0 Medium 2 Low 4. Fixed: (1) the interactive jail launch now runs on a dedicated OS thread nested in the blocking closure — pooled-thread Landlock poisoning (R3 cleanup denial + H4 leaf-creation failure) is structural gone; (2) the ack write end and lifeline read end are CLOEXEC at creation and re-admitted in the forked child via pre_exec — no concurrently spawned process (a jail workload among them) can inherit and forge the setup acknowledgement; (3) the R4 test's unshare override is mutex-scoped and removed after the test (the 22.04 gate caught the leak in the parallel real-chain suites); (4) stale reopen comments corrected. Disputes held: H3/H5 trusted-host contract, R7/R8 tally-vs-pass design.

**Verified:**

- sandbox lib 31 green; syscall_filter 1 + unix_socket_reach 8 green through the new pre_exec path; agent 3 suites green; clippy -D warnings across touched crates; fmt clean; no-async-kernel EXIT=0
- kernel-security 22.04+24.04 green at c42b78ab (env-leak lineage)

**Limits:**

- Final lanes for 034bbed8..c42b78ab + the pass-3 fix commit pending

**Next move:** Push; lanes green -> consolidated final report + handoff (pass-3 result: pass with only Low findings remaining).

**Evidence:**

- /tmp/harvest-pass3-claude-review.md
