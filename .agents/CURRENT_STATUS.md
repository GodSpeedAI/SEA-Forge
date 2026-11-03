# Current status

**Status revision:** 190

**Stage:** Security: settlement authority class derived from the verified ingress

**Summary:** The settlement authority class is no longer a caller-supplied value. Each owning path builds a VerifiedSettlementIngress from its ingress kind and transport, which is validated against the operation, and derive_settlement_authority_class is the single function that returns the class. SWE_SEED declaration and artifact transition resume derive External; CEP settle_with and obligation closure derive Local only with no transport or adapter attestation.

**Verified:**
- cargo test -p sea-forge-authority: all suites pass (cep_settlement: 28 pass, 0 fail)
- cargo check -p sea-forge-authority -p sea-forge-artifact-ip -p sea-forge-server -p sea-forge-cli --all-targets: pass

**Limits:**
- An escalated external declaration is still refused as an error; durable pending state is not implemented yet
- The adapter attestation is produced by declare() after the gate, so the SWE_SEED transport, not the attestation, carries the External class at gate time
- The resume state machine, approval validation, and reconciliation are not implemented

**Next:** Record the durable pending settlement state for an approval-gated external declaration, idempotent by canonical request hash, with no authoritative declaration while pending.
