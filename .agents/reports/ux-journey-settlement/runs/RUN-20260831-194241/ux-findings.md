# Screenshot-Backed UX Findings

## Affordance & Continuity Diagnosis
Findings below are anecdotal observations from reviewing this run's screenshots and captured DOM text; they are not independently re-verified the way gate results are, and should be read as qualitative notes rather than pass/fail claims.

1. **Clear Action Paths**: Primary affordances ("Create case", "Run preflight", "Commit case", "Approve"/"Reject") are prominently labeled controls the harness could locate by accessible role/name.
2. **Governed Denial Surface**: The `guardSimFail` denial surface renders an explicit guard id and reason rather than a blank or crashed page.
3. **Fail-Closed Preview Surfaces**: `UnbackedSurface` renders its "nothing here can be evidenced yet" state from a real `system.hello` negotiation rather than a hardcoded claim.
