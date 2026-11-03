# Current status

**Status revision:** 208

**Stage:** Security: approved request hash binds the declared content (SWE_SEED and artifact resume)

**Summary:** The approved request hash now binds the declaration content, not only the gated ingress. Both callers build the declaration request before the approval gate, hash it with hash_canonical (settlement_declaration_content_sha256, authorisation fields cleared), and pass that as declared_request_sha256. The claim compares it with the approved hash. A changed field is refused as content_mismatch and nothing is declared. The appended declaration must echo the request, or the claim fails as content_mismatch with nothing appended.

**Verified:**
- Red-to-green: a claim whose one declared field (criteria_ref) changed after approval is refused as content_mismatch, the record is Failed, and no declaration is made (cep_settlement a_claim_whose_one_declared_field_changed_after_approval_is_refused_as_content_mismatch)
- cargo test: sea-forge-authority 273, sea-forge-core 100, sea-forge-capability 40, sea-forge-thoth 48, sea-forge-cli 89, sea-forge-server 558; all passed, 0 failed
- sea-forge-server doctest still fails to link sea_forge_agent (agent_probe.rs, pre-existing, not touched on this branch)
- cargo clippy --all-targets -- -D warnings: exit 0; cargo fmt --all -- --check: clean

**Limits:**
- Bound into the approved hash: settlement_ref, run_id, case_id, plan_item_id, claim_manifest_sha256, criteria_ref, criteria_sha256, criteria_record_hash, criteria_declared_at, execution_started_at, job_contract_ref, origin_refs, verifier_ref, verifier_sha256, acting_entity_id, requested_strength, declarer, variation_tags, disruption_tags, orchestration_burden, source_evidence_refs, authored_by; plus the ingress identity (case, operation, actor, role, evidence refs, class).
- Not bound: authorization_ref and authorization_request_sha256 (the claim sets them); authority-issued fields (declaration_id, issued_at, status, qualifies_for_capability, independence, reliability, adapter_attestation_ref, declaration_hash, verification_evidence_refs). These are checked only by the echo of the requested fields after the authority answers.
- A pending record is keyed by ingress identity. A second request for the same ingress with different content reuses the first record; if that record is approved, a claim with the other content fails as content_mismatch (terminal). Pending records written before this change are never claimable (fail closed).
- The approver sees the pending record and its hash, not the declared content.
- Live round-trip against a running gateway not performed.

**Next:** Owner review of the content binding and the unbound field list, then the live round-trip and PR (not done here).
