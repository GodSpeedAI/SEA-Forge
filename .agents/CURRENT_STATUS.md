# Current status

**Status revision:** 137

**Stage:** security-round4-audit-arch-fix

**Summary:** Root cause of the 24.04 bypass: AUDIT_ARCH_X86_64 was 0x4000_0003 (the i386 value; x86_64 is 0xc000_003e), so the supervisor filter's arch check never matched a native process and every syscall fell through to ALLOW. The unit test fed the same wrong constant to its own interpreter and passed. The constant is corrected and the test now also pins the literal kernel values.

**Verified:**

- run 39479954703: helper self-probe connect returned ENOENT (untrapped) on 24.04; the standalone probe with the correct arch value trapped on the same kernel
- the 22.04 job is vacuous for the socket gates (Landlock ABI 4, workload isolation UNAVAILABLE, NOT-EXECUTABLE); only 24.04 executes them

**Limits:**

- The fix is unverified until the 24.04 kernel-security run for this commit passes; revision 113 lists the other round-4 open limits.

**Next move:** Kernel-security 24.04 run for this commit; then tighten the datagram test, which passes on file permissions alone.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
