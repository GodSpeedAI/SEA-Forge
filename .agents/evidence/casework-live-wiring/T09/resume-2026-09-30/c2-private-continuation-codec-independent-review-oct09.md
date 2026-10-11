# Private continuation codec grant — independent document review

**Result: HOLD pending one test-vector clarification.** This review accepts the architectural boundary and normative alignment, but does not accept the exact draft hash as-is because the token cap vector can be read as requiring an impossible valid exact-cap token.

## Reviewed evidence

- Grant draft SHA-256: `19be806076544b1f06323c6186bf8aa1484eef4831d907b6827a213c62f16906`.
- Approved C2 supplement SHA-256: `8ccf30be4131de57e6118bdbdd9f272c20edfd47bf0915f0fe74c7c5edfe3b11`.
- Approved registration/raw-input decision SHA-256: `f0a5ec83e9a84e7ffa45ad69e1df743082f07fc26f45413d7cb5a3077f2c32c8`.
- Source anchors inspected: `crates/sea-forge-ledger/src/lib.rs` and `src/signing.rs`. `sea_forge_ledger::signing` is a public module; it publicly re-exports `SigningKey` and `VerifyingKey`, and `sign_bytes` / `verify_signature` are public. The existing signature representation is `ed25519:` plus standard Base64 of 64 bytes: 8 prefix bytes + 88 Base64 bytes = 96 ASCII bytes. The documented path is valid.

## Findings

The proposed signed payload, field-specific digest encoding, validation order, ACK/frontier separation, no-authority statement, and future ledger revalidation align with REQ-C2-RANGE-001/-004/-005 and V-C2-RANGE-01/-03. The separate registration decision does not require this private codec slice to touch `ServerState`, ledger I/O, dispatch, or the registered stream; the proposed module-only scope is consistent. No new dependency or payload Base64 layer is proposed. No other material deviation from the approved requirements was found.

**Required clarification — impossible exact token cap success.** With a 2,048-byte payload and fixed 97-byte separator-plus-signature suffix, the largest structurally valid token is 2,145 bytes. Therefore no valid token can be exactly 4,096 bytes. Clarify the listed “token 4,096-byte and payload 2,048-byte exact/cap-plus-one cases” as admission-stage boundary tests: 4,096 bytes passes only the outer token-length check and then fails the payload-size/layout check; 4,097 bytes fails at the outer length check before slicing/splitting; separately, a 2,048-byte payload with its fixed suffix is admitted to signature verification and a 2,049-byte payload is rejected before signature verification. Do not require a valid token at the 4,096-byte boundary.

**Small implementability clarification (non-blocking).** State whether the private decoder accepts `&str` (then UTF-8 validity is guaranteed by its type and `.len()` is the byte length) or accepts raw bytes and validates UTF-8 after the size check. This will make the invalid-UTF-8 instruction concrete without changing the wire policy.

## Scope and disposition

This is a documentation-only review. No source changes, compiler, tests, gates, Git, or status commands were run. The hold is limited to clarifying the cap-vector classification in the grant; once corrected, request a fresh review of the corrected exact document and retain the explicit root implementation-release gate. No runtime, DTO, dispatch, startup-key, ledger-I/O, or public-contract work is approved by this review.
