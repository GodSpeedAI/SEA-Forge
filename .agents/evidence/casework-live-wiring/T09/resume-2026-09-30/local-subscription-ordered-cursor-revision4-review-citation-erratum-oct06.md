# Revision 4 independent review — citation erratum

Date: 2026-10-06

The immutable review `local-subscription-ordered-cursor-revision4-independent-review-oct06.md` (SHA-256 `893ddd2e2e75ed63b14689fa19ddbbb8f00d6e11e5725b523e57922eaba359ab`) contains proposal line references that are offset from the actual 340-line `local-subscription-ordered-cursor-concrete-proposal-revision4-oct06.md` (SHA-256 `6d7baafd41a098cc24556b32ea222c999ffe4ea116998740339acd2086131873`). Do not rely on those quoted proposal line ranges. They should have cited the section names below. This corrects citation navigation only; it adds no semantic claim and does not change the recorded document-only acceptance.

The reviewed content is located in these actual proposal sections:

- Cursor grammar, seed validation, ceiling, and parsing: **“Private state, initialization, and cursor semantics.”**
- Candidate validation, commit/publication order, snapshot atomicity, timer failure, and allocation sites: **“One allocate-and-publish linearization.”**
- Optional `sinceCursor`, omitted-floor `F`, supplied floors, and conformance branch behavior: **“Subscription behavior, including omitted and supplied floors.”**
- Queue capacity, overflow, callback failures, and disposal: **“Ordered bounded delivery and cleanup.”**
- Deterministic obligations: **“Test-first acceptance matrix for a later authorized source task.”**
- Corrected source paths and conformance-current-state note: **“Citation corrections and source evidence.”**

The underlying source evidence in the review points to real, verified files and spans: local adapter snapshot/subscription/append/emit at `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts:146-173,367-435`; port signature at `src/ports/contract.ts:204-216`; actual future-only/replay fixture at `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:109-139`; normative stream rules at `.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`; and canonical cursor/event types at `.agents/reports/interface-contracts/typescript/types.ts:101-110,426-448`. In particular, the actual conformance subscription still uses `snapshot.cursor` at line 113; the captured-head change remains proposed work.

No source, fixture, test, gate, or approval status was changed by this erratum. The earlier review remains preserved as written.
