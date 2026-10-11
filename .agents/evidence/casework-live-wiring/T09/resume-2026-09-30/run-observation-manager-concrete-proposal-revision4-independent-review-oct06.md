# Independent architecture review: run-observation manager proposal revision 4

Date: 2026-10-06  
Reviewed artifact: `run-observation-manager-concrete-proposal-revision4-oct06.md`, SHA-256 `ece24b1308c949ad7489c42edfe28aa9419b3596e8a0af2b0ab53fbf8807b600`.  
Disposition: **REJECT as a complete implementation blueprint; keep manager and public wiring on HOLD.** DOCONLY review; this does not authorize implementation, contract changes, tests, or runtime claims.

## What revision 4 repairs

Revision 4 closes the two central revision 3 review findings. `InitialCohort` now wraps the exact `contract.RunTraceObservation` DTO (proposal lines 93-96), including the listed/selected/validated/unreadable/unavailable/omitted count pointers, read budget, run states, standings, frame metadata, and optional command pointers. It requires the existing `boundRunObservationHydration` helper, not a pre-prune size rejection (lines 410-428). The helper does deep-copy, deterministic global-oldest frame trimming, exact count adjustment, and typed failure only when metadata itself cannot fit (`apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go:22-77,80-149,152-194`; root acceptance `run-observation-hydration-cap-root-acceptance-oct06.md`).

Revision 4 also declares `RunDelta`, `RunWindowGap`, `WindowState`, `ObservationDelta`, typed categories, and a concrete `attach` signature (proposal lines 98-179, 305-357). It preserves the original manager assignment's 16 pollers, maximum eight selected/initial logical reads, one-second per-run floor, accepted shared-client ownership of retries/cooldown/concurrent reads, exact count categories, and the explicit proposal-only status of the separate 16-lease and per-poller 1 MiB retention caps. The additional caps are not represented as approved policy (lines 348-357, 499-510). The source trace adapter really does return exact identities, validates known rows, rejects duplicate IDs within one response, and keeps only the last 1,024 allowlisted frames in input order (`apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go:66-97,99-177`).

These are material improvements and are accepted as document corrections, not as proof the full contract is now implementable. The original revision 3 independent review remains applicable wherever revision 4 has not expressly corrected it. The present-context guard, authorization seams, cleanup, initial DTO outcome and no-public-wiring constraints remain required.

## Blocking findings

### 1. The bounded watermark cannot establish “new” across source rewrites or ID reappearance

The operator-approved contract says later poll updates contain **only newly observed frame IDs** (T09 extension proposal lines 27, 39). The underlying source provides exact per-response event IDs and a bounded suffix, but no monotonic ID, append-only/cross-poll continuity token, or validated cumulative counter: `ReadRunTrace` parses each returned row and retains the rolling last 1,024 (`run_trace.go:99-177`), while `ports.RunTraceSnapshot` count is scoped to that response. Therefore a bounded set of IDs from the most recent window cannot prove an ID absent from that set has never been previously disclosed. If ID `e` is disclosed, leaves the 1,024-frame window, then reappears in a later accepted response, the proposed algorithm classifies `e` as new once it is no longer in `SeenEventIDs` (proposal lines 127-133, 373-395). The explicit disclaimer against exactly-once behavior across source rewrite/eviction (lines 392-395) does not resolve the stronger “only new IDs” rule.

This is not a demand for an unbounded ID ledger or a fabricated count. The proposal needs a bounded, reviewable continuity rule that preserves the accepted output contract: for example, a source-proven continuity/generation signal, or fail-closed/no-frame-delta behavior when continuity cannot be established. It must state how loss uncertainty is represented without inventing a missed-frame number. Until then, `Next` can replay a previously observed identity as “new,” so the producer/consumer contract is unresolved.

### 2. `EvictedUnseenEventIDs` is not derivable from the watermark as specified

The watermark is initialized in `Prepare` to the exact complete current source window (proposal lines 361-371), but `InitialCohort.Observation` is subsequently passed through the 1 MiB helper, which may remove the oldest initial frames (lines 410-428). `SeenEventIDs` is defined as IDs in the last source window (lines 127-133), not IDs actually disclosed in the initial DTO or a later committed delta. The gap algorithm then asks whether an evicted prior-window ID “was not disclosed” and promises its exact reporting (lines 385-388), without defining a separate bounded disclosed-ID set or another way to recover that fact.

Consequently an initially helper-pruned frame is in `SeenEventIDs` even though the caller never received it. When that ID is later absent, the implementation cannot infer from this watermark alone whether it was sent. Conversely, `SeenEventIDs` is not an unbounded historical record of disclosed IDs, so reappearance after eviction cannot be distinguished from first observation. Define distinct bounded source-window and disclosed-identity state, precisely define the lifetime/accounting of each, and state what happens when an ID is no longer available. If a fact is unknowable from the retained source window, report uncertainty; do not manufacture `EvictedUnseenEventIDs` or a numeric loss count.

### 3. Snapshot-commit retry has no bounded progress outcome

`Next` builds a candidate off-lock, rejects it if any generation changed, and repeats until all captured generations remain current at commit or the context/lease stops (proposal lines 397-408). A 1-second floor constrains each poller, not the aggregate generation churn or the duration/number of retries across the up-to-eight attached pollers. Under continuous accepted poll updates, a lease can repeatedly lose the global commit race and starve without returning any delta. A context deadline eventually terminates one caller, but is not a manager-level progress or ownership policy and does not make a long-lived caller safe from indefinite retry.

