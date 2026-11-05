# Current status

**Status revision:** 219

**Stage:** Security: PR 84 review round 2 (1/3) separation of duties and close-before-audit for uncertain resolution

**Summary:** resolve_uncertain_settlement refuses a resolver who is the requester (self_approval_refused, audited as approval_self_resolution_refused) after the operator-role check and before policy, status or close. The attempt is now closed first, and the resolution audit record is written only after the close succeeds, so a resolution fenced out by the worker leaves no audit. If the audit write fails after a successful close, the error says the close took effect and the audit was not written.

**Verified:**
- cep_settlement: 76 pass, including the requester refused with the claim open and no resolution record, a different operator still resolving, and a worker completing between the operator's checks and the close (refused, no settlement_uncertain_resolution record) for both resolutions
- The concurrent-resolver test still shows one close and one audit record

**Limits:**
- The interleaving test uses a doc-hidden hook (resolve_uncertain_settlement_with_hook) that runs after the checks and before the close
- A detached grandchild that survives the process-group kill could append after the operator attests NotDeclared; lease safety assumes the authority call is bounded
- CLI approve and resolve-uncertain build an Operator from --actor, the same local trust model as approve
- Operator resolution trusts the declaration reference the operator supplies; it is not checked against the external authority record

**Next:** CLI keep-decision proof, server wiring test, docs and PR body.
