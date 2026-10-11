# Independent review: global ordinal frontier clarification

**Disposition: APPROVE this clarification as a DOC-only resolution of the revision-5 ordinal-gap finding.** This is not approval of a complete V6 candidate, public schema, implementation, or runtime readiness. Existing C-2 holds remain.

## Evidence and assessment

The reviewed clarification is `live-cursor-v4-global-frontier-root-clarification-oct08.md` (SHA-256 `1ec8293012b5b846c344c833c3b55653af77315ac61c460008482348249782c1`). It resolves the prior review’s concrete ambiguity: per-case ordinal gaps cannot establish missing rows because the append ordinal belongs to the shared ledger. Case A rows at ordinals 10 and 12 are consistent with an intervening Case B row at 11. The proposed reader instead validates a contiguous global scan through a pinned head, including non-event rows, before filtering events by case. Its global `scanned_through_ordinal` can advance over an empty filtered page; the per-case watermarks remain separate.

This distinction matches the existing source model. `crates/sea-forge-server/src/events.rs:61-88` appends event records with optional `case_id` to the common ledger. `events.rs:125-130` resolves cursors to ordinals in the full entry list. `events.rs:169-208` reads that list, applies cursor and ordinal bounds, then filters to event records; the filter is not a per-case ledger. `crates/sea-forge-server/src/ledger/types.rs:191-198,701-723,1029-1042` defines and assigns the append ordinal over that shared ledger. These anchors support the clarification’s global-versus-case distinction; they do not establish that its proposed V2 reader exists.

The conservative failure rule is coherent: if global coverage or continuation integrity is unproved, the affected cell remains unavailable and the failed page is not acknowledged. Case-local isolation is allowed only after global coverage is established and the affected case is independently known. This avoids turning an unknown global gap into a fabricated case-local conclusion. The clarification also explicitly updates revision-6 language and requires interleaving, non-event, empty-filtered-page, and malformed-global-row tests.

No concrete contradiction remains in this bounded semantic issue. Proposed reader behavior, recovery, and tests remain unimplemented and unverified. **C-2 stays held** pending the complete candidate, required operator approvals, and its separate source/runtime gates.
