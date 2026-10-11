# Final preflight archive correction

Date: 2026-10-08

The first two manually transcribed Base64 preflight archives are preserved but
invalid. This final archive was produced from the retained original
`/tmp/sea-observation-graft-build-oct08-EL8icd/preflight.raw` bytes and is the
authoritative preflight archive. It decodes byte-for-byte to that original.
The original preflight is 1742 bytes with SHA-256
`820675c70745e11ef20c6e9c30af72e56cf756ec897b2d204830a5c87aa04a64`.
The original command, preflight exit, stdout, stderr, and exit archive entries
each independently decode/compare to their retained `/tmp` originals. This
note supersedes the malformed preflight references in earlier receipts; it
does not alter the Graft invocation or its result.
