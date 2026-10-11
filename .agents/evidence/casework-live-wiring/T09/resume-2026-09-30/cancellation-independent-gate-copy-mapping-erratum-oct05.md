# Cancellation independent-gate copy mapping erratum

Date: 2026-10-05. This note corrects one historical-copy statement in `cancellation-serialized-independent-approval-oct05.md`; that approval file is left byte-for-byte unchanged. Its SHA-256 at the time this erratum was prepared is `1d6316b40072e3f900c89c63f3d2640f7737de9b5648c4bcc7ea700dcac58405`.

## Correct mapping and result

The prior 12-file archive under `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/independent-cancellation-oct05/` maps to `/tmp/sea-cancellation-rootcritic-oct05-<basename-without-leading-rootcritic->`, for example:

* `rootcritic-focused-preflight.raw` → `/tmp/sea-cancellation-rootcritic-oct05-focused-preflight.raw`
* `rootcritic-focused.raw` and `.exit` → `/tmp/sea-cancellation-rootcritic-oct05-focused.raw` and `.exit`
* Correspondingly, `rootcritic-sfwp-{preflight.raw,raw,exit}`, `rootcritic-canonical-{preflight.raw,raw,exit}`, and `rootcritic-module-{preflight.raw,raw,exit}` map to the same `sea-cancellation-rootcritic-oct05-` prefix.

I compared all 12 files against those correct originals with `cmp`; all 12 are byte-exact. The earlier claim that 10 of those 12 mismatched came from comparing them to a different archive at `/tmp/sea-cancellation-oct05/rootcritic-*`. That comparison used the wrong originals and its result is invalid. This correction supersedes only the historical subdirectory-copy mismatch statement.

The 20 older root-archive files are a separate set: 10 raw and 10 exit files under `/tmp/sea-cancellation-oct05/rootcritic-*` compared to the mappings in `cancellation-gate-evidence-root-findings-oct05.md`; all 20 remain byte-exact as stated in the final approval. Exact copies do not remove that record's serialization finding: the earlier canonical retry overlapped the first canonical command, so those prior gates remain procedurally insufficient.

The four newly rerun serialized gates are another separate set. Their 16 `/tmp` raw/exit/preflight files under `/tmp/sea-cancellation-oct05/final-independent-gates-oct05/` each compare exactly with the same-named repository archive. All four fresh exits are zero, and each completed before the next preflight. No compiler session is active.

## Corrected approval statement

Use `cancellation-serialized-independent-approval-corrected-oct05.md` together with this erratum and the original gate record. It corrects the historical mapping statement without rewriting the already-created report. The scoped Unit5A cancellation and response-cap fixture approval remains supported by the source review and the four fresh sequential gates; no broader T09 completion is claimed.
