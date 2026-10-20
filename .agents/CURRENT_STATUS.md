# Current status

**Status revision:** 127

**Stage:** security-round4-reply-sign-fix

**Summary:** The live bypass is identified as a reply-protocol defect: seccomp_notif_resp.error must carry the NEGATED errno, and the broker was replying positive EACCES - a value the kernel does not treat as a denial, so denied connects completed as fake successes (return value 13) exactly matching the observed outside-canary connections. All denials now reply -errno. The launcher forwards SEA_FORGE_SOCKET_SUPERVISOR_DEBUG explicitly through the cleared environment (the audit printed zero lines because env_clear stripped it), and mutation controls are skipped unless the baseline suite passes, since a failing baseline cannot kill a mutant (the earlier KILLED verdicts on 24.04 are retracted as noise).

**Verified:**

- just lint clean; sandbox unit tests 11 pass; see revision 113 for the round-4 verification record

**Limits:**

- See revision 113 for the round-4 open limits; this entry records a per-commit increment only.

**Next move:** Kernel-security run for this commit; see revision 113.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
