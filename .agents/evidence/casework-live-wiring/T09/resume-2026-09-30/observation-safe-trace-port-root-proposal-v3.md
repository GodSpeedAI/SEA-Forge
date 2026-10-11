# Unit5 safe trace port: root proposal v3 for independent source critique

Preparation only. This proposal authorizes no fixture or production writes and
no runtime work. It defines the already approved trace disclosure projection;
it does not authorize session sharing, pollers, SSE, UI implementation, or full
T09 settlement.

## Semantic boundary

Add a separate application `ports.RunTracePort` with
`ReadRunTrace(ctx context.Context, caseID, runID, planItemID string)` returning
`ports.RunTraceSnapshot` and error. The snapshot contains exact run/case/item
identity, execution and settlement strings, safe frames, and a safe-frame
count. A safe frame contains only event ID, kind, original timestamp, optional
execution status, and optional int64 exit code. Domain structs have no JSON
wire tags, following neighboring semantic ports. The gateway stamps the later
observation time; it never replaces a frame's recorded timestamp.

Implement this port on the existing SFWP Authority with exactly one logical
`Client.Do` using `NewRunGet(runID)`. Preserve the existing inspect transport
retry inside `Do`. Keep `RunArtifactProvenanceView` and artifact authorization
unchanged; use a separate decoder and projection for this disclosure. Add no
kernel verb or dependency. Do not put this data in `CaseFacts`, `Store`, or
captured world snapshots.

Reject blank expected identities before dialing, using the existing invalid
input error convention. For a completed read, require nonblank and exact run
ID, case ID, and plan item ID matching all three expected values. Unknown
ownership or mismatch is unavailable with no safe result. Require execution
in pending/enabled/active/completed/failed/terminated and settlement in
unsettled/accepted/rejected/escalated. Preserve authority refusal classes;
malformed responses and invalid disclosure projections are unavailable.

## Trace presence and returned-row boundary

Require `records` to be an array containing exactly one entry whose `record`
is exactly `trace.jsonl`. That entry must have `present: true` and `bytes`
represented as a nonnegative exact JSON integer within `u64`. Missing,
duplicate, absent, false, malformed, or missing-byte trace metadata makes the
read unavailable. Validate ownership, standing, response shapes, and this
presence metadata before returning a safe result. Do not infer trace presence
from `trace` being an array.

Require `trace` to be an array. Missing, null, or wrong-shaped trace is
unavailable. An empty array is a valid result of **zero returned safe rows**
after exact ownership, standing, response-shape, and trace-record-presence
validation. It is not proof that the source journal was empty, fully read, or
fully parsed. Do not compare `bytes` with the returned array length or reject
empty/nonempty rows based on byte size: `run.get` reads trace before it later
collects filesystem metadata, so these values are not an atomic snapshot.

The current `run.get` reader can return no rows on over-cap or read failure
and stops at the valid prefix after malformed JSONL. Its response has no
parse-completeness signal. Therefore the safe-frame count is only the number
of validated allowlisted rows in the `trace` array returned by this successful
`run.get` response, before port retention. Any later cohort total, retained,
omitted, and truncated accounting also refers only to returned frames. UI
wording must say returned frames/rows where counts appear; it must not imply
source-journal totals, full trace completeness, or parse completeness.
Presence and byte size do not cure this limitation. Do not fabricate a source
count or parse status. Resolving source completeness requires a separately
authorized kernel contract/behavior change and is out of scope.

## Projection, identity, and count rules

Select only the ten canonical safe kinds in `types.ts:306-316`. Omit unknown
kinds completely, including their IDs and payloads; an unknown-kind row cannot
invalidate a selected row. Among allowlisted/selected rows only, require a
nonblank original event ID, an RFC3339/RFC3339Nano timestamp, and no duplicate
selected event ID anywhere in the full returned array before retention.
Thus a selected duplicate invalidates the response even when one occurrence
falls beyond the newest-1024 retention boundary. Unknown-kind rows do not
participate in selected-ID uniqueness. Preserve original source order and
timestamp text. Do not sort or manufacture IDs or times.

The total safe-frame count is the number of validated allowlisted rows in this
returned array before retention; unknown kinds do not count. Retain the newest
at most 1024 safe rows in source order. Later cohort serialization may retain
fewer to satisfy the approved one-MiB envelope bound, but must preserve this
returned-array count and report accurate retained/omitted/truncated fields
within the same returned-frame boundary. This port makes no claim that the
upstream trace journal or total cohort wire bytes are bounded by 1024.

For `command_finished`, inspect `payload.execution.status` and
`payload.execution.exit_code` only when supplied by an object payload. A
missing, null, or non-object whole
`payload` contains no execution metadata and means both optional values are
absent. For an object payload, missing/null `execution`, `status`, or
`exit_code` means that optional value is absent. A supplied nonnull
`execution` must be an object; supplied nonnull `status` must be a canonical
command execution status, and supplied nonnull `exit_code` must be a JSON
integer within both int64 and the JavaScript exact interval
[-9007199254740991, 9007199254740991]. Fractional, string, boolean, overflow,
or unsafe supplied values make the entire read unavailable. Never silently
discard or round a supplied nonnull status or exit code. Outgoing canonical
optional fields remain omitted when absent and nonnull when present. Do not
interpret payload metadata on other kinds. Actor, arguments, output,
environment, evidence, and all other raw payload fields are ignored and cannot
exist in the semantic DTO.

