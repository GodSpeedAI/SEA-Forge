# Current status

**Status revision:** 189

**Stage:** capability-harvest: round-4 closures committed — pushing for the convergence round

**Summary:** cgroup: an empty leaf whose rmdir the kernel denies (6.17-azure delegation propagation limits; parent probe mode 755 uid 1001) now degrades to a logged destroy-sweep record instead of failing settled runs — non-empty leaves still error. macOS: the cage helper's parent holds 1.5s post-fork so the 100ms census observes the detached child (round-3 confirmed the census never sampled before reparenting). Windows: the final rename uses the years-proven SetFileInformationByHandle(FileRenameInfo) absolute-target call; three RootDirectory-relative/NT-absolute forms are recorded as INVALID_PARAMETER-rejected platform verdicts. F17: the allow-control is now an honest BLOCKED scenario (SEA_FORGE_REQUIRE_EGRESS_CONTROL flips it to a lane failure) with the full guest dump and the three-round upstream-requirement chain; the deny variant keeps its narrowed non-attribution claim.

**Verified:**

- Full local sweep green: sandbox all targets, case-runner non-ignored, clippy -D warnings host + windows-msvc + aarch64-apple-darwin, fmt
- graft build run; diff reviewed; 6 files changed +313/-135

**Limits:**

- All four closures await their CI lanes (round 4)

**Next move:** Push, re-dispatch openshell-live, collect round 4; if green, freeze for the codex adversarial review and fill the record.

**Evidence:**

- .agents/OBSERVED_DEBT.md
- gh runs 40400357003 / 40400356837 / 40400369301
