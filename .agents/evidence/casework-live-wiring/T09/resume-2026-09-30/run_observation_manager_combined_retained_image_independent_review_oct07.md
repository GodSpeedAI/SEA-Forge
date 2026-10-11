# Independent review: combined retained image lifecycle design

Date: 2026-10-07  
Verdict: **REJECT as test-first lifecycle readiness; retain the core encoder design for a bounded supplement.**  
Review scope: source/design review only. No source or fixture edits, compilation, tests, build, scanners, or Git operations were performed for this review.

## Reviewed records

- `run_observation_manager_combined_retained_image_design_proposal_oct07.md` (especially §§20–21, 24–70, 74–99).
- `manager-combined-image-root-decisions-oct07.md` (decisions 1–7 and final concurrency paragraph). These later root answers supplement and supersede unresolved choices in the proposal; they do not grant source work.
- Full `run_observation_manager_unit1_lifecycle_implementation_preregistration_oct07.md`, including §§1–6 and “Required ownership and behavior invariants.”
- Full `run-observation-manager-unit1-original-assignment-oct06.md`, revision 6, revision 6 addendum, corrections 1–3, response-line-cap erratum, and their independent reviews.
- Current manager and fixture source, including `run_observation_manager_test.go` shared initializer, rollback, capacity, and stop/drain fixtures.

## Accepted design choices and instruction alignment

The manager-owned outer image is a sound boundary around the unchanged helper: fixed-width outer phase and initializer result, exact nil-current key or one raw helper image, no duplicated key/frames/ledger, and a complete combined-size check before publication. Retaining the prior complete value on refusal and changing only a fixed marker follows the lifecycle preregistration and root decision 6. The proposal also correctly keeps runtime refs, contexts, channels, timers, and dependencies outside the per-poller serialized-value cap, and preserves the separate hydration cap.

The scalar initializer-token difference is resolved by root decision 4. Pointer identity plus one manager-owned publisher is equivalent only while the map entry remains unique and cannot be removed/recreated until the old worker has actually joined. The proposal states those conditions in §58 and its publish check in §66. Keep the helper accepted-version generation distinct. No scalar-token deviation remains if the implementation and tests enforce those conditions.

The stale proposal text in §68/§90 asks root to decide whether an initial read or nonterminal retention failure is retryable. Root decision 2 has answered this: the first failed, invalid, or unretainable selected read counts `A`, creates no Runs row and no synthetic accepted value, returns the normal unavailable initial DTO with successful list counts, records a fixed generic result, and bars more starts for that failed initializer. A later accepted current value may recover from read-unavailable. Shared waiters receive the same initial result; each releases its own failed attachment, and actual worker completion/JOIN precedes capacity reuse. Treat the root answer as controlling and explicitly carry it into the fresh supplement/assignment rather than preserving the stale “if root confirms” branch.

The proposal keeps selected initializer reservations and launches ahead of waiting (§64), which agrees with preregistration §1 and root’s closing direction. Preserve the eight-selected-starts-before-any-wait invariant when adding ready/stop notification. Its lock rule is directionally correct: manager state and immutable publication under the mutex; adapter work, callbacks, cancellation, channel close/send, waits, and JOIN outside it (§§64–70; root decision final paragraph).

The proposed `Key *...` / `Current json.RawMessage` DTO also needs the concrete encoding choice to guarantee the stated absent-field branches. Go’s default `encoding/json` emits nil pointers/slices as `null`; the implementation must use explicit branch-specific structs, `omitempty` where sufficient, or a canonical custom marshaler, and must prove that nil-current output omits `Current`, current output omits wrapper `Key`, and `RawMessage` embeds an object rather than a quoted string. This is a bounded encoding clarification, not a reason to change the architecture.

## Blocking test-first gaps

The five proposed cases in §78–85 cover image shape, exact combined cap, `+1` refusal, oversized nil key, and pointer guard. Those are useful encoder/publication tests, but they do not establish the lifecycle behavior now required by root decision 2 or the preregistration. Root decision 7 also explicitly says lifecycle fixtures must separately prove JOIN-before-reuse and shared initializer survival. The original bounded assignment requires failed-Prepare rollback, a surviving watcher not stranded, and actual read return plus worker JOIN before freeing a slot; preregistration §§3, 5, and 6 retain those obligations.

The existing `TestRunObservationManagerSharedInitializerSurvivesInitiatorCancellation` exercises one successful shared initializer (fixture lines 1137–1214). The stop/drain fixture exercises a blocked successful read and capacity held through its return/JOIN (lines 1217–1267). The partial-Prepare cancellation fixture exercises its own rollback (lines 1049–1125). They do not exercise a *shared first read failure* with all registered waiters observing the same unavailable result and independently releasing their own refs. The existing success-path fixtures therefore cannot stand in for the following first-read failure test.

Before lifecycle implementation, the separate test-first supplement needs channel-controlled assertions that:

1. Two or more authorized refs join one initializing entry, then its sole initial read fails (and cover first unretainable/invalid outcome if those share this branch).
2. Every waiter gets the one root-approved unavailable initial DTO/result with all successful list counts present; the selected item counts as `A`, no Runs row or synthetic accepted helper/current state appears, and no retry starts for this failed initializer.
3. Each waiter releases only its own failed ref. No waiter cancels or removes work still needed by another waiter; cleanup has an owner even though no lease is returned.
4. The actual read/worker returns and the worker closes `workerDone`; no entry/poller capacity is reusable before the required owner JOIN and drain completes. Assert capacity remains counted while a channel-controlled read/retirement is blocked.

The ready/stop race also remains under-specified. §68 allows a failed initial candidate to leave only controls and lifecycle phase; §69 closes `ready` after result/current installation but does not define waiter resolution when a concurrent Stop or final detach makes publication inadmissible. The supplement must specify a complete waiter select/notification protocol for both published readiness and manager/lease stop, with each relevant close or signal performed outside the actual mutex and exactly-once ownership. A waiter must not depend exclusively on `ready` if the worker can be prevented from publishing by a phase change. Add a deterministic channel-controlled stop/detach-versus-initializer-publication fixture proving no waiter strands, no channel is double-closed, cancellation occurs outside the mutex, and the actual worker is joined before removal/reuse. Retain the “reserve and start all selected initializers before waiting” assertion in that same lifecycle test plan.

These are material lifecycle requirements, not additional retained-image encoder cases. Do not approve lifecycle implementation from the five pure image tests alone. A new fixture file may remain the only fixture write; existing manager/helper/policy fixtures stay byte-identical as root decision 7 requires.

## Required bounded next step

Reject the current proposal as complete test-first readiness. A fresh supplement should:

- adopt root decision 2 in place of the stale unresolved retry text;
- spell out the branch-specific canonical encoding needed to ensure absent fields and raw nested JSON;
- define ready, stop, and detach signaling ownership and the inadmissible-publication waiter path, with all closes/signals outside the mutex;
- add the shared initial-failure/ref-release/actual-JOIN test and the stop-versus-ready race test to the new fixture plan; and
- preserve all eight selected initializer launches before the first wait, exact A/C mapping, pointer-identity plus JOIN-before-reuse rule, and all frozen fixture hashes.

This review grants neither fixture authoring nor implementation. A separate reviewer and root must approve the bounded test-first supplement before any source/test write or compiler token. No public API, Next/delta/SSE, schema, auth-policy, dependency, or broader lifecycle scope is approved.
