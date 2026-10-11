# Gitleaks exceptions

## Test private key fixture

- Fingerprint: `b4464a91de97f7a12ac86a86afaf4bf9e691344a:crates/sea-forge-ledger/src/types.rs:private-key:377`
- Rule: `private-key`
- Location: `crates/sea-forge-ledger/src/types.rs`
- Classification: Intentional test fixture
- Reason: The key material exists only to exercise ledger serialization,
  redaction, or conformance behavior. It is not used to authenticate, sign,
  decrypt, or protect production data.
- Reviewed: 2026-07-13

## Fake API-key fixture

- Fingerprint: `b4464a91de97f7a12ac86a86afaf4bf9e691344a:crates/sea-forge-ledger/tests/conformance_m0_ledger.rs:generic-api-key:490`
- Rule: `generic-api-key`
- Location: `crates/sea-forge-ledger/tests/conformance_m0_ledger.rs`
- Classification: Intentional test fixture
- Reason: The value is a deliberately fake API-key-shaped string used to
  verify evidence redaction or secret-handling behavior. It cannot authenticate
  to any service.
- Reviewed: 2026-07-13

## Source hash reference in policy-format result

- Fingerprint: `617dac3ddf78b660ca95f1c7a53fdf59b87653d5:.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-policy-gofmt-repair-result-oct07.md:generic-api-key:83`
- Rule: `generic-api-key`
- Location: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-policy-gofmt-repair-result-oct07.md:83`
- Classification: Source integrity reference
- Reason: This Markdown line records the SHA-256 of the committed private
  source file `apps/godspeed-casework-go/internal/server/run_observation_key.go`.
  Root recomputed the blob from the finding's commit using `git show
  <commit>:<path>` and SHA-256; the equality check against the line's recorded
  source digest succeeded (digest prefix `a6f0114d`). The complete
  90-character line is the expected filename-plus-source-hash reference. It is
  file-integrity metadata, not credential material and not used for
  authentication or authorization.
- Reviewed: Root source proof and independent review, 2026-10-07; see
  `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-gitleaks-exception-independent-review-oct07.md`.
- Canonical `just security`: passed on 2026-10-07 (exit 0; Gitleaks reported no
  leaks across 516 commits / 84.29 MB; all three root capture archives compared
  equal to their originals). The normal push retry remains pending.
