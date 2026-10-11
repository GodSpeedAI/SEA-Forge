# Private Next unit 1: captured delta projection

Date: 2026-10-08
Builder: Luna manager_lifecycle_scope_recon; source release AFTER current
canonical gate has joined and root explicitly releases edits. No compile token.

Read governing instructions, full approved revision6 and addendum, retained
initializer correction revision3 and applicable accepted lifecycle corrections.
Read current retained version/clone helpers, initial manager projection, contract
frame types and nearby tests before editing. Private only; no public contract,
SSE, UI, dependency, persistent schema, manager or worker changes in this unit.

Root architectural decision: existing immutable runObservationRetainedState is
the captured pollerVersion. Capture its pointer, a copied exact-ID ledger and
prior scalar watermark atomically under manager mu in the later integration
unit. This unit implements only pure projection from those supplied values.
Never inspect manager maps or mutable current state, recapture/retry, or infer
source continuity. No duplicate retained frame storage is introduced.

Allowed new files ONLY:
apps/godspeed-casework-go/internal/server/run_observation_delta.go
apps/godspeed-casework-go/internal/server/run_observation_delta_test.go
First prove both paths absent; preserve existing files/operator work. Native
apply_patch is the persistent authoring writer. Preserve all eleven current
observation source identities from canonical retry02 preflight.

Define private delta/window/gap/watermark types matching revision6 semantics,
using existing contract enum and frame types. Define a pure per-run assembler
that accepts exactly the captured immutable state, copied exact ledger and
prior watermark; returns private run delta, optional gap and candidate watermark
or an error. No public exports, no new public enum/tag/error code. Use existing
typed internal unavailable error infrastructure for invalid/unavailable inputs.
The later lease unit will assemble case-level delta and commit watermarks.

Requirements:
1. Captured H=HighestOrdinal, prior P. Return current-window frames whose exact
   immutable first ordinals exceed P, preserving captured source order. Never
   parse/normalize/order event IDs to determine delivery. Deep-copy optional
   frame pointers so recipients cannot mutate retained state.
2. Gaps use ONLY supplied copied ledger: exact IDs satisfying P<ordinal<=H
   absent from captured window. Byte-sort gap IDs only for deterministic output;
   count equals length. Include prior/current ordinal and exact captured window.
   UnknownMissedFrameCount is true for a reported gap; do not claim source-row
   loss counts. UnobservedLossPossible follows approved revision6 semantics.
3. Exact window metadata: successful generation, source total, retained length,
   omitted difference, truncation; preserve captured accepted timestamp,
   case/run/plan identity and execution/settlement/observation standing. Candidate
   watermark advances to H and captures those standings/window atomically.
4. No frames does not suppress legitimate standing/window-only updates. Output
   frame and gap ID slices must be nonnil when present. Equal/lower opaque IDs,
   evict/reappear and rewritten payload never invent a new first ordinal.
5. Reject invalid/unavailable/terminal-failure markers with zero outputs; never
   serve an old version as current, mutate inputs, partially return a candidate,
   or advance caller state. Successful terminal retained state is valid.

TDD: FIRST add focused tests and a minimal nonworking private seam if needed;
freeze and return for independent source review, then separately authorized
actual expected RED before implementation. Do NOT implement the algorithm in
this first phase. Tests cover P boundary, H boundary, later ledger IDs>H,
captured-window vs later publication independence, exact opaque-ID gaps once,
source ordering, retained timestamps/windows moving both ways, standing-only
delta, optional-pointer ownership, input immutability and unavailable markers.
Use deterministic fixtures; no sleeps, production test hooks, weakened assertions
or public type substitution. Tests can call the pure seam directly; lifecycle
and concurrency are deliberately left to the separately assigned integration.

Return concise NEW immutable result with exact source identities, tests listed,
and full original instructions archived here. No tests/build/formatter writes,
Git, status changes or claim of Next/lifecycle/T09 completion. Independent critic
gets these FULL original instructions plus actual implementation. No evidence
means no approval; a rejection is repaired by a different builder.
