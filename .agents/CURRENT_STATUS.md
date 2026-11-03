# Current status

**Status revision:** 193

**Stage:** Security: shared settlement-declaration verification and quarantine at every reconciliation path

**Summary:** One shared verification (sea-forge-authority cep::verify_case_declarations) decides whether a settlement_declaration is authoritative. An external declaration (SWE_SEED verifier, adapter attestation, or authorisation reference) is authorised only when it names a settlement pending record whose request_sha256 and operation match, and that record is approved, claimed, or completed by this declaration. Anything else is quarantined: recorded once per reason as settlement_quarantined in the case ledger, and never correlated. Startup reconciliation, read-time reconciliation, and the artifact completion read all use it. Local declarations keep their behaviour. The declaration carries authorization_ref and authorization_request_sha256, included in declaration_hash. Legacy declarations with no provenance are quarantined, not grandfathered.

**Verified:**
- cargo test -p sea-forge-server --lib swe_seed_reconciliation: 9 pass (startup quarantine idempotent, approval for one run does not authorise another, local declarations not quarantined, plus the correlation cases now driven through the real approve-and-claim lifecycle)
- cargo test -p sea-forge-server --test conformance_m16: 19 pass, 2 ignored (t16_7 startup and t16_8 read-time now assert an unauthorised out-of-process declaration is quarantined and does not correlate)
- cargo test -p sea-forge-authority -p sea-forge-artifact-ip -p sea-forge-settlement -p sea-forge-capability: all pass
- cargo fmt --all -- --check: clean

**Limits:**
- Not yet done: the server path does not use the approval lifecycle for an External Allow verdict, and the outright claim of a pending record does not yet require Approved for External (checkpoint c)
- Not yet done: out-of-process writers can still append declarations; they are refused at verification, not at write (checkpoint b)
- Not yet done: conformance_m8 CLI tests (checkpoint d); race test (checkpoint e); adversarial review

**Next:** Checkpoint b: write provenance on every gated declaration request and refuse an External declaration without it at the settlement writer.
