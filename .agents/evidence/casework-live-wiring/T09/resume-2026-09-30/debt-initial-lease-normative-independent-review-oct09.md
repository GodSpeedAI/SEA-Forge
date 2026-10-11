# Independent review — bounded lease and reader debt updates

**Disposition: REJECT CW-37 wording pending one narrow correction; CW-38 is supported.** This is a review of `.agents/DEBT.md` CW-37 at lines 1433–1453 and CW-38 at lines 1455–1480 against the cited evidence. No tests, compiler, gates, or Git operations were run.

## CW-38 — bounded initial lease unit

The entry correctly closes only the bounded Prepare lease bookkeeping unit. The final independent review explicitly limits approval to Prepare state and scalar watermark snapshot, then disclaims Next/wake/drain, reader runtime, public wiring, settlement, and T09 completion. It reports all three gates with direct exit 0: focused race (33 top-level plus 6 nested), canonical `just casework-go-check`, and full-module race (729 cases, zero failures). It also records root’s six captures per gate and the critic’s independent 18/18 archive-to-original comparisons. The review gives final source hashes and confirms them across preflights.

The debt wording preserves the evidence limitations: canonical03’s earlier formatting failure remains historical; formatter stdout/exit originals are absent even though the final source independently matched `gofmt` and canonical formatting passed (final review §§“Formatting provenance and limits,” lines 50–52). The additive final-review erratum corrects the review’s last sentence so it does not incorrectly deny the authorized gate runs. CW-38 also preserves the earlier RED setup holds, accepts only RED03 as behavioral RED, and leaves Next/T09 open; the RED disposition explicitly says attempts 01/02 did not execute and accepts attempt 03 only for the bounded assignment. This matches the final review §§“Verdict” and “Independent gate evidence” (lines 3–5, 15–30), its erratum, and `initial-lease-red-root-disposition-oct08.md`.

## CW-37 — reader debt

The updated status and next step correctly say the proposal and four additional policies were approved, the repaired normative package was independently approved, and implementation/runtime/migration proof remains open. The normative review approves only documentation fidelity, so the entry does not overclaim code or runtime completion.

**Finding:** CW-37’s Impact at lines 1446–1449 still says “The proposed1MiB raw row may omit valid1MiB event payloads once the ledger envelope is included.” The approved operator receipt and final normative spec set the raw-row admission limit to 2 MiB including LF (receipt item 2; `.agents/specs/casework-live-cursor-v4-spec.yaml:107–116,266–300`). The current sentence describes a superseded 1 MiB proposal as if it were the active design. Update this impact to describe the approved 2 MiB raw-row limit and its actual admission consequence, or explicitly label the 1 MiB concern as historical and resolved by the revised proposal. Keep CW-37 open for implementation and runtime/migration proofs.

## Scope preservation

CW-26 at lines 1016–1052 retains the provenance limits for the post-result reader traceability correction and missing formatter process captures. CW-39 at lines 1482–1494 remains an active cross-project compiler constraint and says no foreign process was interrupted. Neither entry is contradicted by the lease review: its gates ran only after preflight recorded no competing heavy process (final review line 17), and it continues to distinguish those successful runs from earlier blocked attempts.

No debt text was edited for this review. No other entry, source, test, evidence, status, or Git state was changed.
