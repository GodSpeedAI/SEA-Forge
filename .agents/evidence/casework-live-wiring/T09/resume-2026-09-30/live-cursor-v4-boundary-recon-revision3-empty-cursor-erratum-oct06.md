# Live cursor V4 boundary recon revision 3 — empty-cursor factual erratum

Date: 2026-10-06  
Scope: factual correction only. Preserve the original recon, revision 3, and its independent review unchanged. Public V4 remains HOLD.

## Correction

The revision 3 review identifies an exact query-presence distinction in `apps/godspeed-casework-go/internal/server/server.go:188-220`: the handler enters historical lookup only when `r.URL.Query().Get("cursor")` is **nonempty**. A nonempty value calls `Store.At` and renders that retained revision (`:188-207`). An explicitly supplied but empty `?cursor=` does not call `Store.At`; it falls through to the live case/newest-case branch (`:209-230`).

Therefore the precise current behavior is:

- nonempty cursor value: retained historical response;
- absent or empty cursor value: live path, using explicit case if supplied, otherwise newest case or the current synthesized empty-world response.

This clarifies request classification only. It does not approve a new bootstrap arm, alter the current empty-world response, establish complete inventory/frontier/readiness, or authorize a public API/schema change. Preserve review SHA `fbd7a5f482f32cd632e23479d536fe7a03306ab717cdd0400b30c30bdf3c9e59` and its original finding; this note is a new erratum, not an overwrite.

## Source/review anchor

The correction follows the direct handler check at `server.go:188` and the independent revision 3 factual review's “Remaining factual precision issue” at `live-cursor-v4-boundary-recon-revision3-independent-review-oct06.md:81-106`. No test or runtime claim is made.
