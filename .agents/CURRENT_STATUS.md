# Current status

**Status revision:** 218

**Stage:** Security: PR 84 review fixes (4/4) uncertain-claim resolution is policy-gated

**Summary:** resolve_uncertain_settlement now evaluates the policy's approval_resolution decision for the resolver before it reads or changes the attempt, so the CLI cannot complete or release an uncertain claim for an actor the policy does not allow. A refusal is audited as an authority_decision and leaves the claim open.

**Verified:**
- cep_settlement: an operator without an approval_resolution rule is refused, the claim stays open and no resolution record is written

**Limits:**
- No end-to-end server test drives submit_swe_seed_declaration through a transport; the ambiguous case is tested at the authority layer
- No independent review yet; PR 84 must not merge until one is obtained
- CLI approve takes the actor id from --actor and hard-codes the operator role; self-approval is enforced on that string, so the CLI is not an authenticated identity path
- Operator resolution trusts the declaration reference the operator supplies; it is not checked against the external authority record

**Next:** Independent review of PR 84; server end-to-end test; CLI identity fix; then SPEC-CR-003.
