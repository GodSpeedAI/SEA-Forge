# Gitleaks exceptions

## Test private key fixture

- Fingerprint: `b4464a91de97f7a12ac86a86afaf4bf9e691344a:crates/sea-forge-ledger/src/types.rs:private-key:377`
- Rule: `private-key`
- Location: `crates/sea-forge-ledger/src/types.rs`
- Classification: Intentional test fixture
- Reason: The key material exists only to exercise ledger serialization, redaction, or conformance behavior. It is not used to authenticate, sign, decrypt, or protect production data.
- Reviewed: 2026-07-13

## Fake API-key fixture

- Fingerprint: `b4464a91de97f7a12ac86a86afaf4bf9e691344a:crates/sea-forge-ledger/tests/conformance_m0_ledger.rs:generic-api-key:490`
- Rule: `generic-api-key`
- Location: `crates/sea-forge-ledger/tests/conformance_m0_ledger.rs`
- Classification: Intentional test fixture
- Reason: The value is a deliberately fake API-key-shaped string used to verify evidence redaction or secret-handling behavior. It cannot authenticate to any service.
- Reviewed: 2026-07-13
