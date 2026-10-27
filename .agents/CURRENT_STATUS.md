# Current status

**Status revision:** 198

**Stage:** capability-harvest: FULL SURFACE GREEN at fcea8c66 (portability windows+macos, CI gate, kernel-security 22.04); the 24.04 guard race fixed; codex pass 1 running

**Summary:** Round 13 delivered the first simultaneously-green surface: portability windows AND macos success, CI gate success, KVM proof standing green. The kernel-security 24.04 red was the guard race W1 recorded in OBSERVED_DEBT, now with its precise trigger evidenced by the de-collided artifact: the guard read REQUIRE=1 inside required_lane's transient forced window plus a mid-run tally (the four Active scenarios honestly BLOCK on that VM - no delegation there - and only the unlocked read turned that into a failure). The guard now takes env_lock, serializing against every env-forcing test in the file. The codex adversarial review (pass 1, read-only) is running against the frozen fcea8c66 diff.

**Verified:**

- gh runs 40507892357 (portability success), 40507892479 (CI success), 40406886571/40400369301-line KVM green; setsid suite 9/9 locally after the guard fix; fmt clean

**Limits:**

- kernel-security must re-run green on the guard fix; codex pass 1 findings pending

**Next move:** Push the guard fix, verify kernel-security green, process the codex findings, then record/report/handoff.

**Evidence:**

- gh run 40507892410 artifact (ubuntu-24.04 de-collided evidence)
