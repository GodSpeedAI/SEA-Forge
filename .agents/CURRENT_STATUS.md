# Current status

**Status revision:** 220

**Stage:** Security: PR 84 review round 2 (2/3) CLI proof that a started SWE_SEED failure keeps the claim

**Summary:** A new m8 conformance test starts a SWE_SEED command that records each invocation and then fails (nonzero exit, and a timeout). resume exits 1 with settlement_authority_call_failed, the claim stays Resuming, and a second resume does not call the authority again. The resume.rs comment no longer says a lapsed lease settles the claim; only a found declaration or an operator resolution does.

**Verified:**
- sea-forge-cli conformance_m8_artifact: 7 pass
- Mutation M2 (if true || !call_started || unavailable) by hand: the new test fails on the claim-status assertion; reverted

**Limits:**
- A detached grandchild that survives the process-group kill could append after the operator attests NotDeclared; lease safety assumes the authority call is bounded
- --declared <ref> is not checked against a ledger declaration
- The CLI builds an Operator from --actor, the same local trust model as approve

**Next:** Server wiring test, docs and PR body.