Specify a bounded retry/commit strategy and its result. It must not advance watermarks on a discarded candidate, falsely report a gap/count, or lose the pending wake. Suitable designs may linearize a stable set of immutable per-entry snapshots or bound optimistic attempts and return a documented retryable outcome while preserving the wake and watermark. The proposal currently requires all-or-retry without a bound or progress guarantee.

### 4. Guard status and `Next` error inventory contradict actual source and the proposal's own outcomes

The source reference section calls `internal/server/run_observation_present_context.go` a held guard stub (proposal lines 573-575). That statement is stale: the current file at SHA-256 `19bd9a574a25beab1e76824ddf136a3091e418d77e7573068c8d3e9464a3b1c2` contains `checkRunObservationPresentContext` validation at lines 25-76, including newest-row-only selection, case/cursor/reference consistency, nonnil horizon, duplicate/blank parent rejection, exact Relay cursor match, and copied parent IDs. The proposal should cite the implemented private prerequisite and its accepted assignment/critic evidence, without implying manager integration or continuous freshness.

Separately, `runListUnavailable` is present in `observationErrorKind` (lines 144-161), and the refused/over-cap/undecodable-list outcome says the completed empty lease's `Next` reports `ErrRunListUnavailable` (lines 279-285, 475-482). But the enumerated `Next` errors at lines 207-211 omit it. This leaves the declared private method result contract inconsistent on a specified path. Add the typed error to the Next inventory or alter the outcome consistently; keep failed-list count pointers absent as the accepted DTO contract requires.

### 5. Exhaustion remains marked unresolved despite root's explicit executor decision

Revision 4 repeats `Exhausted=(ReadsAttempted==8 && R>8)` as a proposal and then calls the interpretation unresolved (lines 260-303, 525-526, 571-572). Root has since directed the manager proposal to preserve this executor interpretation: true iff eight logical reads started and `R>8`; false for fewer than eight reads, cache/shared reuse below eight, or `R==8`. This is the required working interpretation because the eight-read and eight-selection caps overlap. It is **not** a claim that the schema sentence is unambiguous or that operator approval changed. The document must record the interpretation as root's executor decision, preserve the caveat, and not defer it as undecided.

## Concrete private contract items still needing blueprint clarity

The type block still refers to `*cohortLease` and `*pollerEntry`, and to `RunObservationManager`, without defining their state/ownership fields (proposal lines 305-333). The later lifecycle prose is substantial, but an independent implementer cannot verify from the declared state model which entry owns the worker cancel/done channels, how each lease's wake notifier reference is held/released, which set records actual disclosure, or how a shared initializer's ref handoff is linearized during rollback. Since rollback correctness and no-send-after-close are central accepted requirements, this is more than naming style. Add compact concrete manager/lease/entry fields or an equally exact state/ownership table; keep every cancel, callback, wait, and join outside the lock as already required.

The failure table also says selected trace failures produce an unavailable candidate result and that whole-Prepare errors roll back partial refs (lines 199-211, 475-491). Preserve the assignment's distinction: selected row unavailability is counted in `A`, while cancellation/auth/guard/list infrastructure/assembly failures fail the cohort; every failed `Prepare` owns rollback since it returns no lease to its caller. Revision 4 addresses this in prose and is directionally sound; this review does not authorize loosening it.

## Contract/source checks and retained decisions

- The approved extension defines the DTO count-pointer rule, selected/read limits, statuses, shared poller bounds, and the “only newly observed frame IDs” phrase (`.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md:27,31,35,39-45`). The normative spec says the trace event is informational, has no SSE ID, and is not a case revision (`.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-106`). Revision 4 correctly leaves public route/event mapping and frontier changes held.
- Exact `RunTraceObservation` and `RunTraceRunObservation` fields are present in `internal/contract/contract.go:202-238`; the proposal's exact DTO wrapper/helper requirement matches them. Helper source hash is `3dfa2993004676a13b1dd9c449965c3b531a199885db325a6fd14941a90f501a`.
- The source has a 1,024-frame rolling ring (`run_trace.go:103-177`) but offers no cross-read append-only proof. No source-proven missed-frame total may be derived from `TotalFrameCount`; the proposal is right to leave that total unknown.
- The present-context check is as-of the captured trajectory/Relay cursor and does not make authorization atomic. The actual source and root-released guard prerequisite are evidence of a private helper only, not approval of manager callsites, lease state, or public readiness.
- The extra 16-cohort cap and 1 MiB serialized per-poller cap remain proposal-only pending separate review and operator approval. The initial 1 MiB DTO bound is accepted and distinct; neither it nor the proposed per-poller budget proves whole-process memory bounds.
- Physical retry/concurrency/cooldown remain with the accepted shared client owner. Exact source identity and parent validation, no backfill, truthful count equations, and as-of limitations remain required.

## Required next revision and verdict

Produce a fresh complete proposal revision addressing findings 1-5 and defining enough private state/ownership to audit the concurrency contract. Preserve revision 4 and this review immutably. Do not soften the “only newly observed IDs” contract to fit an insufficient bounded watermark, and do not turn unknown source gaps into numeric loss claims. Update the guard evidence to current source and record the root-selected exhaustion interpretation with its ambiguity caveat. Keep both aggregate policy proposals explicitly unapproved and public wiring held.

**No implementation release.** This review ran no compiler, tests, scanner, Graft build, Git, or network action and made no source/test/status/debt changes.
