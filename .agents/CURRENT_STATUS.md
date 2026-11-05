# Current status

**Status revision:** 215

**Stage:** Security: PR 84 review fixes (1/4) keep the claim lease when a started SWE_SEED call fails

**Summary:** Only a SWE_SEED command that could not be started is settlement_authority_unavailable and releases the claim. A started command that times out, exits nonzero, exceeds output limits or answers badly is settlement_authority_call_failed and keeps the lease, because the authority may already have appended a declaration.

**Verified:**
- cargo test -p sea-forge-settlement: transport tests prove spawn failure is unavailable and timeout, nonzero exit and invalid JSON are call_failed
- sea-forge-cli conformance_m8_artifact: 6 pass with the outage simulated by an unstartable command

**Limits:**
- No end-to-end server test drives submit_swe_seed_declaration through a transport; the ambiguous case is tested at the authority layer
- No independent review yet; PR 84 must not merge until one is obtained
- CLI approve takes the actor id from --actor and hard-codes the operator role; self-approval is enforced on that string, so the CLI is not an authenticated identity path
- Operator resolution trusts the declaration reference the operator supplies; it is not checked against the external authority record

**Next:** Remaining PR 84 review threads: resolution audit single-winner, policy-gated uncertain resolution, refused-resolver standing test; then independent review.
