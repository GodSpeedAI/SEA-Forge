# Historical preflight archive normalization correction

Date: 2026-10-08. Additive provenance correction for one historical record;
not a diagnostic execution result or a security-gate pass.

## Exact identities

The worktree file
`run-observation-primitives-gitleaks-diag-preflight-oct07.raw` was 857 bytes
with SHA-256
`824c110962c6b62f62e3f9f4088555c9249f3da8570c82a69be508398c9cf992` when
read for this correction. Its exact bytes are preserved in
`run-observation-primitives-gitleaks-diag-preflight-oct07-lossless-oct08.json`
using Base64, with the source path, length and digest recorded in the wrapper.

The committed blob is 843 bytes with SHA-256
`7a00a48311963e23f6665d35b5a82e999d6a504e7a12ef6814594255a1e4c643`.
The worktree has 14 CRLF pairs; replacing each CRLF with LF reproduces the
committed blob byte-for-byte. This is the one historical `.raw` record that
Git's CRLF clean filter normalized in evidence-history commit `36bcd1b`.
Root verified the other 454 records byte-exact and all 56 source-directory
records byte-exact. No global Git setting, `.gitattributes`, or history was
changed to address the normalization.

## Provenance limit

This wrapper preserves the current worktree bytes for comparison and
provenance. The underlying preflight file is a historical preflight record;
it is not scanner output and must not be promoted as proof that the diagnostic
command executed or passed. Earlier malformed copies remain historical and
excluded as described in their immutable receipts. This correction does not
rewrite those records or change any gate result.

No source, test, security configuration, debt entry, index, or Git history was
changed. No scanner, compiler, or verification gate was run for this correction.
