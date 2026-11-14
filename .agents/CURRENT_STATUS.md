# Current status

**Status revision:** 240

**Stage:** FDE integration branch: S1 pre-registration, anchor and proof-runner skeleton (WP-09)

**Summary:** fde/integration: froze G's S1 subset under .agents/preregistrations/fde-s1/ (21 tests with oracle verbatim from G, falsifiers and commands; three program episodes with predicted comparison classes; per-test predictions, 12 PASS and 9 PARTIAL; READINGS.md for ambiguous criteria; MANIFEST.sha256). Pre-registration digest 4f9e81677a36adbc3ed5cb015803ed7c529b0fa8233a33b52d2309f6968389ac (sha256 of the manifest), anchored as tag fde-anchor/prereg/fde-s1. Added scripts/fde-anchor.sh (manifest, publish, verify) and the scripts/fde-proof.sh skeleton (verifies the anchor first, dispatches scripts/fde-proof.d/<ID>.sh, records NOT_RUN for unwired tests); AT-01 is wired to check-fde-sop-model.sh.

**Verified:**
- scripts/tests/fde-anchor-proof.sh passes (tamper names the file, double publish refused, unwired test is NOT_RUN and exits non-zero, AT-01 passes)
- scripts/fde-anchor.sh verify prereg/fde-s1 --remote exits 0 after publication
- scripts/check-fde-contract-digests.sh passes
- scripts/check-agent-context.sh passes

**Limits:**
- RD-26: pusher and verifier are one principal on a shared OS user; the anchor proves not changed since publication, not independence; anchoring rows stay PARTIAL until OD-6
- The tag is unsigned and the server push time is not read from GitHub
- fde-proof.sh --replay and --mutate are stubs; only AT-01 is wired, every other S1 test is NOT_RUN until its work package lands
- Mutation patches are not part of this pre-registration; adding them is a new version fde-s1-v2
- Integration branch only: not merged to main, no PR

**Next:** WP-14 case keeper (freeze, anchor checks, FREEZE_KEY_EXISTS); wire fde-proof.d/<ID>.sh as each owning work package lands.

**Evidence:**
- .agents/preregistrations/fde-s1/PREREGISTRATION.md
- .agents/preregistrations/fde-s1/MANIFEST.sha256
- scripts/fde-anchor.sh
- scripts/fde-proof.sh
- scripts/tests/fde-anchor-proof.sh
