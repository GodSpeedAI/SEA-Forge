# Phase B dependency-map clarification — 2026-10-07

This is a supplement to
`run-observation-phase-b-implementation-dependency-map-oct07.md`; it preserves
the original map and records several exact ownership/classification edges.

## Cohort selection and result classification

The manager's list is classified before DTO construction: successful decoded
list gives all six count pointers, with `S=min(8,R)`, `V+A+C=S`, and
`O=(R-S)+C` (lifecycle preregistration lines 104–116). The selected first-eight
ordering comes from `selectObservationRuns` (`run_observation_selection.go:14–49`);
present-context membership is available in `runObservationPresentContext.parentIDs`
(`run_observation_present_context.go:19–23, 42–75`). Do not backfill a rejected
selected row with a later row. Poller-cap refusal is C; invalid/unavailable
initial read or retained-value refusal is A. A too-large nil-current complete
image is A before entry/ref/worker creation, whereas `marshalRunObservationPollerImage`
rejects the full wrapper (`run_observation_poller_image.go:68–95`). A pure
inner helper cap result alone cannot establish that classification.

For a post-read complete-wrapper refusal, the candidate and wrapper are
temporary values. Do not publish the candidate pointer. The lifecycle keeps
the existing current/prior safe copy and fixed marker as authorized by its
classification; it does not mutate the published prior. If no prior exists,
the selected actual read is still counted and classified A. The complete
wrapper must be marshaled and measured before changing `entry.current`; its
nil-current and current branches have different canonical shapes.

## Terminal scheduling and retained-state ownership

`runObservationRetainedState.Availability` already carries
`retainedTerminalRetentionFailure` and `retainedStopScheduling`
(`run_observation_retained_version.go:19–50`). The pure candidate helper rejects
future publication from either prior terminal marker and returns a deep safe
copy (`:94–100`); generation overflow creates the stop-scheduling marker
(`:101–105`); terminal ordinal overflow or terminal over-budget with prior
state creates terminal-retention-failure (`:128–166`). Root retained-publisher
decisions §1 and §5–6 specify marker preservation/recovery semantics and state
that the helper does not schedule reads. Phase B's worker must consult this
existing result/state while deciding whether to continue recurring reads; it
must stop recurring scheduling for terminal-retention-failure and
stop-scheduling, while read-unavailable and nonterminal retention-unavailable
may recover. No terminal boolean or helper-result field is authorized. The
first terminal over-budget candidate may return rejected with nil state, so the
worker/lifecycle already holding the actual snapshot must make the stop decision
without inventing an extra persistent field.

## Read count and preregistration reconciliation

The root successful-Prepare read-count decision (lines 6–30) controls: count
actual `RunTracePort` invocations only. Deriving the successful Prepare's count
from its creator-owned new-initializer slice is valid only after proving every
slice member made exactly one actual call. Stop/cancellation/final-ref before a
first call is a typed failed Prepare with zero DTO/nil lease, not A. Shared
waiters and existing-current reuse have no new initializer; a real shared first
failure is successful A with owner/waiter counts 1/0. A Prepare-local increment
at the port-call site is the direct alternative and adds no persistent state.

One historical clause needs to be read in order: lifecycle preregistration
§3 (lines 117–125) mentions a generation-bound initializer token; later
approved corrected read-start proposal lines 117–136 explicitly rejects a
persistent claim/initializer token, extra worker, and read-start counter,
relying on the sole worker and exact map membership. The corrected proposal and
root read-start decision are the later accepted design; Phase B must not carry
the older token forward unless root explicitly revises that decision. The
current Phase A source already has `prepareOperations`, `cohorts`, `stopping`,
and the poller's fixed `phase`, `initResult`, `current`, readiness/context/JOIN
channels. No further lifecycle state is identified by this recon.

## Citation correction and limits

The prior map's sentence about pointer cloning referenced `retained_version.go`;
the actual file is `run_observation_retained_version.go`, where
`cloneRunTraceFrame` and `cloneRunObservationRetainedState` are at lines
179–211. No source changes were made. This supplement does not approve an
implementation or claim execution proof.
