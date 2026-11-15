# Current status

**Status revision:** 256

**Stage:** WP-19 S1 gate: results, claim ledger, evidence bundle

**Summary:** fde/integration: S1 proof run on a clean tree: 21 frozen tests, 12 PASS / 9 PARTIAL / 0 FAIL; three real program episodes (settled U, F, U), approved promotion, capability delta NO_GAIN; claim ledger and raw-record manifest committed under .agents/evidence/fde-field-case/s1-20261114T234200Z.

**Verified:**
- scripts/fde-proof.sh --slice s1 on a clean tree: 0 FAIL; fde-s1-verify.sh: anchors, manifest, replay bytes and classes agree

**Limits:**
- Stack ran on the DomainForge 3960326 build (SEA-Forge embeds 0.21.2); I-1 execution gate unwired; anchor local-only (OD-6); ep-3 NO_GAIN; AT-14/NT-03/NT-06 PARTIAL

**Next:** WP-20 contract; anchor the results digest under a new kind; re-execution by the verifier; mutation re-application

**Evidence:**
- .agents/reports/fde-convergence-2026-11-12/evidence/s1/wp-19-gate.md
- .agents/evidence/fde-field-case/s1-20261114T234200Z/claim-ledger.md
