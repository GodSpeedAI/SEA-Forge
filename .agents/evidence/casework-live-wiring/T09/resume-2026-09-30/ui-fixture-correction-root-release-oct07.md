# Root release: narrow UI fixture corrections

Root reviewed independent `ui-private-focused-attempt01-independent-review-oct07.md` (6a24f619) and the relevant exact test/source ranges. Actual focused failure and two tsc diagnostics are valid observations; the review's phrase rejecting verification evidence means these do not provide GREEN approval or demonstrate an unsubscribe implementation defect. The captures remain immutable valid failed-run evidence.

Revision4 requires unsubscribe to cancel its pending timer. Therefore only cursorOrder line279 is released to assert no pending timer instead of requiring a timer to fire. All subsequent no-delivery, exact cursor and independent case delivery assertions remain mandatory. Bounds line475 and settlement line99 are released only to assert optional response cursor presence explicitly, then preserve cursor equality with the narrowed value. No production source changes are authorized by this fixture correction.

A fresh different builder must preserve all three exact preimages, document the original assignment and diff, then a different independent critic must review before any sole-owner runtime rerun. This is a correction of fixture contradictions and type errors, not weakening behavior or changing a public contract. T09 remains partial.
