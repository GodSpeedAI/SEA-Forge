# Independent review: private run-observation manager proposal

Date: 2026-10-06  
Reviewer: independent Luna source/document critic; proposal author was a different worker.  
Verdict: **REJECT for implementation release; bounded architecture prerequisites remain unresolved.**

## Frozen identity and review boundary

- Proposal: `run-observation-manager-concrete-proposal-oct06.md`, SHA-256 `5b67ddabed59d2915f70c1cbab6eef1defdccc50b9a837956e9802bd0e17c244`.
- Original repair assignment: `run-observation-manager-proposal-original-assignment-oct06.md`, SHA-256 `bfbe804be5c260f5e096e34bacd5041385648a7f8bfd860d9af4318299518036`.
- The approved T09 extension, governing run-observation bounds, cited source inventory/correction, accepted trace/session/physical-admission/selector prerequisites, and actual Go projection Store/Relay/server/port source were reviewed. This is a documentation-only architecture verdict; no implementation or runtime evidence is approved here.

## Requirements the proposal represents correctly

The proposed process-shared `(case_id, run_id)` key, 16-entry bound including initializing/draining entries, no active eviction, one-second per-run floor, selector limit eight, initial logical trace-call limit eight, and ownership of physical concurrency/retry/cooldown by the accepted SFWP client match the assignment (`concrete-proposal-oct06.md:7-10`; original assignment decision bullets 1-2). The explicit `R/U/S/V/A/C/O` definitions, absent-vs-zero list counts, selected-row failure accounting, and no-replacement behavior match the assignment (`:77-116`; assignment bullets 3-5).

The proposal also preserves the 1,024-frame retention, JSON envelope 1 MiB limit, parsed-time/full-ID pruning order, accurate retained/omitted/truncated values, metadata-only typed-unavailable path, and bounded per-run dedupe (`:163-177`; assignment bullet 9). Its lock discipline, initialization single-flight, same-key draining behavior, watcher-specific reauthorization, cancellation ownership, and worker join before map-slot removal are directionally aligned with the assignment (`:119-161`; assignment bullets 6-8). The listed test-first decomposition is bounded, and public SSE/V4 wiring remains explicitly held (`:179-196`).

The proposal is honest that the stored horizon is only knowledge as of a retained cursor, that run ports do not carry watcher identity, and that no manager implementation/runtime proof exists. Those limits are correctly distinguished from live kernel truth and source approval (`:43-59,207-214`). The cohort exhaustion ambiguity is identified with the relevant two readings and fixture dimensions (`:104-114,199-202`), as required by the assignment. I do not treat the proposed exhaustion formula as approved policy.

## Mandatory unresolved findings

### 1. The required present-case guard has no concrete enforcing boundary

The assignment requires a private caller guard that rejects cold/evicted/gapped/history contexts, backed by the latest retained `Revision.Facts.Horizon` and a comparison to the Relay cursor (original assignment, “parent checks” bullet). Actual sources make that check feasible but nontrivial: `Store.Trajectory` can return an empty history and cloned revisions (`internal/projection/store.go:122-139`); `Revision.Facts` is optional (`:41-53`); Relay advances its observed case cursor before capture/append and deliberately leaves a gap when capture fails (`internal/server/relay.go:101-150`); `CursorForCase` exposes that observed cursor (`:170-175`).

The proposed API does not encode or perform this guard. `presentCaseContext` consists only of `CaseID`, `Cursor`, caller-populated `ParentItemIDs`, and `Fresh bool` (`concrete-proposal-oct06.md:38-43`). `runObservationManager` has `history RevisionHistory` but no Relay/cursor dependency (`:66-76`); `Prepare` receives the context by value (`:89-93`). The proposal nevertheless says both that “the manager rejects” missing `Facts`, identity mismatch, cold/evicted history, and cursor gaps and that `Fresh` is only a caller assertion (`:51-59`; “Present-case and watcher preconditions,” `:60-65`). There is no named guard function/interface, no returned typed guarded context, and no call path tying a successful guard to `Prepare`.

