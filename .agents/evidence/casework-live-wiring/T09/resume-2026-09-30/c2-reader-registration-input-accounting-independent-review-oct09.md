# Independent review: reader registration and raw-input accounting

Date: 2026-10-09

## Verdict

**Approve this architecture clarification only.** It is consistent with the
approved bounded-reader limits and existing registration boundary. It does
not authorize implementation or claim runtime/readiness proof.

## Evidence and material differences

* Registration is correctly fixed to `ServerState.events_ledger`, created by
  `open_events_ledger`; `events.get_range` uses that registered handle
  (`crates/sea-forge-server/src/lib.rs:67-110,150-200,237-251`;
  `src/sfwp/events.rs:43-45`). The clarification introduces no client path,
  registry, authority, dependency, or async kernel boundary. Separating ledger
  raw-byte/lock/hash work from server token/filter/DTO work matches the
  revision-2 proposal's pinned-head and projection ownership description.
* The cooperative lock remains scoped to the registered stream. The proposal
  requires nonblocking `try_lock` through a monotonic 50 ms deadline; this
  clarification preserves it. The existing ledger writer lock path is
  synchronous and blocking (`crates/sea-forge-ledger/src/types.rs:694-723`),
  so the new reader must use the already-proposed bounded acquisition and
  must not call that blocking path.
* The strict total accounting rule resolves a real ambiguity: revision 2
  specifies a 4 MiB raw-input page budget for rows, non-events and lookahead,
  while describing head pinning and predecessor revalidation separately. It
  does not expressly grant those reads an auxiliary allowance
  (`run-observation-bounded-ledger-reader-design-proposal-revision2-oct08.md`,
  sections “Pinned head and cooperative append boundary” and “Bounded input”).
  The operator receipt approves 4 MiB raw input per page, including
  non-events/lookahead, with oversize input unavailable and no RSS or I/O
  latency guarantee (`c2-bounded-reader-additional-policy-operator-approval-oct08.md`,
  item 2). The spec likewise names one 4 MiB raw-page-input cap and keeps it
  separate from the unchanged row, row-count, frame-count, response and lock
  caps (`casework-live-cursor-v4-spec.yaml:105-119,266-300`). Charging every
  actual read, including repeated pin/predecessor reads, enforces that
  approved total; an uncharged allowance would exceed it. This is a
  conservative accounting clarification within the approved cap, not a
  resource-limit increase.
* The cap-before-read/allocation, no partial-row or unvalidated-lookahead
  acknowledgement, and unavailable-on-no-progress behavior preserve the
  proposal's acknowledged-frontier rules. The required combined auxiliary
  plus forward-read exact-cap/cap-plus-one vectors sharpen the existing
  aggregate-page-boundary proof in `V-C2-RANGE-03` (spec lines 653-663).
* Empty success remains limited to a successfully established absent/zero-byte
  registered stream under lock; unreadable, nonregular, whitespace, torn or
  corrupt data stays unavailable. The clarification adds no symlink ban and
  retains the approved registered-path semantics. Its 2 MiB row cap including
  LF, 1 MiB complete serialized-response cap, and 500-row/frame caps remain
  distinct and unchanged (operator receipt item 2; spec lines 109-118 and
  292-300; ADR-008 lines 209-212).

The cited current source establishes the fixed registration and current lock
mechanism only; it does not establish the proposed bounded reader's runtime
correctness. The clarification properly says so and leaves source release
gated on independent review and normative traceability.
