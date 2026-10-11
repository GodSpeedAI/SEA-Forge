# C2 V4 revision 7 DOC-only build result

Date: 2026-10-08

## Full original builder instruction

> Fresh C2v7 repair builder (you critiqued but did not buildv6; fresh from FORMATbuilder). Read FULLoriginalv6grant in buildresult, originalv5fullassignment, actualFULLv6candidate, yourrejection, NEWrootclarification c2-legacy-bounds-and-u64-root-clarification-oct08.md, approvedsixrecommendationsrecord. Allowed NEW files ONLY live-cursor-v4-complete-candidate-revision7-oct08.md +revision7-build-result-oct08.md. Preserve completev6 bytecontent/decisions except additions to reader/client/verification sections resolving EXACTtwofindings, title/revision/history identity updates. Rootrule legacy validatesboundedscan torealpinnedhead then returns existingvector/requestedprefix; capbeforecoverage typedunavailable NOpartial success; largescansrequirev2; callersmust surface typedunavailable, nomislabel complete/no fallback to unbounded. TypeScript canonicaldecimalu64 checkedBigInt range≤18446744073709551615, max20digits grammar0|[1-9][0-9]*, noNumber/parseFloat/parseInt/lexsort; opaque IDsneverparse. Tests sparse >500rows legacyfailsclosed, exactlimit/headcomplete boundary, v2progress, fullu64 extremes/roundtrip/malformed. Userexplicitly approvedall6recommendations; do NOT ask again sameapproval. Design/runtime verification holdsremain, candidate approvalrequiresfreshindependentcritic; recordrecommendationsapprovalwithout claimingoperatorreadhash. Nativeapply_patch writer. No source/compiler/Git/gates. Fullgrant/archivepreposthashes/completenessdelta in newresult. Escalate narrowly if unclear.

## Input and output identities

The preserved revision-6 candidate preimage is 25,674 bytes, SHA-256 7e9009c5d733c8517531e705b9e5b2d6478bda5c9e4a551ce03e65a46e8b258d. The new complete revision-7 candidate is 29,077 bytes, SHA-256 ad82478fbe008358841b3be2ff0e29139f0327f1dd7cbbcbaf798a2795bd09b6.

The full revision-6 assignment/result is archived in live-cursor-v4-complete-candidate-revision6-build-result-oct08.md (SHA-256 b5e598b165974f07c063b31d39fb4f478e0db14e47a9a32b3baed414833574fa). The independent revision-6 rejection is live-cursor-v4-complete-candidate-revision6-independent-review-oct08.md (SHA-256 9cc1b3dcfcdc8d300deabe7c8549f87ef0703afb4b2fb65116a2258d35b2fc93). The added root clarification is c2-legacy-bounds-and-u64-root-clarification-oct08.md (SHA-256 44e82f523b4063d4aafc88ea5fd716fc457632cf2b0bd1e0f545ad558099a8ef). The recommendation approval record is c2-six-recommendations-operator-approval-oct08.md (SHA-256 7651f333fcf27b741db557d0dcbd4aba140e06b670edb49d70968f267be6be4b). The full revision-5 assignment remains live-cursor-v4-complete-candidate-revision5-full-assignment-oct08.md (SHA-256 535ea0bd1ed5a7717edbddbcb7422b779ed9949fc01653ed18e434d072672187).

## Revision-6 to revision-7 completeness delta

The complete revision-6 candidate is carried forward. Its global-frontier correction, all prior bootstrap, journal, writer inventory, capture, authorization, digest, history, Store/SSE, error, cap, drain, recovery, and verification decisions are retained.

Only these corrections and necessary status updates were made:

1. **Legacy bounded reader:** the old vector shape is successful only after a bounded scan proves coverage to the real pinned head. If row, byte, lock, or time caps stop proof early, return typed unavailable and no vector or partial success. Callers surface the error; large history requires v2 paging; no unbounded compatibility fallback.
2. **Full-u64 ordinal wire:** all ordinal wire fields use canonical decimal strings. Rust/Go use checked u64 adapters; TypeScript validates the decimal grammar, 20-digit maximum, and checked BigInt u64 range. BigInt alone is used for comparisons; Number conversion, parseInt, parseFloat, and lexical ordering are prohibited. Opaque cursors/digests remain untouched.
3. **Tests required after approval:** sparse history beyond 500 rows fails closed; scan-limit/head-complete boundary and requested prefix semantics; v2 progress; zero, MAX_SAFE_INTEGER and successor, u64 maximum, overflow, malformed strings, wire round trip, and opaque IDs.
4. **Approval status:** the candidate records the operator's explicit approval of all six recommendation-level areas and does not claim approval of the exact revision-7 hash. It does not ask for the same six approvals again. It preserves independent review, authored spec/schema/ADR updates, complete writer audit, and runtime gates as pending. Any material decision outside the approved recommendation areas still requires a specific decision before implementation.

No other material revision-6 candidate decision was intentionally removed or changed. The candidate remains a DOCONLY proposal, not source or runtime readiness. No source, compiler, test, formatter, scanner, build, or Git operation was performed.

## Review handoff

The complete candidate and this result are ready for a fresh independent critic to compare against the full revision-6 assignment, revision-5 assignment/rejection, root global-frontier clarification, the two new clarifications, and the operator approval record. This build does not claim the operator read or approved the exact candidate hash. C-2 source, schema, and runtime HOLDs remain pending that independent review and the still-required design and implementation proof.