**Required before release:** specify one private guard boundary and its exact inputs/result. It must obtain/check retained latest facts, case/horizon identity and parent IDs, and compare latest stored cursor with the observed Relay cursor; `Prepare` must accept only that guarded result or the caller must be explicitly responsible and the proposal must remove the unsupported claim that the manager itself rejects those failures. Preserve the no-extra-hydration-read constraint.

### 2. Aggregate watcher/queue memory is unbounded

The per-watcher `updates` channel is declared bounded, but its capacity and overflow behavior are expressly left unresolved (`concrete-proposal-oct06.md:28-35,157-161`). The proposal also states that no watcher-count limit is selected and acknowledges that 16 poller entries do not bound concurrent leases, so queue memory can grow without bound (`:160-161`). The original assignment requires bounded queues and race-safe attach/delivery/detach/shutdown (assignment bullet 8), and the proposal’s own test plan calls for a bounded backlog policy (`:187-190`). A per-queue bound without a total lease/attachment bound is not an aggregate memory bound.

**Required before release:** select a bounded queue capacity, explicit overflow action that does not claim complete delivery, and an aggregate watcher/lease bound (or an equivalently enforceable aggregate memory bound), including attach outcome and count behavior at that bound. This is a required lifecycle contract, not a tuning detail.

### 3. Read-exhaustion semantics are deliberately not decided

The proposal presents `exhausted = (reads_attempted == 8 && R > 8)` but acknowledges the alternative causal interpretation: rows after the first-eight selection boundary are already omitted and may not have been excluded by the read budget (`concrete-proposal-oct06.md:104-114,199-202`). The approved extension defines the field causally (“only when the eight-read limit prevented a known candidate from validation”), while its cohort and read limits are both eight (approved extension `:31,35,39,41,43`). The original assignment explicitly says root/critic must resolve this before fixture implementation. The proposal leaves fixture expectations contingent on that decision (`:179-184`).

**Required before fixture/source release:** root must choose and record the causal interpretation for the simultaneous selection/read-limit case. The proposal is right to expose the ambiguity; its proposed formula cannot be treated as settled.

### 4. Bearer-mode observation policy is still awaiting root decision

The assignment requires explicit, honest behavior within existing development-only bearer constraints (original assignment bullet 7). The proposal specifies that bearer lifetime is request-context-only and makes no revocation guarantee, but then leaves whether observations are allowed or withheld to root (`concrete-proposal-oct06.md:68-71,207-210`). This is a material disclosure/lifecycle branch and is listed as required before implementation in the proposal itself.

**Required before release:** root must select allow or withhold behavior and the tests must distinguish it. No broader bearer guarantee is inferred from this review.

## Other scope and proof notes

The current `/api/events` handler has no selected case input and subscribes globally (`internal/server/server.go:293-318`); current `/api/world` distinguishes live from historical cursor reads (`:177-230`). The proposal correctly keeps production bridge and public route changes held. Consequently, the proposed manager’s `Prepare` caller and private case-selection seam remain a later separately authorized integration concern; this review does not approve route wiring.

The status descriptions `initializing/running/stopping/draining/removed`, bounded map semantics, and no-waits-under-lock rule give a useful reviewable lifecycle outline, but they do not resolve findings 1-4. No claim is made that the proposed APIs compile, that tests exist/are RED, or that cancellation actually joins the accepted client cleanup path. Those require the assigned implementation and separately owned runtime gates after architecture prerequisites are resolved.

## Verdict

**Reject for implementation release.** The proposal correctly captures the main approved bounds and surfaces its uncertainties rather than hiding them. It does not yet provide the required enforceable present-case guard boundary, and it leaves the queue/aggregate lease bound, causal read-exhaustion rule, and bearer disclosure behavior undecided. Resolve these four bounded items in a new proposal revision; this review record remains immutable. Root retains architecture acceptance and runtime ownership.
