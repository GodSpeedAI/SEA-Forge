# Current status

**Status revision:** 216

**Stage:** Security: PR 84 review fixes (2/4) refused approval resolvers leave the approval pending

**Summary:** The gateway and self-approval refusal tests now read the approval standing and assert it stays Pending, and a new test covers the policy-only refusal of a service actor that is not the requester.

**Verified:**
- horizon_gateway_policy: 10 pass; mutation writing a resolution before either refusal fails the strengthened tests

**Limits:**
- No end-to-end server test drives submit_swe_seed_declaration through a transport; the ambiguous case is tested at the authority layer
- No independent review yet; PR 84 must not merge until one is obtained
- CLI approve takes the actor id from --actor and hard-codes the operator role; self-approval is enforced on that string, so the CLI is not an authenticated identity path
- Operator resolution trusts the declaration reference the operator supplies; it is not checked against the external authority record

**Next:** Remaining PR 84 review threads: resolution audit single-winner, policy-gated uncertain resolution; then independent review.