## Test-first decomposition after cancellation prerequisite

After the cancellation fixture/repair is independently verified, release a
fresh builder for tests and any explicitly named minimal declarations needed
for a compiling assertion RED. Then an independent critic receives this v3
proposal plus the actual result and must prove the intended RED before
production implementation. Required matrix:

- exact request and all three ownership identities; blank expected identity
  rejected before dialing; all allowed execution and settlement standings;
- all ten safe kinds; unknown-kind omission including a row whose ID duplicates
  a selected row; selected-row ID/timestamp validation; source order; and
  duplicate selected IDs across the full returned array, including a selected
  duplicate outside retained 1024;
- exactly one present `trace.jsonl` record with a nonnegative exact `u64`
  byte count is required; missing, duplicate, absent/false, malformed, or
  missing-byte record metadata is unavailable; missing/null/wrong-shaped
  `records` and trace-presence entries are unavailable;
- empty trace with valid presence metadata yields zero **returned** safe rows
  after ownership and standing checks, for both zero and positive byte counts;
  no assertion labels this source-empty or parse-complete. Nonempty rows with
  zero or positive byte counts are not rejected solely for byte/array
  consistency;
- demonstrate/document that a nonempty returned prefix can be counted without
  asserting source-journal completeness or source totals, including the
  unreadable, over-cap, and malformed-prefix limitation;
- zero, 1024, and 1027 returned safe-row counts and newest-row retention order;
  count/UI assertions use returned-frame wording and boundary;
- command payload missing/null/scalar handling as absent optionals; object
  payload execution missing/null handling; supplied nonnull execution shape;
  optional metadata absent/null versus outgoing omission; safe integer
  boundaries, unsafe adjacent values, fraction/string/boolean, and int64
  overflow;
- malformed standing/trace shape; public synthetic ignored-field markers
  absent from semantic serialization; existing artifact authorization and
  authority refusal behavior unchanged.

This matrix preserves the original cases and the justified presence/returned-
row repairs while correcting v2's unsupported restrictions. Root retains
integration and count semantics; the critic must identify any remaining
contradictions or missing cases. This document is a proposal, not passing
evidence.

## Material changes from the original and v2

1. Preserves v2's exact-one `records` entry requirement for `trace.jsonl`,
   `present: true`, and nonnegative exact `u64` byte representation. This
   establishes reported filesystem presence only, not parse completeness.
2. Removes v2's empty/nonempty trace versus byte-size consistency guards. The
   trace read precedes metadata collection and is not an atomic snapshot;
   empty returned arrays remain valid zero-returned-row results after required
   validation, regardless of reported byte count.
3. States directly that empty returned trace does not prove an empty source
   journal, and that unreadable/over-cap input or malformed-prefix parsing can
   produce zero or partial returned rows without a parse-completeness signal.
   Counts and UI wording remain scoped to returned frames only.
4. Restores uniqueness to selected allowlisted rows across the full returned
   array before retention. Unknown kinds and their IDs/payloads are omitted
   entirely and cannot poison selected rows. Selected duplicates beyond the
   retention boundary still invalidate the read.
5. Removes v2's nonnull whole-payload object requirement. Missing, null, or
   scalar payload has no execution metadata and yields absent optional fields;
   a supplied nonnull `execution` still must be an object, and supplied
   nonnull status/exit values retain exact validation.
6. Retains v2's returned-array safe count, explicit source-completeness
   limitation, required trace presence metadata, and test coverage for
   malformed records shape and returned prefixes.
7. Preserves the original identity/refusal rules, ten safe kinds, selected-row
   timestamp and ID validation, source order, newest-1024 retention,
   int64/JavaScript-safe-number rules, no-sensitive-fields boundary, separate
   port, and unchanged artifact path.

## Source anchors

- Original semantic port, transport, identity, projection, and optional
  metadata proposal: `observation-safe-trace-port-root-proposal.md`.
- Prior reviews and clarifications:
  `observation-safe-trace-port-proposal-independent-review.md`,
  `observation-safe-trace-port-proposal-independent-review-anchor-clarification.md`,
  and `observation-safe-trace-port-independent-review-v2-oct05.md`.
- `crates/sea-forge-server/src/sfwp/run_views.rs:425-436`: `read_jsonl`
  returns empty on cap/read failure and stops at the first malformed line.
  `:240-250` defines record filename/present/optional bytes;
  `:85-95` lists canonical records; `:283-326` exposes defaulted trace and
  records arrays; `:836-855` reads trace before assembling the run; `:870-880`
  gathers record metadata later.
- `crates/sea-forge-core/src/types.rs:478-492,499-536`: command execution
  status and trace event kind/payload types.
- `.agents/reports/interface-contracts/typescript/types.ts:306-335,338-372`:
  ten frame kinds, optional safe command fields, run standings, and observation
  count fields.
- `apps/godspeed-casework-go/internal/ports/ports.go:276-305`: neighboring
  semantic run-summary port types.
- `apps/godspeed-casework-go/internal/adapters/sfwp/frame.go:308-313,686-694`
  and `authority.go:744-780`: `NewRunGet`, separate artifact provenance view,
  and current artifact read path.
- `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml:82-105`:
  informational run-trace boundary and its bounds.

Graft retrieval preceded direct source-anchor verification. No source code,
fixture, test, or runtime was changed or approved by this proposal.
