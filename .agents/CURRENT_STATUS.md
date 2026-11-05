# Current status

**Status revision:** 221

**Stage:** Security: PR 84 review round 2 (3/3) server wiring test for a started SWE_SEED failure

**Summary:** A unit test in sea-forge-server drives submit_swe_seed_declaration end to end through the policy's own settlement_authorities command (no transport fake needed): held for approval, approved by a different operator, then a command that starts and exits nonzero. The claim stays Resuming and a second submit does not call the command again.

**Verified:**
- sea-forge-server lib delegation: new test passes
- Mutation M1 (ambiguous_after_call always false) by hand: the new test and the predicate unit test both fail; reverted

**Limits:**
- A detached grandchild that survives the process-group kill could append after the operator attests NotDeclared; lease safety assumes the authority call is bounded
- --declared <ref> is not checked against a ledger declaration
- A failure inside declare before the transport spawns is held for an operator, which is conservative
- The CLI builds an Operator from --actor, the same local trust model as approve
- The Horizon policy renderer and the sea-rs fixture are not tied by a test (follow-up)
- The server test covers a nonzero exit only; the timeout variant is covered at the CLI

**Next:** Independent re-review of PR 84.
