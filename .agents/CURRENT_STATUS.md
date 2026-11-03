# Current status

**Status revision:** 211

**Stage:** Security: declaration integrity from the verified case ledger (F-A)

**Summary:** Settlement declarations are read only from the hash-chained case ledgers. The declarations.jsonl view is derived and decides nothing. Reads verify the chain and recompute each declaration hash; any integrity failure is an explicit error, never a skip. An authorised declaration must carry its declared-content hash, which must reproduce the approved request hash from the pending record's ingress hash. Promotion and Thoth read through the same path.

**Verified:**
- cargo test (authority, settlement, thoth, capability, artifact-ip, server, cli): 1094 pass, 0 fail
- 11 tests in cep_settlement declaration_integrity (10 adversarial cases and 1 positive control): tampered hash, altered ledger status, corrupted ledger, forged authorisation id, content mismatch, unbound content, cross-case replay, superseded declaration, view edit
- cargo clippy --all-targets -D warnings on touched crates: clean
- cargo fmt --all: applied

**Limits:**
- Trust anchor: the hash chain detects changes by anyone without write access. A writer that can rewrite a case ledger and its MMR state together is not detected. Checkpoints (signed) exist only in the legacy migration path, and their key is stored in the state root, so they are not an external anchor. An external anchor (key held outside the gateway trust domain) is still required for that threat.
- Integrity failure blocks all promotion and Thoth reads until repaired; it is explicit by design
- Pending records written before this change lack ingress_sha256 and are refused as unscoped
- Live gateway not redeployed with this build

**Next:** Open PR for F-A. Then F-B (resume reconciliation) on its own branch.
