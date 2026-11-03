# Current status

**Status revision:** 195

**Stage:** Security: server settlement path uses the pending, approval, claim and resume lifecycle

**Summary:** The server-owned SWE_SEED declaration path (delegation.rs) now uses the same lifecycle as the CLI. A claim left open by an interrupted call is settled first by cep::reconcile_operation_claims, which completes only on a declaration this case verified as authorised for that claim and otherwise fails it. A new attempt is gated only after that. A declare failure releases the claim when the authority was unavailable and fails it otherwise, so nothing is left stuck. The CLI resume uses the same reconciliation. An artifact transition that reuses an existing declaration verifies it first. External declarations are never authorised outright: policy that allows one without an approval-gated rule is refused, and a claimed external record is claimed only once approved.

**Verified:**
- cargo test -p sea-forge-server --lib: 180 pass, 0 fail (swe_seed_reconciliation 12 pass, including interrupted claim completes from its authorised declaration, interrupted claim without a declaration fails and is never redeclared, startup quarantine, approval for one run does not authorise another, writer refuses external without provenance)
- cargo test -p sea-forge-server --tests: all 20 integration targets exit 0 (conformance_m16: 19 pass, 2 ignored)
- cargo test -p sea-forge-authority -p sea-forge-artifact-ip -p sea-forge-settlement: 338 pass, 0 fail
- Note: a policy that allows an external settlement outright is already refused at policy load (existing test an_external_class_rule_that_is_a_plain_allow_is_refused_at_policy_load), so the gate-level refusal is defence in depth and has no separate test
- cargo fmt --all -- --check: clean

**Limits:**
- conformance_m8 CLI tests still fail (checkpoint d): the resume now needs a settlement approval step and the fixture has none
- race test not yet red-then-green (checkpoint e)
- adversarial review not done

**Next:** Checkpoint d: give the m8 fixture an external settlement rule that requires approval, a distinct operator approver identity with an approval_resolution rule, and finish both m8 CLI tests without loosening the requester's policy.
