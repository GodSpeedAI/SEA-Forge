# Independent Phase B semantic source review: REJECT

Review date: 2026-10-01. Read-only review; no tests or compiler were run, and no implementation or test source was edited.

## Governing instructions and frozen identities

Reviewed the original run-children runtime assignment, runtime phases, fixture clarifications, real-kernel supplement, Phase A immutable rejection, and actual frozen Phase B source/tests. The builder's successful inner race run is not approval: /tmp/t09-phaseb-inner-race-04.raw SHA-256 9c86a66ee1de8ed785b3b45dfdbfc27936ced58ee7d1f15bf0d8280f5adfb9b6.

- internal/ports/ports.go: e4343e489927c553c789ea3dab6f191326ea31e57b765e295fd184a9eddcd686
- internal/adapters/sfwp/frame.go: be8800c437c9f2f699772371a28f14dc068e7c8bc8a543955f6874c0b6259750
- internal/adapters/sfwp/authority.go: 6a047a3a56192eacbb0f8bb816516d872eca944425c796e531b79859d2e21b15
- internal/projection/live.go: d4d517dd68325353e093b3c24ba9e9fb27ec4df4339026b25f94d55288abc706
- internal/projection/builder.go: 3edf1db574480e3d5ce619f4fa9df4c3998127dbb13f6cf3bb84cd1df36d10b2
- internal/projection/store.go: 64da50272f3ffee84d0f461c8043eab0420159cda13833ec7a7a27af2d29e05e
- internal/adapters/sfwp/scoped_runs_test.go: 0013b05f69def466e8434182642b4543e68c5b4ad7fd1a3f3f14608337fcc2b5
- internal/projection/scoped_live_test.go: 361fc0328217bf78d4ac3d7c92dad6f8641e931225e3c78cd58685a7bb4a7800
- internal/projection/unreadable_runs_test.go: de85a2edb9bd05dc5542c7c9850a9cf9caea69dbb491eb21f95fe93913bfcebb
- Preserved pure child fixture internal/projection/run_children_test.go: 4ddb2445a7ff6726c3b6fd93832ac4122b53da4d26e5b54180f9a1cafb802741
- New real-kernel proof fixture internal/projection/scoped_runs_live_test.go: 8e485c64cdc0776d46a8fda6678d22f3bfe98ec837bd8eaaeb94fecced8a9134

## Blocking findings

1. **Unknown requested cases are misclassified and trigger downstream reads.** Baseline commit 0b55022, internal/projection/live.go lines 55–58, returns apperr.KindInvalid immediately if case.list has no record matching the requested CaseRef. Frozen Phase B code instead calls CaseOverview, CaseHorizon, PendingApprovals, and RunsListForCase before checking whether record is nil (current live.go lines 60–82). It then returns Unavailable whenever the list contains any other case (lines 76–81). A valid request for a nonexistent case therefore changes from Invalid to Unavailable and sends more authority reads before reaching that verdict. Preserve baseline not-found semantics: when requested Record.Ref is absent, return Invalid before those later reads, regardless of whether unrelated cases are listed.

   The frozen scoped_live_test.go encodes the defective behavior. Its record-ref case at line 103 changes the only returned record to case_other, so there is no requested case_1 record. Yet shared assertions at lines 121–126 require Unavailable and exactly one scoped call. That is an unknown-case fixture, not a mismatched selected record. Correct the fixture to expect Invalid and no overview/horizon/approval/scoped reads for a missing requested case, including when other cases exist. Keep separate mismatched Overview/Horizon and malformed run cases.

2. **Scoped refusal mapping flattens established classes.** authority.go lines 263–265 wraps any Refusal found in Client.Do errors as KindUnavailable; lines 270–274 do the same for any resp.Err returned by Response.Into. This contradicts existing Refusal.kind() in frame.go lines 517–533 and the adapter's established typed error contract: input_error and related input classes are KindInvalid; identity and separation-of-duty classes are KindAuthorityDenied; server_busy, unsupported_version, request_cancelled, and request_interrupted are KindUnavailable; unrecognized classes are KindInternal. Preserve those mappings and the Refusal error chain. Only a refusal explicitly classified as unavailable may be normalized to Unavailable.

   The new scoped refusal test at scoped_runs_test.go lines 162–178 exercises only error_class: unavailable; it cannot catch the blanket flattening. Add focused input-error, identity-denial, and unknown-class cases through both refusal-return paths as appropriate, while retaining the explicit unavailable-chain assertion.

These are semantic defects even though the builder's focused inner race log passes. The unknown-case fixture currently blesses the changed Invalid/Unavailable meaning; the refusal suite checks only the one class that the implementation hard-codes. Source and test repair must be performed by a fresh builder, followed by a new independent review. No RED/green verdict is issued here.

Disposition: Phase B source REJECT. Root retains the compiler token. Full T09 remains pending.
