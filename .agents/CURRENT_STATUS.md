# Current status

**Status revision:** 220

**Stage:** capability-harvest: review protocol complete (3 passes, C0/H0); 24.04 fixture ack-failure recorded as open non-gating lane defect; final report phase

**Summary:** Pass 3 (independent Claude model, disclosed; codex rate-limited): C0/H0/M2/L4. The two Mediums fixed (dedicated jail launch thread; CLOEXEC-at-creation ack/lifeline pipes with pre_exec re-admit). Pass-3's rework broke the 24.04 launcher fixture (silent chain death, empty helper stderr) — recorded as an open defect in the NON-REQUIRED kernel-security lane: it fails only there, only in the workload_boundary fixture; unix_socket_reach's identical handshake passes on the same VM, 22.04 passes, all local launch-chain suites pass, and the REQUIRED gates are green. The alternative (reverting R4's handshake) would restore the false-PidNamespace claim the reviewer reproduced — wrong trade. Remaining Lows (skip-accounting in two tests, stale comments, staging-reuse nuance) recorded.

**Verified:**

- Pass-3 fix locally: sandbox lib 31, syscall_filter/unix_socket_reach/workload_isolation/workload_boundary green; clippy/fmt clean
- Kernel lane: 22.04 green; 24.04 fails only the workload_boundary fixture (evidence retained in /tmp/sg24i)

**Limits:**

- 24.04 fixture ack-failure root cause open (silent chain death; capture in place); final lanes for fa7576b2/e7e998f5 lineage in flight

**Next move:** Final consolidated report; graft build; handoff. The 24.04 defect stays open and honestly reported.

**Evidence:**

- /tmp/harvest-pass3-claude-review.md
- /tmp/sg24i
- gh runs 40694441440 / 40732032144 / 40691041682
