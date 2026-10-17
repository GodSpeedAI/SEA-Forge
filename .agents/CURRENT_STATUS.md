# Current status

**Status revision:** 109

**Stage:** security-round2-adversarial-gaps-locally-tested

**Summary:** Round 2 closes the five BYPASS findings of the independent adversarial review of 0b25e08 at the implementation level: isolated read allowlist, lifeline-bounded workload lifetime with orphan reaping before slot reissue, runtime execute_command isolated class, expired PostgreSQL session reconciliation, exposure on classified error payloads, issued-credential redaction, and an auditor-reconstructable attempt projection. All are LOCALLY TESTED with code-level mutation controls; none is independently re-verified or settled. Frozen v1 remains 6 PASS / 10 NOT-YET-EXECUTABLE.

**Verified:**

- Frozen v1 pinned verifier passes against af41ea2465e38c87dfe86471c33c6b54ea8564e1; tests/security-v1 unchanged.
- Adversarial review (context-isolated reviewer at 0b25e08): 38 attempts, 20 HELD, 9 PARTIAL, 5 BYPASS, 4 INCONCLUSIVE (.agents/evidence/security-v2/adversarial-20261017).
- Six code-level mutations each failed their targeted test, sources restored and hash-verified (.agents/evidence/security-v2/round2-20261017).
- Gitleaks continuation exception narrowed to exact recomputed digests; 13 canaries behave as specified (.agents/evidence/security-v2/gitleaks-control-20261017).

**Limits:**

- Round 2 is not independently re-verified; nothing is SETTLED.
- Uncovered: DB write sessions not revoked on later exposure; workspace/artifact/stdout export not exposure-gated; pathname UNIX sockets DAC-only; no seccomp (io_uring, nested userns); state tree created 0755/0644 (Landlock-only protection); busy orphaned PostgreSQL backends last until the next issuance.
- RealityTrace producer distinction and field-level difference exist only on unpushed sxr branch security/acceptance-evidence-20261017 (8c95489); evidence.recorded registration needs CEP-owned semantic decisions.
- Work is on branch security/enforcement-gaps-20261017 (worktree ~/projects/_wt/sec-gaps); merge to main and any push need operator intent.

**Next:** Operator decisions on candidate revisions and RealityTrace adoption; independent re-verification of round 2; then exposure-driven revocation of DB write sessions.

**Evidence:**

- .agents/reports/security-coverage-ledger-20261017.md
- .agents/evidence/security-v2/adversarial-20261017/
- .agents/evidence/security-v2/round2-20261017/
- .agents/evidence/security-v2/gitleaks-control-20261017/
- docs/decisions/ADR-013-workload-boundaries-and-lifecycle.md

**Spec:** .agents/specs/security-acceptance-v1.spec.md

**Ledger:** .agents/reports/security-coverage-ledger-20261017.md
