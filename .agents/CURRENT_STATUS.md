# Current status

**Status revision:** 212

**Stage:** Security: lease-gated settlement resume reconciliation with attempt fencing (F-B)

**Summary:** An open settlement claim is failed by reconciliation only after its lease lapses (claimed_at + 900s). A live claim is left alone, so a concurrent resume cannot fail an in-flight declaration. Complete, fail and release are fenced to the attempt that holds the claim. Each attempt closes once, through one create-only marker, so a worker and a reconciler cannot both settle it. The CLI releases a claim only on settlement_authority_unavailable; other errors leave it open for reconciliation. Descriptor timeouts are capped at 600s, leaving 300s of the lease for work after the authority returns. A failure before the authority call releases its claim; an ambiguous failure after the call keeps the lease.

**Verified:**
- cargo test (authority, artifact-ip, cli, capability, thoth, settlement, server): pass, 0 fail
- cep_settlement lease_and_fencing: 7 tests (live claim survives reconciliation, lease boundary at -1ms and exact, stale attempt fenced after release and reclaim, worker after reconciler, reconciler after completion, 8 concurrent reconcilers close once, two simultaneous resumes claim once); concurrent test run 12 times clean
- cargo clippy --all-targets -D warnings on touched crates: clean
- cargo fmt --all --check: clean
- Merged F-A review fixes: directory-only ledger reads with a stray-file regression test; declaration_integrity is 11 tests (10 adversarial, 1 positive control)
- descriptor timeout boundary: 600s accepted, 601s refused (conformance_m0_authority)

**Limits:**
- Lease safety assumes the authority call is bounded by the descriptor timeout. Verified for the SWE_SEED command transport (deadline plus process-group kill). Other external adapters must meet the same bound.
- A claim whose worker dies mid-call with an external side effect already applied, but before the case ledger records it, is failed by reconciliation. The external effect is not visible to the ledger and needs authority-side reconciliation. This change does not address it.
- A declaration that lands after its lease lapsed is quarantined, and the approval is spent. This is an explicit operator recovery path, not automatic.
- Declarations are not bound to an attempt number. Fencing covers the ledger outcome only; the single-declaration rule relies on the named-declaration count.
- No dedicated CLI test for the error paths that release or keep a claim.
- Lease uses the host clock of the single writer. Clock skew between writers is not modelled.
- Trust anchor: the hash chain detects changes only by readers without write access. A writer that rewrites a case ledger and its MMR state together is not detected. Signed checkpoints exist only on the legacy migration path, with their key in the state root, so there is no external anchor. An external anchor is still required for that threat.
- Integrity failure blocks all promotion and Thoth reads until repaired; it is explicit by design.
- Pending records written before the F-A change lack ingress_sha256 and are refused as unscoped.
- Live gateway not redeployed with this build.

**Next:** Open PR stacked on F-A. Then Horizon/Casework parity and the live round-trip (runbook step 6a).
