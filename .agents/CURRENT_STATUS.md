# Current status

**Status revision:** 217

**Stage:** Security: PR 84 review fixes (3/4) resolution audit record is single-winner per attempt

**Summary:** The uncertain-resolution audit record is now written create-once under a per-attempt key. Of several operators racing past the status check only the first record exists, an identical retry replays it, and a conflicting resolution is refused, so the audit log never holds a resolution that did not take effect.

**Verified:**
- cep_settlement concurrent_operator_resolutions now counts audit records: exactly 1; mutation back to a plain commit fails it

**Limits:**
- No end-to-end server test drives submit_swe_seed_declaration through a transport; the ambiguous case is tested at the authority layer
- No independent review yet; PR 84 must not merge until one is obtained
- CLI approve takes the actor id from --actor and hard-codes the operator role; self-approval is enforced on that string, so the CLI is not an authenticated identity path
- Operator resolution trusts the declaration reference the operator supplies; it is not checked against the external authority record

**Next:** Remaining PR 84 review thread: policy-gated uncertain resolution; then independent review.
