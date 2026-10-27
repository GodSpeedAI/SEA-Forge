# Current status

**Status revision:** 194

**Stage:** capability-harvest: round 9 verdicts — kernel-security fully green (22.04+24.04), safe-write lane green; breakaway test given a positive-control create; pushing round 10

**Summary:** Round 9: both socket-gate VMs pass (the round-8 22.04 failure does not reproduce with retained evidence), and the safe-write windows lane passes with the corrected pinned-anchor semantics — the release-pin rename publishes, hard links keep their bytes, planted junctions do not redirect. The one remaining red is the breakaway-refusal test: the escape IS refused (created=False) but Server 2025 returned error 123 rather than the documented 5, so the exact-code assertion both over-specified the kernel and could mask a malformed probe. The probe now performs a no-flag control create that must succeed from inside the job (attributing the refusal to the flag omission), records both error codes, and asserts only created-True-control + created-False-breakaway plus the existing survivor check.

**Verified:**

- gh run 40444187271: socket gates green on both VMs; 40444187212: safe-write step green (6/7 with the old test, now corrected)
- windows-target check + clippy -D warnings + fmt green on the rewrite

**Limits:**

- Round 10 must confirm the windows lane end-to-end green

**Next move:** Push round 10; on green, freeze and run the codex adversarial review.

**Evidence:**

- gh runs 40444187271 / 40444187212
- .agents/OBSERVED_DEBT.md
