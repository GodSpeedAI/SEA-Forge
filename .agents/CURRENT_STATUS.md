# Current status

**Status revision:** 251

**Stage:** WP-19 S1 gate: runner wiring and episode orchestration

**Summary:** fde/integration: fde-proof.sh records command, exit, status, duration, reason, repo commits and tool versions per test; fde-proof.d wrappers for AT-05 (both halves), AT-06, AT-07, AT-08, AT-14, AT-18, NT-02, NT-03, NT-03b, NT-04, NT-06, NT-09; fde-s1-run.sh runs the three program episodes through SEA-Forge, Cognate, RealityTrace and DomainForge into a scratch cell; fde-s1-verify.sh is the AT-18 verifier.

**Verified:**
- plumbing exercised end to end on debug cells (freeze, program.run, ingest, derive, compare, settle by another actor, propose, regression, approved promotion, subsequent episode, capability delta)

**Limits:**
- No full real run committed yet

**Next:** Full run, claim-ledger builder, evidence bundle

**Evidence:**
- scripts/fde-s1/
- scripts/fde-proof.d/
- scripts/fde-s1-run.sh
