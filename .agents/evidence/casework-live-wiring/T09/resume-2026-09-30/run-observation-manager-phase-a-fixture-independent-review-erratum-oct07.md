# Phase A fixture review wording erratum — 2026-10-07

This erratum applies to
`run-observation-manager-phase-a-fixture-independent-review-oct07.md`. Its
sentence “No assertion or case was removed” is too broad because the authorized
repair removes the rejected `cancelOutsideLock` TryLock-spy assertion. Read
that sentence as: **No case or approved lifecycle behavior assertion was
removed; the unauthorized, inconclusive spy assertion was removed as explicitly
required by the repair assignment.** All other approved assertions and all
nine cases remain.

This is a wording correction only. It does not change the source review or its
bounded verdict.
