# Initial Phase A archive recovery erratum

Recorded 2026-10-01. This is an append-only archive correction record, not test output.

An earlier archive step overwrote two initial log copies with incorrectly transformed
versions instead of preserving them as separate artifacts. The original `/tmp` outputs and
the current raw logs were left unchanged; root independently verified those originals.

The incorrect sandbox-limited archive copy was recovered from the unchanged raw log using
only the documented historical differences: the projection
`readable-unreadable-overlap` subtest duration changed from `0.00s` to `0.01s`, and the
final projection summary's separator after the package path changed from an actual tab to
the literal bytes `\\t`. Its recovered incorrect-copy SHA-256 is
`6cff61f2678524a9e5671a1ac254332ee97e210c655ae20b0be2f059c1b476fb`; it is preserved at
`run-children-phase-a-sandbox.initial-archive-mismatch.log` for historical traceability.

The incorrect host `scoped-red` archive copy was not recovered. The reported prior SHA-256
is `697abd30fc2344adb383a4e4eb230fc3851ea7d8f762b82fb9b278996015e002`, but the candidate
reconstructed from the recorded line differences did not match it. No replacement copy was
created; its exact prior bytes remain unavailable. The unchanged raw log remains
`run-children-phase-a-scoped-red.log` (SHA-256
`a6b05114166fbda6d5e4f02f8fbce8d16316ed1d7dd81f7e5c88db681b394261`).

The recovered mismatch copy is explicitly not a source of test-gate proof. Gate evidence
continues to be the original `/tmp` outputs and unchanged current raw logs, which root
independently byte-verified.
