# Unit5 safe trace port: root proposal for independent source critique

Preparation only. No source or fixture write release and no runtime approval.
This unit implements the already approved trace disclosure projection; it does
not authorize session sharing, pollers, SSE, UI, or full T09 settlement.

## Semantic boundary

Add a separate application `ports.RunTracePort` with
`ReadRunTrace(ctx context.Context, caseID, runID, planItemID string)` returning
`ports.RunTraceSnapshot` and error. The snapshot contains exact run/case/item
identity, execution and settlement strings, safe frames and total safe-frame
count. A safe frame contains event ID, kind, original timestamp, optional
execution status and optional int64 exit code only. Domain structs have no
JSON wire tags, following the neighboring semantic ports. The gateway stamps
the later observation time; it never replaces a frame's recorded timestamp.

Implement on the existing SFWP Authority using exactly one logical `Client.Do`
with `NewRunGet(runID)`. Preserve existing inspect transport retry inside Do.
Keep `RunArtifactProvenanceView` and artifact authorization unchanged; use a
separate decoder/projection for this disclosure. No kernel verb or dependency.
Do not put this data in CaseFacts, Store or captured world snapshots.

Reject blank expected identities before dialing, using the existing invalid
input error convention. For a completed read, require nonblank and exact
run ID, case ID and plan item ID matching all three expected values. Unknown
ownership or mismatch is unavailable with no safe result. Require execution
in pending/enabled/active/completed/failed/terminated and settlement in
unsettled/accepted/rejected/escalated. Preserve authority refusal classes;
malformed responses and invalid disclosure projections are unavailable.

## Projection and count rules

Require a trace array; explicit empty array is a valid zero-frame result.
Absent/null/wrong-shaped trace is unavailable. Select only the ten canonical
safe kinds in types.ts306-316. Unknown kinds are omitted entirely, including
their payloads. For every selected row, require a nonblank original event ID,
RFC3339/RFC3339Nano timestamp and no duplicate selected event ID anywhere in
the full array, including rows later omitted by retention. Preserve original
source order and timestamp text. No sorting, manufactured IDs or times.

`total safe-frame count` counts validated allowlisted rows before retention;
unknown kinds are excluded. Retain the newest at most 1024 safe rows in source
order. Later cohort serialization may retain fewer to satisfy the approved
one-MiB envelope bound, but must preserve this actual total count and report
accurate retained/omitted/truncated fields. This port does not claim that the
upstream trace journal or total cohort wire bytes are bounded by 1024.

For command_finished only, inspect payload.execution.status and exit_code.
Raw missing/null execution, status or exit_code means no recorded optional
value; output omits the corresponding field. Nonnull execution must be an
object; a supplied nonnull status must be a canonical command execution
status, and a supplied nonnull exit_code must be a JSON integer within both
int64 and the JavaScript exact interval [-9007199254740991,9007199254740991].
Fractional, string, bool, overflow or unsafe recorded values make the entire
read unavailable. Never silently discard or round a supplied nonnull value.
Outgoing canonical optional fields remain NON-null. Do not interpret payload
metadata on other kinds. Actor, arguments, output, environment, evidence and
all other raw payload fields are ignored and cannot exist in the semantic DTO.

## Test-first decomposition after cancellation prerequisite

Once cancellation fixture/repair is independently verified, release a fresh
builder for tests and any explicitly named minimal declarations needed for a
compiling assertion RED. Then an independent critic receives this original
proposal plus actual result and must prove the intended RED before production
implementation. Required matrix: exact request/ownership; all ten kinds;
unknown omission; malformed selected row and duplicate outside retained1024;
empty/1024/1027 counts and newest order; actual optional command metadata;
raw absent/null vs outgoing absence; safe integer boundaries, unsafe adjacent
values, fractional/string/bool/int64 overflow; malformed standing/trace shape;
public synthetic ignored-field markers absent from semantic serialization;
existing artifact and refusal behavior unchanged. Root retains integration
and count semantics; critic must identify contradictions or missing cases.

Source anchors: ports.go276-295 semantic run summaries; frame.go308-313 existing
run_get constructor and 686-694 narrow artifact view; authority.go744-780
artifact path; run_views.rs116-130 raw trace and 286-326 run record;
types.ts306-365 canonical safe frames/standing; approved proposal section
Run-trace observations. This document is a proposal, not passing evidence.
