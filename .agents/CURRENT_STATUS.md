# Current status

**Status revision:** 114

**Stage:** security-round4-federation-export

**Summary:** export_bundle joins the evaluable reserved surfaces under the operator-approved destination-authority rule: a rule matches only when its permitted_exposure covers every label of the exported runs' exposure union (round-3 already states that union in the authority request); the empty union exports under any allowing rule, labeled unions need explicit admission, import/adopt stay denied as unclassified. export_exposure pins both directions plus the noplan/traversal never-reach-authority cases.

**Verified:**

- just check all gates green on this tree; see revision 113 for the round-4 verification record

**Limits:**

- See revision 113 for the round-4 open limits; this entry records a per-commit increment only.

**Next move:** Operator clean-side socket gates, then the independent re-attack; see revision 113.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
