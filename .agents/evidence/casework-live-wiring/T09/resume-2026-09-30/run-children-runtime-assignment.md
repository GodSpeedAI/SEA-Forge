# Scoped lists and captured run children runtime assignment

Prepared by root; HOLD implementation until independent run-child fixture assertion RED.
Read original test-first assignment, root fixture clarifications, current repaired fixture,
runtime-integration-decisions.md, run-observation-source-preparation.md and scoped instructions.
This implements unit4 only. Original fixture remains frozen throughout implementation.

Owned production files under apps/godspeed-casework-go: internal/ports/ports.go,
internal/adapters/sfwp/frame.go and authority.go, internal/projection/live.go, builder.go
and store.go. New focused scoped adapter/source/store tests may be added separately;
do not change existing tests, golden files or approved Ask source. No dependencies,
Rust, UI, hydration, polling, SSE or current observation buffers in this unit.

Add semantic RunListResult with Runs []RunSummary and UnreadableIDs []string; no JSON
tags or wire types. Add CaseAuthorityPort.RunsListForCase(ctx, CaseRef)(RunListResult,error),
preserving RunsList and all constructors. Add NewRunListForCase(caseID string)(*Request,error)
encoding run_list and case_id; empty/whitespace-only request case fails invalid before dial.
Legacy NewRunList remains byte-for-byte unscoped. The scoped adapter preserves all actual
summary fields, standings/timestamps and unreadable IDs. Missing/null required list arrays,
blank/duplicate/overlapping run IDs, missing/foreign readable case or missing actual item
ID, unknown standings, malformed timestamps or negative evidence count fail honestly as
typed unavailable. Preserve valid ID strings; never normalize or invent ownership.

LiveSource validates requested nonblank case, selected Record.Ref, Overview.Ref and
Horizon.Ref. Use exactly one scoped list call, never legacy list or a fallback. Validate
readable case ownership, actual captured horizon parent, distinct IDs and no readable/
unreadable overlap even for non-SFWP port implementations. Absent parent fails honestly;
do not require Overview.RunIDs or HorizonItem.RunIDs membership because reads are not
jointly atomic. Keep readable Runs and distinct factual UnreadableRunIDs in CaseFacts;
clone that unreadable slice at Store retention/read boundaries. Never retain trace frames,
RunTraceObservation or poller state in captured facts/history. Scoped ownership is not
an upstream directory-enumeration or pagination bound.

Pure Build emits exactly one execution_trace CognitiveObject per valid owned summary,
using actual run ID and actual captured item ParentID. All24 execution/settlement pairs
must expose separately labelled actual standings through existing badge/explanation;
reuse existing cognitive status mapping without implying accepted settlement for merely
completed execution. No actions, invented percentage, new CognitiveObject fields, fake
IDs/parents or eight-child cap. Omit invalid rows; omit every duplicate ID and colliding
child while preserving valid siblings and parent objects. Unknown standings are invalid.
Preserve parent consequential actions, primary focus/narration and relative parent ranking;
run status alone must not manufacture attention. New child IDs may enter salience rank.
Ensure deterministic output for the same captured facts.

Tests must prove exact scoped case_id request and unchanged legacy request, complete
summary/unreadable mapping, refusals/malformed/null/duplicate/overlap/foreign cases, zero
wire contact for invalid request refs, exactly1 scoped call and0 legacy calls, mismatched
Record/Overview/Horizon and absent actual parent failures, and unreadable slice cloning.
Use explicit scoped fake methods; a nil embedded port must not supply a panic fallback.
Identify focused tests before implementing behavior and record any unavoidable declaration
dependency that prevents a compiling pre-implementation assertion RED. Existing pure
child fixture's approved RED remains mandatory. Never loosen tests or update goldens.

Builder runs no compile until root transfers sole token. Freeze source hashes, exact
test coverage/deviations and integration choices. Independent critic receives these
ORIGINAL instructions plus original fixtures/clarifications and actual resulting source.
Critic must rerun focused race and fresh canonical/full-module Go gates with actual-host
RAM/process preflights; new real-kernel scoped list proof is required using owned disposable
cells and existing binaries, no hidden Cargo. Preserve exact raw outputs/exits/preflights.
If a golden assertion changes, report its actual difference for root semantic review;
do not modify it. Any rejection requires a fresh builder. Full T09 remains pending.
